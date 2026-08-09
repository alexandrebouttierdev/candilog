//! DTO d'analyse d'une offre.

/// Texte d'offre soumis au moteur IA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeOfferDto {
    /// Contenu intégral de l'offre.
    pub offer: String,
}
