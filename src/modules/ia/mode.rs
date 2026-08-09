//! Profils d'exécution du pipeline `IA` par mode d'analyse.
//!
//! Un [`ModeProfile`] traduit un [`AnalysisMode`] en réglages concrets appliqués par
//! [`CvEngine`](crate::modules::ia::cv_engine::CvEngine) : budgets de tokens par étape,
//! fenêtre de contexte, nombre de tentatives, validation anti-hallucination, parallélisme.
//!
//! Objectif : un petit modèle (≈ 1B) reçoit un contexte réduit, une tâche par requête, des
//! sorties bornées et une validation forte ; un gros modèle / cloud n'est pas bridé.

use crate::modules::ia::provider::GenOptions;
use crate::shared::llm::AnalysisMode;

/// Étape du pipeline `IA` — sert à choisir un budget de tokens de sortie adapté.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Extraction de l'identité (nom, headline, résumé) — sortie courte.
    Identity,
    /// Extraction du parcours (expériences + formations) — sortie potentiellement longue.
    History,
    /// Extraction des compétences et langues — sortie moyenne.
    Skills,
    /// Extraction des projets et certifications — sortie moyenne.
    Portfolio,
    /// Analyse d'une offre — sortie moyenne.
    Offer,
    /// Structuration d'un CV importé — sortie longue.
    ParseCv,
    /// Analyse `ATS` (recap + suggestions + recommandations) — sortie longue.
    Ats,
    /// Génération d'un CV reformulé — sortie longue.
    Generate,
    /// Analyse d'un compte rendu d'entretien — sortie moyenne.
    Interview,
    /// Génération d'une lettre de motivation (250–400 mots) — sortie longue.
    CoverLetter,
    /// Analyse d'une demande libre de lettre (extraction structurée) — sortie courte.
    LetterIntent,
}

/// Réglages d'exécution dérivés d'un [`AnalysisMode`].
///
/// Struct de drapeaux de configuration : les booléens sont indépendants et lisibles nommés,
/// un enum combiné n'apporterait rien.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct ModeProfile {
    /// Mode résolu à l'origine du profil (jamais `Auto`).
    pub mode: AnalysisMode,
    /// Si vrai, dimensionne `num_ctx` au fragment (Ollama). Faux pour les gros modèles / cloud
    /// (contexte géré côté serveur, aucune contrainte envoyée).
    pub size_context: bool,
    /// Si vrai, borne la sortie de chaque étape (`num_predict`/`max_tokens`). Faux en mode
    /// avancé (le modèle décide, aucune troncature de génération).
    pub cap_output: bool,
    /// Nombre maximal de tentatives de parsing `JSON` par opération.
    pub max_attempts: u32,
    /// Si vrai, applique la validation d'ancrage anti-hallucination après extraction.
    pub grounding: bool,
    /// Si vrai, lance les appels d'extraction de sections en parallèle (gros modèle / cloud).
    /// Faux en local : les petits modèles traitent une génération à la fois (contention mémoire).
    pub parallel_sections: bool,
    /// Maintien du modèle chargé côté Ollama entre appels (`None` = défaut du serveur).
    pub keep_alive: Option<&'static str>,
}

impl ModeProfile {
    /// Construit le profil correspondant au mode (déjà résolu, jamais `Auto`).
    #[must_use]
    pub fn for_mode(mode: AnalysisMode) -> Self {
        match mode {
            // Petit modèle local : tout est resserré et validé.
            AnalysisMode::Small => Self {
                mode: AnalysisMode::Small,
                size_context: true,
                cap_output: true,
                max_attempts: 2,
                grounding: true,
                parallel_sections: false,
                keep_alive: Some("10m"),
            },
            // Modèle intermédiaire : comportement historique, équilibré.
            AnalysisMode::Auto | AnalysisMode::Standard => Self {
                mode: AnalysisMode::Standard,
                size_context: true,
                cap_output: true,
                max_attempts: 3,
                grounding: true,
                parallel_sections: false,
                keep_alive: Some("10m"),
            },
            // Gros modèle local / cloud : aucune contrainte de contexte ni de sortie, parallèle,
            // confiance accrue (pas de rejet d'ancrage sur des reformulations légitimes).
            AnalysisMode::Advanced => Self {
                mode: AnalysisMode::Advanced,
                size_context: false,
                cap_output: false,
                max_attempts: 2,
                grounding: false,
                parallel_sections: true,
                keep_alive: None,
            },
        }
    }

    /// Budget de tokens de sortie d'une étape, ou `None` si non borné (mode avancé).
    ///
    /// Les valeurs bornent la génération d'un petit modèle (qui aurait tendance à rallonger
    /// ou boucler) sans couper un contenu légitime : une section de CV tient largement dans
    /// ces limites. Le mode `Standard` double le budget par sécurité.
    #[must_use]
    pub fn output_tokens(&self, step: Step) -> Option<u32> {
        if !self.cap_output {
            return None;
        }
        let small = matches!(self.mode, AnalysisMode::Small);
        let base = match step {
            Step::LetterIntent => 256,
            Step::Identity => 384,
            Step::Skills | Step::Portfolio | Step::Offer | Step::Interview => 512,
            Step::History | Step::Ats | Step::CoverLetter => 1024,
            Step::ParseCv | Step::Generate => 1536,
        };
        Some(if small { base } else { base * 2 })
    }

    /// Options d'exécution d'une étape : `num_ctx` dimensionné au fragment (si activé),
    /// budget de sortie, et `keep_alive`.
    ///
    /// `input_chars` = taille du texte réellement envoyé (fragment ciblé). `num_ctx` est
    /// arrondi au palier supérieur pour couvrir prompt + entrée + sortie, ce qui **évite la
    /// troncature silencieuse à 2048 tokens** des modèles locaux sans sur-allouer inutilement.
    #[must_use]
    #[allow(clippy::unnecessary_lazy_evaluations)]
    pub fn gen_options(&self, step: Step, input_chars: usize) -> GenOptions {
        let num_predict = self.output_tokens(step);
        let num_ctx = if self.size_context {
            Some(size_num_ctx(
                input_chars,
                num_predict.unwrap_or_else(|| 1024),
            ))
        } else {
            None
        };
        GenOptions {
            num_ctx,
            num_predict,
            keep_alive: self.keep_alive,
        }
    }
}

/// Dimensionne `num_ctx` au palier supérieur couvrant l'entrée et la sortie attendue.
///
/// Estimation prudente : ≈ 1 token pour 3 caractères (français/anglais), plus une marge pour
/// le prompt système et le format. Paliers `2048/4096/8192/16384` : on prend le plus petit qui
/// suffit. Bornes : jamais moins de 2048 (défaut Ollama), jamais plus de 16384.
#[allow(clippy::unnecessary_lazy_evaluations)]
fn size_num_ctx(input_chars: usize, output_tokens: u32) -> u32 {
    let input_tokens = (input_chars / 3) + 512; // marge prompt système + format
    let needed = input_tokens as u64 + u64::from(output_tokens);
    [2048_u32, 4096, 8192, 16384]
        .into_iter()
        .find(|bucket| u64::from(*bucket) >= needed)
        .unwrap_or_else(|| 16384)
}

#[cfg(test)]
#[path = "tests/mode/mod.rs"]
mod tests;
