//! Helpers communs et déclaration des cas de test.
use super::*;
use async_trait::async_trait;
use std::sync::Mutex;

/// Fournisseur de test renvoyant des réponses séquentielles.
struct SeqProvider {
    responses: Mutex<Vec<String>>,
}
#[async_trait]
impl LlmProvider for SeqProvider {
    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
    async fn generate(&self, _prompt: &str, _system: &str) -> AppResult<String> {
        let mut r = self.responses.lock().unwrap();
        Ok(if r.is_empty() {
            String::new()
        } else {
            r.remove(0)
        })
    }
    async fn list_models(&self) -> AppResult<Vec<String>> {
        Ok(vec![])
    }
}

fn engine(responses: Vec<&str>) -> CvEngine {
    CvEngine::new(Arc::new(SeqProvider {
        responses: Mutex::new(responses.into_iter().map(str::to_string).collect()),
    }))
}

/// Fournisseur de test renvoyant une réponse selon la section demandée (via le prompt
/// système), donc **robuste à l'ordre** des appels — indispensable pour tester le mode
/// parallèle.
struct KeyedProvider {
    identity: String,
    history: String,
    skills: String,
    portfolio: String,
}
#[async_trait]
impl LlmProvider for KeyedProvider {
    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
    async fn generate(&self, _prompt: &str, system: &str) -> AppResult<String> {
        Ok(if system.contains("identity block") {
            self.identity.clone()
        } else if system.contains("work experience and education") {
            self.history.clone()
        } else if system.contains("skills and languages") {
            self.skills.clone()
        } else {
            self.portfolio.clone()
        })
    }
    async fn generate_structured(
        &self,
        prompt: &str,
        system: &str,
        _s: &serde_json::Value,
    ) -> AppResult<String> {
        self.generate(prompt, system).await
    }
    async fn list_models(&self) -> AppResult<Vec<String>> {
        Ok(vec![])
    }
}

fn keyed_engine(
    mode: AnalysisMode,
    identity: &str,
    history: &str,
    skills: &str,
    portfolio: &str,
) -> CvEngine {
    CvEngine::with_mode(
        Arc::new(KeyedProvider {
            identity: identity.into(),
            history: history.into(),
            skills: skills.into(),
            portfolio: portfolio.into(),
        }),
        mode,
    )
}

/// Fournisseur de test comptant ses appels et renvoyant toujours du non-`JSON`.
struct CountProvider {
    calls: Arc<Mutex<u32>>,
}
#[async_trait]
impl LlmProvider for CountProvider {
    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
    async fn generate(&self, _prompt: &str, _system: &str) -> AppResult<String> {
        *self.calls.lock().unwrap() += 1;
        Ok("pas du json".into())
    }
    async fn list_models(&self) -> AppResult<Vec<String>> {
        Ok(vec![])
    }
}

// CV source réaliste : sert de vérité pour l'ancrage et l'extraction des contacts.
const CV_SOURCE: &str =
    "Ada Lovelace — ada@x.io\nIngénieure chez ACME Corporation.\nCompétences : Rust, SQL.";

mod test_correction_prompt_contient_l_erreur_et_la_reponse_invalide;
mod test_extract_json_retire_les_fences_et_la_prose;
mod test_parse_llm_json_repare_les_cles_sans_guillemets;
mod test_parse_llm_json_repare_une_reponse_tronquee;

mod test_analyser_demande_lettre_extrait_la_structure;
mod test_analyser_entretien_json_valide_extrait_les_champs;
mod test_analyser_entretien_toujours_invalide_retourne_erreur;
mod test_analyze_ats_parse_le_recapitulatif;
mod test_analyze_ats_parse_le_score_et_suggestions;
mod test_analyze_ats_parse_les_recommandations_structurees;
mod test_extract_profile_advanced_conserve_les_valeurs_non_ancrees;
mod test_extract_profile_assemble_les_4_appels_specialises;
mod test_extract_profile_contacts_regex_priment_sur_le_llm;
mod test_extract_profile_reessaie_la_section_apres_json_invalide;
mod test_extract_profile_section_illisible_reste_vide_sans_echouer;
mod test_extract_profile_small_rejette_entreprise_et_competence_inventees;
mod test_generate_cv_parse_le_cv;
mod test_mode_small_limite_les_tentatives_a_deux;
mod test_mode_standard_autorise_trois_tentatives;
mod test_parse_cv_json_valide_structure_le_cv;
mod test_parse_cv_reessaie_apres_json_invalide;
mod test_parse_offer_json_valide_extrait_les_champs;
mod test_parse_offer_reessaie_apres_json_invalide;
mod test_parse_offer_toujours_invalide_retourne_erreur;
mod test_stream_cover_letter_accumule_et_renvoie_le_texte;
