//! Dépôt `SQLite` des entreprises.

use crate::core::database::helpers::{
    connection, enum_from_text, like_contains, now_iso, text_from_enum, translate_constraint,
    translate_error, uuid_column, uuid_column_opt, LIKE_ESCAPE,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{clamp_page_size, Page};
use crate::features::companies::domain::{Company, CompanyFilter, CompanyRepository, NewCompany};
use rusqlite::types::Value;

/// Implémentation `SQLite` du dépôt d'entreprises.
pub struct SqliteCompanyRepository {
    pool: SqlitePool,
}

impl SqliteCompanyRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`row_to_company`], dans l'ordre.
const COLUMNS: &str = "e.id, e.name, e.sector_id, s.name, e.company_type_id, t.name, \
                        e.company_size, e.website, e.city, e.address, e.notes, \
                        e.created_at, e.updated_at";

/// Source des colonnes : les libellés des référentiels sont résolus par jointure.
///
/// Une jointure et non une seconde colonne de libellé : le nom du secteur n'existe qu'à un
/// seul endroit, et renommer une entrée du référentiel se voit partout du même coup.
const FROM_SQL: &str = "FROM companies e \
                        LEFT JOIN sectors s ON s.id = e.sector_id \
                        LEFT JOIN company_types t ON t.code = e.company_type_id";

/// Convertit une ligne `SQLite` en entreprise du domaine.
fn row_to_company(row: &rusqlite::Row) -> AppResult<Company> {
    let read = |index: usize| -> AppResult<String> {
        row.get(index).map_err(|e| translate_error(e, "entreprise"))
    };
    let company_size = read(6)?;
    Ok(Company {
        id: uuid_column(row, 0).map_err(|e| translate_error(e, "entreprise"))?,
        name: read(1)?,
        sector_id: uuid_column_opt(row, 2).map_err(|e| translate_error(e, "entreprise"))?,
        sector_name: row.get(3).map_err(|e| translate_error(e, "entreprise"))?,
        company_type_id: row.get(4).map_err(|e| translate_error(e, "entreprise"))?,
        company_type_name: row.get(5).map_err(|e| translate_error(e, "entreprise"))?,
        company_size: enum_from_text(&company_size)?,
        website: row.get(7).map_err(|e| translate_error(e, "entreprise"))?,
        city: row.get(8).map_err(|e| translate_error(e, "entreprise"))?,
        address: row.get(9).map_err(|e| translate_error(e, "entreprise"))?,
        notes: row.get(10).map_err(|e| translate_error(e, "entreprise"))?,
        created_at: read(11)?,
        updated_at: read(12)?,
    })
}

/// Clauses `WHERE` et paramètres liés correspondant à un filtre.
///
/// Chaque critère est un paramètre lié, jamais une valeur interpolée : la recherche libre
/// vient d'un champ de saisie, et la concaténer au SQL ouvrirait une injection.
fn clauses(filter: &CompanyFilter) -> AppResult<(String, Vec<Value>)> {
    let mut clauses = Vec::<String>::new();
    let mut values = Vec::<Value>::new();

    if !filter.search.trim().is_empty() {
        values.push(Value::Text(like_contains(&filter.search)));
        let index = values.len();
        clauses.push(format!(
            "(lower(e.name) LIKE ?{index} {LIKE_ESCAPE} \
              OR lower(coalesce(e.city, '')) LIKE ?{index} {LIKE_ESCAPE} \
              OR lower(coalesce(s.name, '')) LIKE ?{index} {LIKE_ESCAPE})"
        ));
    }
    if let Some(sector_id) = filter.sector_id {
        values.push(Value::Text(sector_id.to_string()));
        clauses.push(format!("e.sector_id = ?{}", values.len()));
    }
    if let Some(company_type_id) = filter
        .company_type_id
        .as_deref()
        .map(str::trim)
        .filter(|code| !code.is_empty())
    {
        values.push(Value::Text(company_type_id.to_owned()));
        clauses.push(format!("e.company_type_id = ?{}", values.len()));
    }
    if let Some(company_size) = filter.company_size {
        values.push(Value::Text(text_from_enum(&company_size)?));
        clauses.push(format!("e.company_size = ?{}", values.len()));
    }

    let sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    Ok((sql, values))
}

