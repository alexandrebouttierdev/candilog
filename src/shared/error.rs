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
    /// Message destiné à l'**utilisateur final**, distinct du `Display` destiné au journal.
    ///
    /// `Display` porte le détail technique — chaîne brute de rusqlite ou de r2d2, message de
    /// serde, chemin absolu du fichier de données. Ce détail est indispensable au diagnostic
    /// (il part au journal, cf. `tracing`) mais incompréhensible à l'écran, et il divulgue
    /// l'arborescence locale dans la moindre capture d'écran de support.
    ///
    /// Les variantes dont le message est *déjà* rédigé pour l'utilisateur — validation métier,
    /// erreurs réseau reformulées par `From<reqwest::Error>`, refus d'un fournisseur — sont
    /// reprises telles quelles : les masquer priverait de toute indication utile.
    #[must_use]
    pub fn message_utilisateur(&self) -> String {
        match self {
            Self::Validation(message) | Self::Provider(message) => message.clone(),
            Self::NotFound(quoi) => format!("Introuvable : {quoi}."),
            Self::Database(_) => {
                "Le fichier de données de Candilog est illisible ou endommagé.".into()
            }
            Self::Http(message) => format!("Problème réseau : {message}."),
            Self::Serialization(_) => {
                "Une donnée enregistrée est illisible : son format n'est pas reconnu.".into()
            }
            Self::Cancelled => "Génération annulée.".into(),
        }
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
