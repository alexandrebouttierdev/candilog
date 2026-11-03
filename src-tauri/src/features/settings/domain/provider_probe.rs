//! Essai d'un fournisseur IA depuis l'écran des réglages.

use crate::core::errors::AppResult;
use crate::features::ai::domain::LlmConfig;
use async_trait::async_trait;

/// Interroge un fournisseur IA décrit par une configuration, sans la persister.
///
/// Le contrat vit ici et son implémentation dans `features::ai::infrastructure`
/// (`HttpProviderProbe`) : les réglages n'ont pas à construire un client HTTP de
/// fournisseur, et `docs/CODE_RULES.md` §5 veut qu'un service ne dépende que de contrats.
#[async_trait]
pub trait ProviderProbe: Send + Sync {
    /// Vérifie que le fournisseur répond avec cette configuration.
    async fn test(&self, config: &LlmConfig) -> AppResult<()>;

    /// Modèles exposés par le fournisseur.
    async fn list_models(&self, config: &LlmConfig) -> AppResult<Vec<String>>;
}
