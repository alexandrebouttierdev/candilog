//! Génération de documents et analyse de CV avec progression et annulation.

use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::ia::domain::*;
use crate::features::ia::infrastructure::{
    charger_config, construire_provider, extraire_pdf, GenerateurLlm,
};
use crate::features::profil::domain::{Profil, ProfilRepository};
use crate::features::profil::infrastructure::SqliteProfilRepository;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

const OFFRE_SYSTEME: &str = r#"Extrais une offre d'emploi en JSON. Recopie uniquement les informations présentes, sans traduire ni inventer. Réponds exactement avec les clés {"titre":"","competences":[],"savoirEtre":[],"experience":null,"motsCles":[]}. Réponds uniquement en JSON."#;
const CV_SYSTEME: &str = r#"Adapte un CV à une offre en JSON. Reformule uniquement les faits du profil, sans ajouter compétence, entreprise, diplôme ou expérience. Conserve toutes les expériences et formations. Réponds avec {"resume":"","experiences":[{"intitule":"","entreprise":"","description":""}],"competences":[],"formations":[{"diplome":"","etablissement":""}]}. JSON uniquement."#;
const ATS_SYSTEME: &str = r#"Compare le CV et l'offre fournis. Réponds en français, uniquement en JSON : {"score":0,"recap":"","suggestions":[],"recommandations":[{"section":"resume","texteOriginal":"","textePropose":"","impact":0}]}. N'invente aucun fait et borne score à 0-100."#;
const LETTRE_SYSTEME: &str = r#"Rédige uniquement le corps d'une lettre de motivation en français à partir du profil et du brief. N'invente aucune expérience ou compétence. Respecte le ton et la longueur demandés. Ne mets ni titre, ni Markdown, ni commentaire autour de la lettre."#;
const PARSE_CV_SYSTEME: &str = r#"Structure le texte brut d'un CV sans traduire, reformuler ni inventer. Réponds uniquement en JSON : {"resume":"","experiences":[{"intitule":"","entreprise":"","description":""}],"competences":[],"formations":[{"diplome":"","etablissement":""}]}"#;
const PROFIL_SYSTEME: &str = r#"Extrais le profil du CV sans inventer. Recopie les valeurs et utilise null ou [] si absentes. Dates au format AAAA-MM ou AAAA. Réponds uniquement en JSON camelCase avec exactement cette structure : {"identite":{"prenom":"","nom":"","email":"","telephone":null,"ville":null,"titre":null,"resume":null,"linkedin":null,"github":null,"siteWeb":null},"experiences":[{"intitule":"","entreprise":"","lieu":null,"dateDebut":"","dateFin":null,"posteActuel":false,"description":null}],"competences":[{"nom":""}],"formations":[{"diplome":"","etablissement":"","lieu":null,"dateDebut":null,"dateFin":null,"description":null}],"langues":[{"nom":"","niveau":""}],"projets":[{"nom":"","description":null,"url":null,"technologies":null}],"certifications":[{"nom":"","organisme":null,"date":null,"url":null}]}"#;

pub struct IaService {
    pool: SqlitePool,
    generations: Mutex<HashMap<String, CancellationToken>>,
}

