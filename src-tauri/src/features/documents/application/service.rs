//! Validation des CV et lettres avant persistance.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::features::documents::domain::{
    CoverLetter, CoverLetterRepository, NewCoverLetter, NewResume, ResumeRepository, ResumeSummary,
    ResumeVersion,
};
use uuid::Uuid;

/// Borne du JSON d'une version de CV.
///
/// `content` est volontairement extensible — le modèle de génération évolue — mais rester
/// non borné laissait entrer en base un blob arbitraire par un simple appel IPC. La valeur
/// couvre très largement un CV complet sérialisé.
pub const MAX_CONTENT_CHARS: usize = 250_000;

/// Borne du texte d'une lettre, cohérente avec ce qu'un PDF d'une page peut porter.
pub const MAX_LETTER_CHARS: usize = 20_000;

/// Tons acceptés, alignés sur ceux que le rendu de lettre sait interpréter.
const TONES: [&str; 3] = ["formal", "casual", "creative"];

/// Longueurs acceptées, alignées sur celles que le rendu de lettre sait interpréter.
const LENGTHS: [&str; 3] = ["short", "medium", "long"];

pub struct DocumentsService<C: ResumeRepository, L: CoverLetterRepository> {
    resume: C,
    cover_letters: L,
}

impl<C: ResumeRepository, L: CoverLetterRepository> DocumentsService<C, L> {
    #[must_use]
    pub const fn new(resume: C, cover_letters: L) -> Self {
        Self {
            resume,
            cover_letters,
        }
    }

    /// Valide puis enregistre une version de CV.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom est vide ou trop long, si le contenu n'est pas un
    /// objet JSON, ou s'il dépasse [`MAX_CONTENT_CHARS`].
    pub fn resume_save(&self, input: &NewResume) -> AppResult<ResumeVersion> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la version est requis".into(),
            ));
        }
        if name.chars().count() > 120 {
            return Err(AppError::Validation(
                "Le nom de la version est trop long (120 max)".into(),
            ));
        }
        valider_contenu(&input.content)?;
        self.resume.save(&NewResume {
            name: name.into(),
            content: input.content.clone(),
        })
    }

    pub fn resume_list(&self) -> AppResult<Vec<ResumeSummary>> {
        self.resume.list()
    }
    pub fn resume_list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
    ) -> AppResult<Page<ResumeSummary>> {
        self.resume.list_page(page, page_size, search)
    }
    pub fn resume_get(&self, id: Uuid) -> AppResult<ResumeVersion> {
        self.resume.get(id)
    }
    pub fn resume_delete(&self, id: Uuid) -> AppResult<()> {
        self.resume.delete(id)
    }

    /// Valide puis enregistre une lettre.
    ///
    /// Le ton et la longueur sont vérifiés ici comme ils le sont au rendu : les accepter
    /// librement à la persistance laissait la même règle produire deux résultats selon la
    /// couche traversée, et une lettre au ton inconnu échouait ensuite à la régénération.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom, le contenu, le ton ou la longueur sont invalides.
    pub fn cover_letter_save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation(
                "Le nom de la lettre est requis".into(),
            ));
        }
        if name.chars().count() > 140 {
            return Err(AppError::Validation(
                "Le nom de la lettre est trop long".into(),
            ));
        }
        if input.content.trim().is_empty() {
            return Err(AppError::Validation(
                "Générez une lettre avant de l'enregistrer".into(),
            ));
        }
        if input.content.chars().count() > MAX_LETTER_CHARS {
            return Err(AppError::Validation(
                "Le contenu de la lettre est trop long".into(),
            ));
        }
        valider_valeur(&input.tone, &TONES, "Le ton de la lettre")?;
        valider_valeur(&input.length, &LENGTHS, "La longueur de la lettre")?;
        let mut nettoyee = input.clone();
        nettoyee.name = name.into();
        self.cover_letters.save(&nettoyee)
    }

    pub fn cover_letters_list(&self) -> AppResult<Vec<CoverLetter>> {
        self.cover_letters.list()
    }
    pub fn cover_letters_list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
    ) -> AppResult<Page<CoverLetter>> {
        self.cover_letters.list_page(page, page_size, search)
    }
    pub fn cover_letter_get(&self, id: Uuid) -> AppResult<CoverLetter> {
        self.cover_letters.get(id)
    }
    pub fn cover_letter_delete(&self, id: Uuid) -> AppResult<()> {
        self.cover_letters.delete(id)
    }
}

