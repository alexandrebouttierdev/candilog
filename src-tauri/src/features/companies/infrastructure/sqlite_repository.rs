//! Dépôt `SQLite` des entreprises.

use crate::core::database::helpers::{
    connection, like_contains, now_iso, translate_constraint, translate_error, uuid_column,
    uuid_column_opt, LIKE_ESCAPE,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{clamp_page_size, Page};
use crate::features::companies::domain::{Company, CompanyRepository, NewCompany};

/// Implémentation `SQLite` du dépôt d'entreprises.
pub struct SqliteCompanyRepository {
    pool: SqlitePool,
}

impl SqliteCompanyRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Columns lues par [`row_vers_company`], dans l'ordre.
const COLUMNS: &str = "id, name, sector_id, sector, type, website, city, address, notes, \
                        created_at, updated_at";

/// Convertit une ligne `SQLite` en entreprise du domaine.
fn row_vers_company(row: &rusqlite::Row) -> rusqlite::Result<Company> {
    Ok(Company {
        id: uuid_column(row, 0)?,
        name: row.get(1)?,
        sector_id: uuid_column_opt(row, 2)?,
        sector: row.get(3)?,
        type_: row.get(4)?,
        website: row.get(5)?,
        city: row.get(6)?,
        address: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

impl CompanyRepository for SqliteCompanyRepository {
    fn list(&self) -> AppResult<Vec<Company>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM companies ORDER BY name COLLATE NOCASE ASC"
            ))
            .map_err(|e| translate_error(e, "entreprises"))?;
        let rows = query
            .query_map([], row_vers_company)
            .map_err(|e| translate_error(e, "entreprises"))?;
        let mut companies = Vec::new();
        for row in rows {
            companies.push(row.map_err(|e| translate_error(e, "entreprises"))?);
        }
        Ok(companies)
    }

    fn get(&self, id: uuid::Uuid) -> AppResult<Company> {
        let conn = connection(&self.pool)?;
        conn.query_row(
            &format!("SELECT {COLUMNS} FROM companies WHERE id = ?1"),
            [id.to_string()],
            row_vers_company,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("entreprise {id}")),
            other => translate_error(other, "entreprise"),
        })
    }

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        company_type: Option<&str>,
    ) -> AppResult<Page<Company>> {
        let conn = connection(&self.pool)?;
        let page = page.max(1);
        let page_size = clamp_page_size(page_size);
        let needle = like_contains(search);
        let selected_type = company_type.unwrap_or_default().trim().to_lowercase();
        let filter = format!(
            "WHERE (?1 = '%%' OR lower(name) LIKE ?1 {LIKE_ESCAPE} OR lower(coalesce(sector, '')) LIKE ?1 {LIKE_ESCAPE} OR lower(coalesce(city, '')) LIKE ?1 {LIKE_ESCAPE}) AND (?2 = '' OR lower(trim(coalesce(type, ''))) = ?2)"
        );
        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) FROM companies {filter}"),
                rusqlite::params![needle, selected_type],
                |row| row.get(0),
            )
            .map_err(|e| translate_error(e, "entreprises"))?;
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let mut query = conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM companies {filter} ORDER BY name COLLATE NOCASE ASC LIMIT ?3 OFFSET ?4"
            ))
            .map_err(|e| translate_error(e, "entreprises"))?;
        let rows = query
            .query_map(
                rusqlite::params![needle, selected_type, page_size, offset],
                row_vers_company,
            )
            .map_err(|e| translate_error(e, "entreprises"))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| translate_error(e, "entreprises"))?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn list_types(&self) -> AppResult<Vec<String>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(
                "SELECT trim(type) FROM companies WHERE trim(coalesce(type, '')) <> '' GROUP BY lower(trim(type)) ORDER BY lower(trim(type)) ASC",
            )
            .map_err(|e| translate_error(e, "types d'entreprise"))?;
        let rows = query
            .query_map([], |row| row.get(0))
            .map_err(|e| translate_error(e, "types d'entreprise"))?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(|e| translate_error(e, "types d'entreprise"))
    }

    fn create(&self, input: &NewCompany) -> AppResult<Company> {
        let conn = connection(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let now = now_iso();
        conn.execute(
            "INSERT INTO companies (id, name, sector_id, sector, type, website, city, address, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            rusqlite::params![
                id.to_string(),
                input.name,
                input.sector_id.map(|value| value.to_string()),
                input.sector,
                input.type_,
                input.website,
                input.city,
                input.address,
                input.notes,
                now
            ],
        )
        .map_err(|e| translate_error(e, "entreprise invalide"))?;
        conn.query_row(
            &format!("SELECT {COLUMNS} FROM companies WHERE id = ?1"),
            [id.to_string()],
            row_vers_company,
        )
        .map_err(|e| translate_error(e, "entreprise"))
    }

    fn update(&self, id: uuid::Uuid, input: &NewCompany) -> AppResult<Company> {
        let conn = connection(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE companies SET name = ?2, sector_id = ?3, sector = ?4, type = ?5,
                    website = ?6, city = ?7, address = ?8, notes = ?9, updated_at = ?10
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.name,
                    input.sector_id.map(|value| value.to_string()),
                    input.sector,
                    input.type_,
                    input.website,
                    input.city,
                    input.address,
                    input.notes,
                    now_iso()
                ],
            )
            .map_err(|e| translate_error(e, "entreprise invalide"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entreprise {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLUMNS} FROM companies WHERE id = ?1"),
            [id.to_string()],
            row_vers_company,
        )
        .map_err(|e| translate_error(e, "entreprise"))
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        conn.execute("DELETE FROM companies WHERE id = ?1", [id.to_string()])
            .map_err(|e| {
                translate_constraint(
                    e,
                    "Suppression impossible : des applications sont liées à cette entreprise",
                    "entreprise",
                )
            })?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
