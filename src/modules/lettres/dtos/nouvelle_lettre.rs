//! Données nécessaires à l'enregistrement d'une lettre.

/// Brouillon validé avant persistance dans la bibliothèque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NouvelleLettre {
    pub name: String,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub tone: String,
    pub length: String,
    pub content: String,
}