/// Refuse un contenu de CV qui n'est pas un objet JSON borné.
fn valider_contenu(content: &serde_json::Value) -> AppResult<()> {
    if !content.is_object() {
        return Err(AppError::Validation(
            "Le contenu du CV est illisible : générez-le à nouveau avant de l'enregistrer".into(),
        ));
    }
    let serialise = serde_json::to_string(content)
        .map_err(|error| AppError::Serialization(error.to_string()))?;
    if serialise.chars().count() > MAX_CONTENT_CHARS {
        return Err(AppError::Validation(
            "Le contenu du CV dépasse la taille maximale autorisée".into(),
        ));
    }
    Ok(())
}

/// Refuse une valeur hors du jeu fermé accepté par le rendu.
fn valider_valeur(value: &str, acceptes: &[&str], label: &str) -> AppResult<()> {
    if acceptes.contains(&value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "{label} n'est pas pris en charge."
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::documents::infrastructure::{
        SqliteCoverLetterRepository, SqliteResumeRepository,
    };

    fn service() -> DocumentsService<SqliteResumeRepository, SqliteCoverLetterRepository> {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        DocumentsService::new(
            SqliteResumeRepository::new(pool.clone()),
            SqliteCoverLetterRepository::new(pool),
        )
    }

    #[test]
    fn refuse_un_cv_sans_nom() {
        let err = service()
            .resume_save(&NewResume {
                name: "   ".into(),
                content: serde_json::json!({}),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    /// `content` traverse l'IPC en `serde_json::Value` (`unknown` côté TypeScript) et
    /// atterrissait tel quel en base : ni forme, ni borne. Un appel forgé — ou une
    /// génération inhabituelle — y écrivait un blob arbitraire que la bibliothèque devait
    /// ensuite relire à l'aveugle.
    #[test]
    fn refuse_un_cv_dont_le_contenu_n_est_pas_un_objet() {
        for contenu in [
            serde_json::Value::Null,
            serde_json::json!("texte"),
            serde_json::json!([1, 2, 3]),
        ] {
            let err = service()
                .resume_save(&NewResume {
                    name: "CV Produit".into(),
                    content: contenu.clone(),
                })
                .unwrap_err();
            assert!(
                matches!(err, AppError::Validation(_)),
                "contenu {contenu} accepté"
            );
        }
    }

    #[test]
    fn refuse_un_cv_dont_le_contenu_depasse_la_borne() {
        let err = service()
            .resume_save(&NewResume {
                name: "CV Produit".into(),
                content: serde_json::json!({ "resume": "x".repeat(MAX_CONTENT_CHARS) }),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    /// Le rendu de lettre refuse déjà tout ton hors `formal|casual|creative` et toute
    /// longueur hors `short|medium|long`. La persistance, elle, acceptait n'importe quelle
    /// chaîne : la même règle avait deux comportements selon la couche traversée.
    #[test]
    fn refuse_une_lettre_au_ton_ou_a_la_longueur_inconnus() {
        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: None,
                job_title: None,
                tone: "sarcastique".into(),
                length: "medium".into(),
                content: "Madame, Monsieur…".into(),
            })
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(_)),
            "ton inconnu accepté"
        );

        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: None,
                job_title: None,
                tone: "formal".into(),
                length: "interminable".into(),
                content: "Madame, Monsieur…".into(),
            })
            .unwrap_err();
        assert!(
            matches!(err, AppError::Validation(_)),
            "longueur inconnue acceptée"
        );
    }

    #[test]
    fn refuse_une_lettre_sans_contenu() {
        let err = service()
            .cover_letter_save(&NewCoverLetter {
                name: "Lettre Nova".into(),
                company: Some("Nova".into()),
                job_title: Some("Designer".into()),
                tone: "formal".into(),
                length: "medium".into(),
                content: "  ".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