impl IaService {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            generations: Mutex::new(HashMap::new()),
        }
    }

    pub fn annuler(&self, id: &str) {
        if let Some(token) = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
        {
            token.cancel();
        }
    }

    fn demarrer(&self, id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        self.generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id.to_owned(), token.clone());
        token
    }

    fn terminer(&self, id: &str) {
        self.generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(id);
    }

    fn profil(&self) -> AppResult<Profil> {
        Ok(SqliteProfilRepository::new(self.pool.clone()).obtenir()?.0)
    }

    async fn provider(&self) -> AppResult<Arc<dyn GenerateurLlm>> {
        construire_provider(&charger_config(&self.pool)?).await
    }

    pub async fn analyser_offre(&self, texte: String) -> AppResult<AnalyseOffre> {
        texte_requis(&texte, "L'offre")?;
        let offre: OffreStructuree =
            generer_json(self.provider().await?, &texte, OFFRE_SYSTEME).await?;
        let score = score_profil(&self.profil()?, &offre);
        Ok(AnalyseOffre { offre, score })
    }

    pub async fn generer_cv(
        &self,
        demande: DemandeGenerationCv,
        notifier: impl Fn(ProgressionIa),
    ) -> AppResult<GenerationCv> {
        texte_requis(&demande.offre, "L'offre")?;
        let id = demande.generation_id.clone();
        let token = self.demarrer(&id);
        let _garde = GenerationEnCours { service: self, id };
        self.generer_cv_interne(&demande, &token, &notifier).await
    }

    async fn generer_cv_interne(
        &self,
        demande: &DemandeGenerationCv,
        token: &CancellationToken,
        notifier: &impl Fn(ProgressionIa),
    ) -> AppResult<GenerationCv> {
        let provider = self.provider().await?;
        let profil = self.profil()?;
        if profil.identite.prenom.trim().is_empty()
            && profil.experiences.is_empty()
            && profil.competences.is_empty()
        {
            return Err(AppError::Validation(
                "Complétez votre profil avant de générer un CV".into(),
            ));
        }
        progres(
            notifier,
            &demande.generation_id,
            "Analyse de l'offre",
            15,
            None,
        );
        let offre: OffreStructuree = annuler(
            token,
            generer_json(provider.clone(), &demande.offre, OFFRE_SYSTEME),
        )
        .await?;
        let score = score_profil(&profil, &offre);
        progres(
            notifier,
            &demande.generation_id,
            "Adaptation du CV",
            45,
            None,
        );
        let contexte = serde_json::json!({"profil":profil,"offre":offre,"score":score}).to_string();
        let cv: CvGenere =
            annuler(token, generer_json(provider.clone(), &contexte, CV_SYSTEME)).await?;
        progres(notifier, &demande.generation_id, "Analyse ATS", 78, None);
        let contexte_ats = serde_json::json!({"cv":cv,"offre":offre}).to_string();
        let analyse: AnalyseAts =
            annuler(token, generer_json(provider, &contexte_ats, ATS_SYSTEME)).await?;
        progres(notifier, &demande.generation_id, "Terminé", 100, None);
        Ok(GenerationCv {
            cv,
            analyse,
            offre,
            score_profil: score,
        })
    }

    pub async fn generer_lettre(
        &self,
        demande: DemandeLettre,
        notifier: impl Fn(ProgressionIa),
    ) -> AppResult<String> {
        if demande
            .entreprise
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
            && demande
                .poste
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            && demande
                .contexte
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            return Err(AppError::Validation(
                "Précisez une entreprise, un poste ou le contexte de la candidature".into(),
            ));
        }
        let id = demande.generation_id.clone();
        let token = self.demarrer(&id);
        let _garde = GenerationEnCours {
            service: self,
            id: id.clone(),
        };
        let profil = self.profil()?;
        let contexte = serde_json::json!({"profil":profil,"entreprise":demande.entreprise,"poste":demande.poste,"ton":demande.ton.as_deref().unwrap_or("formal"),"longueur":demande.longueur.as_deref().unwrap_or("medium"),"contexte":demande.contexte,"lettrePrecedente":demande.lettre_precedente,"instruction":demande.instruction}).to_string();
        progres(&notifier, &id, "Rédaction", 20, None);
        let resultat = annuler(
            &token,
            self.provider()
                .await?
                .generer(&contexte, LETTRE_SYSTEME, false),
        )
        .await;
        if let Ok(lettre) = &resultat {
            let fragments = decouper_fragments(lettre);
            for (index, fragment) in fragments.iter().enumerate() {
                if token.is_cancelled() {
                    return Err(AppError::Cancelled);
                }
                let p = 30 + ((index + 1) * 70 / fragments.len().max(1)) as u8;
                progres(&notifier, &id, "Rédaction", p, Some(fragment.clone()));
            }
        }
        resultat
    }

    pub async fn analyser_cv_importe(
        &self,
        demande: DemandeAnalyseCv,
        notifier: impl Fn(ProgressionIa),
    ) -> AppResult<AnalyseCvImporte> {
        texte_requis(&demande.offre, "L'offre")?;
        let id = demande.generation_id.clone();
        let token = self.demarrer(&id);
        let _garde = GenerationEnCours {
            service: self,
            id: id.clone(),
        };
        progres(&notifier, &id, "Lecture locale du PDF", 10, None);
        let texte = extraire_pdf(PathBuf::from(&demande.chemin)).await?;
        let provider = self.provider().await?;
        progres(&notifier, &id, "Structuration du CV", 30, None);
        let cv: CvGenere = annuler(
            &token,
            generer_json(provider.clone(), &texte, PARSE_CV_SYSTEME),
        )
        .await?;
        progres(&notifier, &id, "Analyse de l'offre", 55, None);
        let offre: OffreStructuree = annuler(
            &token,
            generer_json(provider.clone(), &demande.offre, OFFRE_SYSTEME),
        )
        .await?;
        let score = score_cv_importe(&cv, &offre);
        progres(&notifier, &id, "Recommandations ATS", 78, None);
        let analyse: AnalyseAts = annuler(
            &token,
            generer_json(
                provider,
                &serde_json::json!({"cv":cv,"offre":offre}).to_string(),
                ATS_SYSTEME,
            ),
        )
        .await?;
        progres(&notifier, &id, "Terminé", 100, None);
        Ok(AnalyseCvImporte {
            cv,
            offre,
            score,
            analyse,
        })
    }

    pub async fn importer_profil(
        &self,
        demande: DemandeImportProfil,
        notifier: impl Fn(ProgressionIa),
    ) -> AppResult<ProfilExtrait> {
        let id = demande.generation_id.clone();
        let token = self.demarrer(&id);
        let _garde = GenerationEnCours {
            service: self,
            id: id.clone(),
        };
        progres(&notifier, &id, "Lecture locale du PDF", 15, None);
        let texte = extraire_pdf(PathBuf::from(&demande.chemin)).await?;
        progres(&notifier, &id, "Extraction du profil", 45, None);
        let mut profil: Profil = annuler(
            &token,
            generer_json(self.provider().await?, &texte, PROFIL_SYSTEME),
        )
        .await?;
        nettoyer_profil(&mut profil);
        if profil.identite.prenom.trim().is_empty()
            && profil.identite.nom.trim().is_empty()
            && profil.experiences.is_empty()
            && profil.competences.is_empty()
        {
            return Err(AppError::Provider(
                "Aucune donnée de profil exploitable n'a été trouvée dans le CV".into(),
            ));
        }
        progres(&notifier, &id, "Vérification requise", 100, None);
        Ok(ProfilExtrait { profil })
    }
}

