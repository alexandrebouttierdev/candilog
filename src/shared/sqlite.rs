//! Helpers partagés par les repositories `SQLite`.
//!
//! Regroupe ce que les huit modules métier feraient sinon à l'identique : obtention d'une
//! connexion du pool, horodatage, conversion des colonnes `TEXT` en `Uuid` et en enums serde,
//! et traduction des erreurs `rusqlite` en erreurs applicatives.

use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use r2d2_sqlite::SqliteConnectionManager;
use serde::{de::DeserializeOwned, Serialize};

/// Connexion empruntée au pool.
pub type Connexion = r2d2::PooledConnection<SqliteConnectionManager>;

/// Emprunte une connexion au pool.
///
/// # Errors
/// Retourne `AppError::Database` si le pool est épuisé ou fermé.
pub fn connexion(pool: &SqlitePool) -> AppResult<Connexion> {
    pool.get().map_err(|e| AppError::Database(e.to_string()))
}

/// Horodatage courant au format `RFC 3339` (`UTC`), utilisé pour `created_at` / `updated_at`.
#[must_use]
pub fn maintenant_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Lit une colonne `TEXT` et la convertit en `Uuid`.
///
/// # Errors
/// Retourne une erreur `rusqlite` de conversion si la colonne n'est pas un `UUID` valide.
pub fn uuid_colonne(row: &rusqlite::Row, index: usize) -> rusqlite::Result<uuid::Uuid> {
    let texte: String = row.get(index)?;
    uuid::Uuid::parse_str(&texte).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Variante de [`uuid_colonne`] pour les colonnes nullables.
///
/// # Errors
/// Retourne une erreur `rusqlite` de conversion si la colonne n'est ni nulle ni un `UUID` valide.
pub fn uuid_colonne_opt(row: &rusqlite::Row, index: usize) -> rusqlite::Result<Option<uuid::Uuid>> {
    let texte: Option<String> = row.get(index)?;
    match texte {
        None => Ok(None),
        Some(t) => uuid::Uuid::parse_str(&t).map(Some).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                index,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        }),
    }
}

/// Convertit une valeur `TEXT` en enum serde (`type_contrat`, `statut`, `type` d'entretien).
///
/// # Errors
/// Retourne `AppError::Serialization` si la valeur ne correspond à aucune variante.
pub fn enum_depuis_texte<T: DeserializeOwned>(texte: &str) -> AppResult<T> {
    serde_json::from_value(serde_json::Value::String(texte.to_owned()))
        .map_err(|e| AppError::Serialization(format!("valeur « {texte} » invalide : {e}")))
}

/// Convertit un enum serde en `TEXT` stockable.
///
/// # Errors
/// Retourne `AppError::Serialization` si la valeur ne se sérialise pas en chaîne.
pub fn texte_depuis_enum<T: Serialize>(valeur: &T) -> AppResult<String> {
    match serde_json::to_value(valeur).map_err(|e| AppError::Serialization(e.to_string()))? {
        serde_json::Value::String(s) => Ok(s),
        autre => Err(AppError::Serialization(format!(
            "valeur non textuelle : {autre}"
        ))),
    }
}

/// Traduit une erreur `rusqlite` là où une violation de contrainte est réellement attendue.
///
/// [`traduire_erreur`] fait porter au même paramètre deux rôles incompatibles : l'étiquette de
/// ressource pour `NotFound` (« candidature », « contact ») et le message lu par l'utilisateur
/// pour `Validation`. Résultat, un contexte choisi pour `NotFound` produisait des messages comme
/// « Validation : version de CV ». Cette fonction sépare les deux : `message_utilisateur` est une
/// phrase complète, `label_ressource` sert aux autres cas.
///
/// À employer aux points où une contrainte peut être violée (clé étrangère, unicité) ;
/// [`traduire_erreur`] reste la fonction par défaut partout ailleurs.
#[must_use]
pub fn traduire_contrainte(
    erreur: rusqlite::Error,
    message_utilisateur: &str,
    label_ressource: &str,
) -> AppError {
    match erreur {
        rusqlite::Error::SqliteFailure(ref e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(message_utilisateur.to_owned())
        }
        autre => traduire_erreur(autre, label_ressource),
    }
}

/// Traduit une erreur `rusqlite` en erreur applicative.
///
/// Les violations de contrainte deviennent des erreurs de validation porteuses de `contexte`,
/// l'absence de ligne devient `NotFound`, le reste reste une erreur de base de données. Quand une
/// violation de contrainte est attendue et mérite une phrase lisible, préférer
/// [`traduire_contrainte`].
#[must_use]
pub fn traduire_erreur(erreur: rusqlite::Error, contexte: &str) -> AppError {
    match erreur {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(contexte.to_owned()),
        rusqlite::Error::SqliteFailure(ref e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(contexte.to_owned())
        }
        autre => AppError::Database(autre.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/sqlite/mod.rs"]
mod tests;
