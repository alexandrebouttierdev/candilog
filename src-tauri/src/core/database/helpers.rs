//! Helpers partagés par les repositories `SQLite`.
//!
//! Regroupe ce que les huit modules métier feraient sinon à l'identique : obtention d'une
//! connexion du pool, horodatage, conversion des colonnes `TEXT` en `Uuid` et en enums serde,
//! et traduction des erreurs `rusqlite` en erreurs applicatives.

use crate::core::database::connection::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use r2d2_sqlite::SqliteConnectionManager;
use serde::{de::DeserializeOwned, Serialize};

/// Connection empruntée au pool.
pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Emprunte une connexion au pool.
///
/// # Errors
/// Retourne `AppError::Database` si le pool est épuisé ou fermé.
pub fn connection(pool: &SqlitePool) -> AppResult<Connection> {
    pool.get().map_err(|e| AppError::Database(e.to_string()))
}

/// Timestamp courant au format `RFC 3339` (`UTC`), utilisé pour `created_at` / `updated_at`.
#[must_use]
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Lit une colonne `TEXT` et la convertit en `Uuid`.
///
/// # Errors
/// Retourne une erreur `rusqlite` de conversion si la colonne n'est pas un `UUID` valide.
pub fn uuid_column(row: &rusqlite::Row, index: usize) -> rusqlite::Result<uuid::Uuid> {
    let text: String = row.get(index)?;
    uuid::Uuid::parse_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Variante de [`uuid_column`] pour les colonnes nullables.
///
/// # Errors
/// Retourne une erreur `rusqlite` de conversion si la colonne n'est ni nulle ni un `UUID` valide.
pub fn uuid_column_opt(row: &rusqlite::Row, index: usize) -> rusqlite::Result<Option<uuid::Uuid>> {
    let text: Option<String> = row.get(index)?;
    match text {
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

/// Convertit une valeur `TEXT` en enum serde (`contract_type`, `statut`, `type` d'entretien).
///
/// # Errors
/// Retourne `AppError::Serialization` si la valeur ne correspond à aucune variante.
pub fn enum_from_text<T: DeserializeOwned>(text: &str) -> AppResult<T> {
    serde_json::from_value(serde_json::Value::String(text.to_owned()))
        .map_err(|e| AppError::Serialization(format!("valeur « {text} » invalide : {e}")))
}

/// Convertit un enum serde en `TEXT` stockable.
///
/// # Errors
/// Retourne `AppError::Serialization` si la valeur ne se sérialise pas en chaîne.
pub fn text_from_enum<T: Serialize>(value: &T) -> AppResult<String> {
    match serde_json::to_value(value).map_err(|e| AppError::Serialization(e.to_string()))? {
        serde_json::Value::String(s) => Ok(s),
        other => Err(AppError::Serialization(format!(
            "valeur non textuelle : {other}"
        ))),
    }
}

/// Traduit une erreur `rusqlite` là où une violation de contrainte est réellement attendue.
///
/// [`translate_error`] fait porter au même paramètre deux rôles incompatibles : l'étiquette de
/// ressource pour `NotFound` (« candidature », « contact ») et le message lu par l'utilisateur
/// pour `Validation`. Résultat, un contexte choisi pour `NotFound` produisait des messages comme
/// « Validation : version de CV ». Cette fonction sépare les deux : `user_message` est une
/// phrase complète, `label_ressource` sert aux autres cas.
///
/// À employer aux points où une contrainte peut être violée (clé étrangère, unicité) ;
/// [`translate_error`] reste la fonction par défaut partout ailleurs.
#[must_use]
pub fn translate_constraint(
    error: rusqlite::Error,
    user_message: &str,
    label_ressource: &str,
) -> AppError {
    match error {
        rusqlite::Error::SqliteFailure(ref e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(user_message.to_owned())
        }
        other => translate_error(other, label_ressource),
    }
}

/// Traduit une erreur `rusqlite` en erreur applicative.
///
/// Les violations de contrainte deviennent des erreurs de validation porteuses de `contexte`,
/// l'absence de ligne devient `NotFound`, le reste reste une erreur de base de données. Quand une
/// violation de contrainte est attendue et mérite une phrase lisible, préférer
/// [`translate_constraint`].
#[must_use]
pub fn translate_error(error: rusqlite::Error, context: &str) -> AppError {
    match error {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(context.to_owned()),
        rusqlite::Error::SqliteFailure(ref e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(context.to_owned())
        }
        other => AppError::Database(other.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/helpers/mod.rs"]
mod tests;
