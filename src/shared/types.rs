//! Types transverses partagés par les modules.

use serde::{Deserialize, Serialize};

/// Analyse `IA` du compte rendu d'un entretien.
///
/// Produite par le `LLM` à partir du compte rendu rédigé par l'utilisateur, puis
/// persistée en `jsonb` sur l'entretien. Vit dans `shared` pour que les modules
/// `ia` et `entretiens` la connaissent tous deux sans jamais s'importer l'un l'autre.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AnalyseEntretien {
    /// Résumé synthétique de l'entretien.
    pub resume: String,
    /// Points forts relevés dans le compte rendu.
    pub points_forts: Vec<String>,
    /// Points faibles relevés dans le compte rendu.
    pub points_faibles: Vec<String>,
    /// Suggestions d'amélioration pour les prochains entretiens.
    pub suggestions: Vec<String>,
}
