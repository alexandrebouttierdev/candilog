//! Type d'erreur unifié de l'application et son contrat IPC.

use serde::Serialize;

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
    /// Code stable identifiant la nature de l'erreur, destiné au frontend.
    ///
    /// Le message est rédigé pour l'utilisateur et peut être reformulé sans préavis ; le code
    /// est le seul élément sur lequel React peut brancher un comportement (rediriger vers les
    /// réglages sur `PROVIDER_ERROR`, proposer un rechargement sur `NOT_FOUND`, ignorer
    /// silencieusement un `CANCELLED`). Le comparer à une sous-chaîne du message rendrait
    /// l'interface dépendante d'une formulation.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Http(_) => "HTTP_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Provider(_) => "PROVIDER_ERROR",
            Self::Cancelled => "CANCELLED",
        }
    }

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

/// Forme sous laquelle une erreur traverse l'IPC Tauri.
///
/// Tauri sérialise le `Err` d'une commande et le rejette côté JavaScript. Sans conversion
/// explicite, la valeur rejetée serait la représentation `Debug` de l'énumération : le
/// frontend n'aurait ni code exploitable ni phrase présentable, et le détail technique
/// (chemins locaux, requêtes SQL) franchirait la frontière.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "app-error.ts")]
pub struct AppErrorDto {
    /// Code stable, cf. [`AppError::code`].
    pub code: String,
    /// Message rédigé pour l'utilisateur, cf. [`AppError::message_utilisateur`].
    pub message: String,
}

impl From<&AppError> for AppErrorDto {
    fn from(error: &AppError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.message_utilisateur(),
        }
    }
}

impl serde::Serialize for AppError {
    /// Journalise le détail technique **avant** de ne transmettre que le contrat public.
    ///
    /// C'est le seul point de passage garanti de toutes les erreurs remontées à l'interface :
    /// y placer la trace évite d'avoir à la répéter dans chaque commande, et garantit qu'aucune
    /// erreur n'est reformulée sans laisser d'empreinte exploitable dans le journal.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        tracing::error!(code = self.code(), detail = %self, "erreur remontée à l'interface");
        AppErrorDto::from(self).serialize(serializer)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Http("le fournisseur IA met trop de temps à répondre".into())
        } else if error.is_connect() {
            Self::Http("le fournisseur IA est injoignable".into())
        } else {
            Self::Http(error.to_string())
        }
    }
}

#[cfg(test)]
#[path = "tests/app_error/mod.rs"]
mod tests;