impl SqliteCompanyRepository {
    /// Relit une entreprise après écriture, jointures comprises.
    fn read_one(&self, conn: &rusqlite::Connection, id: uuid::Uuid) -> AppResult<Company> {
        let mut query = conn
            .prepare(&format!("SELECT {COLUMNS} {FROM_SQL} WHERE e.id = ?1"))
            .map_err(|e| translate_error(e, "entreprise"))?;
        let mut rows = query
            .query([id.to_string()])
            .map_err(|e| translate_error(e, "entreprise"))?;
        match rows.next().map_err(|e| translate_error(e, "entreprise"))? {
            Some(row) => row_to_company(row),
            None => Err(AppError::NotFound(format!("entreprise {id}"))),
        }
    }
}

impl CompanyRepository for SqliteCompanyRepository {
    fn list(&self) -> AppResult<Vec<Company>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(&format!(
                "SELECT {COLUMNS} {FROM_SQL} ORDER BY e.name COLLATE NOCASE ASC"
            ))
            .map_err(|e| translate_error(e, "entreprises"))?;
        let mut rows = query
            .query([])
            .map_err(|e| translate_error(e, "entreprises"))?;
        let mut companies = Vec::new();
        while let Some(row) = rows.next().map_err(|e| translate_error(e, "entreprises"))? {
            companies.push(row_to_company(row)?);
        }
        Ok(companies)
    }

    fn get(&self, id: uuid::Uuid) -> AppResult<Company> {
        let conn = connection(&self.pool)?;
        self.read_one(&conn, id)
    }

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &CompanyFilter,
    ) -> AppResult<Page<Company>> {
        let conn = connection(&self.pool)?;
        let page = page.max(1);
        let page_size = clamp_page_size(page_size);
        let (where_sql, mut values) = clauses(filter)?;

        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) {FROM_SQL}{where_sql}"),
                rusqlite::params_from_iter(values.iter()),
                |row| row.get(0),
            )
            .map_err(|e| translate_error(e, "entreprises"))?;

        values.push(Value::Integer(i64::try_from(page_size).unwrap_or(i64::MAX)));
        let index_limite = values.len();
        values.push(Value::Integer(
            i64::try_from(Page::<Company>::offset(page, page_size)).unwrap_or(i64::MAX),
        ));
        let index_offset = values.len();

        let mut query = conn
            .prepare(&format!(
                "SELECT {COLUMNS} {FROM_SQL}{where_sql} ORDER BY e.name COLLATE NOCASE ASC \
                 LIMIT ?{index_limite} OFFSET ?{index_offset}"
            ))
            .map_err(|e| translate_error(e, "entreprises"))?;
        let mut rows = query
            .query(rusqlite::params_from_iter(values.iter()))
            .map_err(|e| translate_error(e, "entreprises"))?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().map_err(|e| translate_error(e, "entreprises"))? {
            items.push(row_to_company(row)?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn create(&self, input: &NewCompany) -> AppResult<Company> {
        let conn = connection(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let now = now_iso();
        conn.execute(
            "INSERT INTO companies (id, name, sector_id, company_type_id, company_size,
                website, city, address, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            rusqlite::params![
                id.to_string(),
                input.name,
                input.sector_id.map(|value| value.to_string()),
                input.company_type_id,
                text_from_enum(&input.company_size)?,
                input.website,
                input.city,
                input.address,
                input.notes,
                now
            ],
        )
        .map_err(|e| {
            translate_constraint(
                e,
                "Le secteur ou le type d'entreprise sélectionné est introuvable",
                "entreprise",
            )
        })?;
        self.read_one(&conn, id)
    }

    fn update(&self, id: uuid::Uuid, input: &NewCompany) -> AppResult<Company> {
        let conn = connection(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE companies SET name = ?2, sector_id = ?3, company_type_id = ?4,
                    company_size = ?5, website = ?6, city = ?7, address = ?8, notes = ?9,
                    updated_at = ?10
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.name,
                    input.sector_id.map(|value| value.to_string()),
                    input.company_type_id,
                    text_from_enum(&input.company_size)?,
                    input.website,
                    input.city,
                    input.address,
                    input.notes,
                    now_iso()
                ],
            )
            .map_err(|e| {
                translate_constraint(
                    e,
                    "Le secteur ou le type d'entreprise sélectionné est introuvable",
                    "entreprise",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entreprise {id}")));
        }
        self.read_one(&conn, id)
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        let deleted = conn
            .execute("DELETE FROM companies WHERE id = ?1", [id.to_string()])
            .map_err(|e| {
                translate_constraint(
                    e,
                    "Suppression impossible : des candidatures sont liées à cette entreprise",
                    "entreprise",
                )
            })?;
        if deleted == 0 {
            return Err(AppError::NotFound(format!("entreprise {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
