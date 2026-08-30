//! Dépôt `SQLite` des référentiels métier.

use crate::core::database::helpers::{connection, translate_error, uuid_column};
use crate::core::database::SqlitePool;
use crate::core::errors::AppResult;
use crate::features::referentials::domain::{
    ActivitySector, ReferenceItem, ReferentialRepository, Referentials,
};

/// Implémentation `SQLite` des quatre catalogues.
pub struct SqliteReferentialRepository {
    pool: SqlitePool,
}

impl SqliteReferentialRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Lit un référentiel à clé métier, dans l'ordre d'affichage.
///
/// `sort_order` d'abord, le libellé ensuite : deux entrées de même rang restent alors dans
/// un ordre stable d'une requête à l'autre, ce qu'un sélecteur qui se réordonne tout seul
/// rendrait déroutant.
fn read_items(
    conn: &rusqlite::Connection,
    table: &str,
    label: &str,
) -> AppResult<Vec<ReferenceItem>> {
    // `table` ne vient pas de l'IPC : les trois valeurs possibles sont écrites ci-dessous,
    // aucune saisie utilisateur n'atteint cette interpolation.
    let mut query = conn
        .prepare(&format!(
            "SELECT code, name FROM {table} ORDER BY sort_order ASC, name COLLATE NOCASE ASC"
        ))
        .map_err(|error| translate_error(error, label))?;
    let rows = query
        .query_map([], |row| {
            Ok(ReferenceItem {
                code: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|error| translate_error(error, label))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|error| translate_error(error, label))?);
    }
    Ok(items)
}

impl ReferentialRepository for SqliteReferentialRepository {
    fn load(&self) -> AppResult<Referentials> {
        let conn = connection(&self.pool)?;

        let sectors = {
            let mut query = conn
                .prepare(
                    "SELECT id, name FROM sectors ORDER BY sort_order ASC, name COLLATE NOCASE ASC",
                )
                .map_err(|error| translate_error(error, "secteurs d'activité"))?;
            let rows = query
                .query_map([], |row| {
                    Ok(ActivitySector {
                        id: uuid_column(row, 0)?,
                        name: row.get(1)?,
                    })
                })
                .map_err(|error| translate_error(error, "secteurs d'activité"))?;
            let mut items = Vec::new();
            for row in rows {
                items.push(row.map_err(|error| translate_error(error, "secteurs d'activité"))?);
            }
            items
        };

        Ok(Referentials {
            sectors,
            professional_domains: read_items(
                &conn,
                "professional_domains",
                "domaines professionnels",
            )?,
            company_types: read_items(&conn, "company_types", "types d'entreprise")?,
            contract_types: read_items(&conn, "contract_types", "types de contrat")?,
        })
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
