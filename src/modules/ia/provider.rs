//! Abstraction d'un fournisseur LLM.

use crate::shared::error::{AppError, AppResult};
use async_trait::async_trait;

/// Options d'exécution d'une génération, adaptées à la capacité du modèle.
///
/// Portées **par appel** (et non par fournisseur) car elles varient selon l'étape
/// (budget de tokens différent pour l'identité et pour le parcours) et selon la
/// taille du fragment (le contexte est dimensionné au texte réellement envoyé).
/// Chaque champ est optionnel : `None` = « laisser le fournisseur décider ».
#[derive(Debug, Clone, Copy, Default)]
pub struct GenOptions {
    /// Fenêtre de contexte souhaitée (`num_ctx` Ollama). Ignoré par les providers cloud
    /// (le contexte y est géré côté serveur). Dimensionner évite la troncature silencieuse
    /// à 2048 tokens des modèles locaux.
    pub num_ctx: Option<u32>,
    /// Nombre maximal de tokens générés (`num_predict` Ollama, `max_tokens` cloud). Borne la
    /// génération d'un petit modèle qui aurait tendance à boucler ou à rallonger inutilement.
    pub num_predict: Option<u32>,
    /// Durée de maintien du modèle chargé en mémoire côté Ollama (ex. `"10m"`). Évite le
    /// rechargement des poids entre appels séquentiels — poste de latence dominant en local.
    pub keep_alive: Option<&'static str>,
}

impl GenOptions {
    /// Options neutres (aucune contrainte), pour les appels non dimensionnés.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }
}

/// Contrat commun à tous les fournisseurs LLM.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Vérifie que le service est joignable.
    ///
    /// # Errors
    /// Retourne `AppError::Provider` si le service est injoignable ou renvoie une erreur.
    async fn health_check(&self) -> AppResult<()>;

    /// Génère une réponse texte à partir d'un prompt utilisateur et d'un message système.
    ///
    /// # Errors
    /// Retourne `AppError::Provider` en cas d'échec réseau, `HTTP` ou de réponse illisible.
    async fn generate(&self, prompt: &str, system: &str) -> AppResult<String>;

    /// Génère une réponse contrainte par un schéma `JSON` (décodage guidé par grammaire).
    ///
    /// Le format de sortie est alors **garanti par le runtime** du fournisseur, pas
    /// seulement demandé par le prompt — décisif pour les petits modèles. Les
    /// fournisseurs sans support natif retombent sur [`generate`](Self::generate)
    /// (le prompt et la réparation `JSON` du moteur restent alors les seuls garde-fous).
    ///
    /// # Errors
    /// Retourne `AppError::Provider` en cas d'échec réseau, `HTTP` ou de réponse illisible.
    async fn generate_structured(
        &self,
        prompt: &str,
        system: &str,
        _schema: &serde_json::Value,
    ) -> AppResult<String> {
        self.generate(prompt, system).await
    }

    /// Génère une réponse en appliquant les [`GenOptions`] (budget de tokens, fenêtre de
    /// contexte, maintien en mémoire) et, si `schema` est fourni, la contrainte de format.
    ///
    /// **Point d'entrée du moteur** : `CvEngine` passe toujours par cette méthode. L'impl
    /// par défaut **ignore les options** et délègue à [`generate`](Self::generate) /
    /// [`generate_structured`](Self::generate_structured) — les fournisseurs qui tirent parti
    /// des options (Ollama, cloud pour `max_tokens`) la surchargent.
    ///
    /// # Errors
    /// Retourne `AppError::Provider` en cas d'échec réseau, `HTTP` ou de réponse illisible.
    async fn generate_with_options(
        &self,
        prompt: &str,
        system: &str,
        schema: Option<&serde_json::Value>,
        _options: &GenOptions,
    ) -> AppResult<String> {
        match schema {
            Some(s) => self.generate_structured(prompt, system, s).await,
            None => self.generate(prompt, system).await,
        }
    }

    /// Génère en **streaming texte brut** : appelle `on_chunk` pour chaque fragment reçu et renvoie
    /// le texte complet accumulé. Permet l'affichage progressif et l'annulation réelle (abandonner
    /// le futur coupe la connexion HTTP). Réservé à la génération créative (lettre) — pas de `JSON`.
    ///
    /// **Impl par défaut** : appelle [`generate_with_options`](Self::generate_with_options) puis
    /// émet le tout en un seul fragment. Un fournisseur sans streaming reste ainsi fonctionnel ;
    /// ceux qui le supportent (Ollama, cloud) la surchargent.
    ///
    /// # Errors
    /// `AppError::Provider`/`Http` en cas d'échec réseau ou de réponse illisible.
    async fn stream(
        &self,
        prompt: &str,
        system: &str,
        options: &GenOptions,
        on_chunk: &mut (dyn FnMut(String) + Send),
    ) -> AppResult<String> {
        let full = self
            .generate_with_options(prompt, system, None, options)
            .await?;
        on_chunk(full.clone());
        Ok(full)
    }

    /// Liste les modèles disponibles chez le fournisseur.
    ///
    /// # Errors
    /// Retourne `AppError::Provider` si le service est injoignable ou la réponse illisible.
    async fn list_models(&self) -> AppResult<Vec<String>>;
}

/// Convertit une erreur quelconque en `AppError::Provider` (helper des implémentations).
pub(crate) trait IntoProviderError {
    fn into_provider_error(self) -> AppError;
}

impl IntoProviderError for reqwest::Error {
    fn into_provider_error(self) -> AppError {
        AppError::from(self)
    }
}

impl IntoProviderError for AppError {
    fn into_provider_error(self) -> AppError {
        self
    }
}

impl IntoProviderError for &str {
    fn into_provider_error(self) -> AppError {
        AppError::Provider(self.to_string())
    }
}

impl IntoProviderError for String {
    fn into_provider_error(self) -> AppError {
        AppError::Provider(self)
    }
}

pub(crate) fn provider_err(e: impl IntoProviderError) -> AppError {
    match e.into_provider_error() {
        AppError::Http(message) => AppError::Provider(message),
        other => other,
    }
}

/// Extrait la charge utile `JSON` d'une ligne `SSE` `data: …`.
///
/// Renvoie `None` pour les lignes non-`data` (commentaires, `event:`, lignes vides) et pour le
/// sentinel de fin `[DONE]`. Partagé par les fournisseurs cloud qui streament en `SSE`.
pub(crate) fn sse_data(line: &str) -> Option<&str> {
    let data = line.strip_prefix("data:")?.trim();
    if data.is_empty() || data == "[DONE]" {
        None
    } else {
        Some(data)
    }
}

#[cfg(test)]
#[path = "tests/provider/mod.rs"]
mod tests;
