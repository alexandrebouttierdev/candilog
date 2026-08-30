//! Type d'erreur unifié de l'application et son contrat IPC.

use serde::Serialize;

/// Error unifiée remontée par toutes les couches de l'application.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Entrée invalide (validation métier).
    #[error("Validation : {0}")]
    Validation(String),
    /// Ressource introuvable.
    #[error("Introuvable : {0}")]
    NotFound(String),
    /// Error d'accès à la base locale (`SQLite`).
    #[error("Base de données : {0}")]
    Database(String),
    /// Base ou sauvegarde créée par une version de données qui n'est plus prise en charge.
    #[error("Données incompatibles : {0}")]
    IncompatibleData(String),
    /// Error réseau / HTTP (fournisseurs `LLM`).
    #[error("HTTP : {0}")]
    Http(String),
    /// Error de (dé)sérialisation JSON.
    #[error("Sérialisation : {0}")]
    Serialization(String),
    /// Error remontée par un fournisseur IA.
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
            Self::IncompatibleData(_) => "INCOMPATIBLE_DATA",
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
    /// Les variantes dont le message est *déjà* rédigé pour l'utilisateur — validation métier
    /// et refus d'un fournisseur — sont reprises telles quelles. Le détail HTTP est toujours
    /// masqué car une erreur `reqwest` peut inclure l'URL et ses paramètres sensibles.
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation(message) | Self::Provider(message) => message.clone(),
            Self::NotFound(resource) => format!("Introuvable : {resource}."),
            Self::Database(_) => {
                "Le fichier de données de Candilog est illisible ou endommagé.".into()
            }
            Self::IncompatibleData(_) => "Cette base a été créée par une ancienne version de \
                Candilog. Déplacez-la ou supprimez-la avant de relancer l'application."
                .into(),
            Self::Http(_) => {
                "La connexion au service distant a échoué. Vérifiez votre réseau et réessayez."
                    .into()
            }
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
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "app-error.ts")]
pub struct AppErrorDto {
    /// Code stable, cf. [`AppError::code`].
    pub code: String,
    /// Message rédigé pour l'utilisateur, cf. [`AppError::user_message`].
    pub message: String,
}

impl From<&AppError> for AppErrorDto {
    fn from(error: &AppError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.user_message(),
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
        if matches!(self, Self::Http(_)) {
            tracing::error!(code = self.code(), "erreur HTTP remontée à l'interface");
        } else {
            tracing::error!(code = self.code(), detail = %self, "erreur remontée à l'interface");
        }
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
        let is_timeout = error.is_timeout();
        let is_connect = error.is_connect();
        let status = error.status().map(|value| value.as_u16());
        tracing::warn!(is_timeout, is_connect, ?status, "requête HTTP échouée");
        if is_timeout {
            Self::Http("délai dépassé".into())
        } else if is_connect {
            Self::Http("connexion impossible".into())
        } else {
            Self::Http("requête distante échouée".into())
        }
    }
}

#[cfg(test)]
#[path = "tests/app_error/mod.rs"]
mod tests;