struct GenerationEnCours<'a> {
    service: &'a IaService,
    id: String,
}

impl Drop for GenerationEnCours<'_> {
    fn drop(&mut self) {
        self.service.terminer(&self.id);
    }
}

async fn generer_json<T: serde::de::DeserializeOwned>(
    provider: Arc<dyn GenerateurLlm>,
    prompt: &str,
    systeme: &str,
) -> AppResult<T> {
    let mut courant = prompt.to_owned();
    let mut derniere = None;
    for _ in 0..2 {
        let brut = provider.generer(&courant, systeme, true).await?;
        match parser_json(&brut) {
            Ok(value) => return Ok(value),
            Err(error) => {
                derniere = Some(error.to_string());
                courant = format!("{prompt}\n\nLa réponse précédente était un JSON invalide. Renvoie l'objet complet, sans Markdown. Réponse invalide :\n{brut}");
            }
        }
    }
    Err(AppError::Serialization(
        derniere.unwrap_or_else(|| "Réponse IA illisible".into()),
    ))
}

fn parser_json<T: serde::de::DeserializeOwned>(brut: &str) -> Result<T, serde_json::Error> {
    let extrait = match (brut.find('{'), brut.rfind('}')) {
        (Some(a), Some(b)) if b >= a => &brut[a..=b],
        _ => brut,
    };
    serde_json::from_str(extrait).or_else(|strict| {
        jsonrepair_rs::jsonrepair(extrait)
            .ok()
            .and_then(|r| serde_json::from_str(&r).ok())
            .ok_or(strict)
    })
}

async fn annuler<T>(
    token: &CancellationToken,
    travail: impl Future<Output = AppResult<T>>,
) -> AppResult<T> {
    tokio::select! { result = travail => result, () = token.cancelled() => Err(AppError::Cancelled) }
}
fn texte_requis(value: &str, label: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::Validation(format!(
            "{label} ne peut pas être vide"
        )))
    } else {
        Ok(())
    }
}
fn progres(
    notifier: &impl Fn(ProgressionIa),
    id: &str,
    etape: &str,
    progression: u8,
    fragment: Option<String>,
) {
    notifier(ProgressionIa {
        generation_id: id.into(),
        etape: etape.into(),
        progression,
        fragment,
    });
}
fn decouper_fragments(texte: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut debut = 0;
    for (index, caractere) in texte.char_indices() {
        if matches!(caractere, '.' | '!' | '?' | '\n') && index + caractere.len_utf8() > debut {
            out.push(texte[debut..index + caractere.len_utf8()].to_owned());
            debut = index + caractere.len_utf8();
        }
    }
    if debut < texte.len() {
        out.push(texte[debut..].to_owned());
    }
    out.into_iter().filter(|v| !v.is_empty()).collect()
}
fn nettoyer_profil(profil: &mut Profil) {
    profil
        .experiences
        .retain(|v| !v.intitule.trim().is_empty() || !v.entreprise.trim().is_empty());
    profil.competences.retain(|v| !v.nom.trim().is_empty());
    profil
        .formations
        .retain(|v| !v.diplome.trim().is_empty() || !v.etablissement.trim().is_empty());
    profil.langues.retain(|v| !v.nom.trim().is_empty());
    profil.projets.retain(|v| !v.nom.trim().is_empty());
    profil.certifications.retain(|v| !v.nom.trim().is_empty());
    for experience in &mut profil.experiences {
        if experience.poste_actuel {
            experience.date_fin = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extrait_un_json_entoure_de_markdown() {
        let v: OffreStructuree = parser_json("```json\n{\"titre\":\"Rust\",\"competences\":[],\"savoirEtre\":[],\"experience\":null,\"motsCles\":[]}\n```").unwrap();
        assert_eq!(v.titre, "Rust");
    }
    #[test]
    fn fragments_reconstituent_le_texte() {
        let texte = "Bonjour. Suite !\nMerci";
        assert_eq!(decouper_fragments(texte).concat(), texte);
    }
}
