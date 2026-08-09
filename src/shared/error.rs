//! Type d'erreur unifié de l'application.

/// Erreur unifiée remontée par toutes les couches de l'application.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Entrée invalide (validation métier).
    #[error("Validation : {0}")]
    Validation(String),
    /// Ressource introuvable.
    #[error("Introuvable : {0}")]
    NotFound(String),
    /// Erreur d'accès à la base locale (`SQLite`).
    #[error("Base de données : {0}")]
    Database(String),
    /// Erreur réseau / HTTP (fournisseurs `LLM`).
    #[error("HTTP : {0}")]
    Http(String),
    /// Erreur de (dé)sérialisation JSON.
    #[error("Sérialisation : {0}")]
    Serialization(String),
    /// Erreur remontée par un fournisseur IA.
    #[error("Fournisseur IA : {0}")]
    Provider(String),
    /// Opération interrompue à la demande de l'utilisateur (annulation).
    #[error("Génération annulée")]
    Cancelled,
}

/// Alias de résultat standard de l'application.
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Convertit l'erreur en message destiné au frontend (contrat Tauri `Result<T, String>`).
    #[must_use]
    pub fn to_command_error(&self) -> String {
        self.to_string()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        let message = if e.is_timeout() {
            "délai réseau dépassé".to_string()
        } else if e.is_connect() {
            "connexion au service distant impossible".to_string()
        } else if let Some(status) = e.status() {
            format!("le service distant a répondu {status}")
        } else {
            "requête réseau impossible".to_string()
        };
        Self::Http(message)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

#[cfg(test)]
#[path = "tests/error/mod.rs"]
mod tests;
