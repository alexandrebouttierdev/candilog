//! Dépôt `SQLite` des candidatures.

use crate::core::database::helpers::{
    connection, enum_from_text, like_contains, now_iso, text_from_enum, translate_constraint,
    translate_error, uuid_column, uuid_column_opt, LIKE_ESCAPE,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{clamp_page_size, Page};
use crate::features::applications::domain::{
    Application, ApplicationFilter, ApplicationRepository, ApplicationSort, ApplicationStatus,
    NewApplication, PipelineBreakdown,
};
use rusqlite::types::Value;
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt de candidatures.
pub struct SqliteApplicationRepository {
    pool: SqlitePool,
}

impl SqliteApplicationRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Columns lues par [`row_to_application`], dans l'ordre.
const COLUMNS: &str = "c.id, c.job_title, c.company_id, e.name, e.city, c.contact_id, \
                        c.contract_type, c.status, c.sent_date, c.job_url, c.notes, \
                        c.created_at, c.updated_at";

/// Source des colonnes : `LEFT JOIN` pour exposer le nom et la ville de l'entreprise.
const FROM_SQL: &str = "FROM applications c LEFT JOIN companies e ON e.id = c.company_id";

/// Convertit une ligne `SQLite` en candidature du domaine.
///
/// Renvoie `AppResult` et non `rusqlite::Result` : `contract_type` et `statut` sont des enums
/// serde, dont la conversion depuis le `TEXT` stocké peut échouer sur une valeur héritée
/// qu'aucune migration n'aurait normalisée.
fn row_to_application(row: &rusqlite::Row) -> AppResult<Application> {
    let read = |index: usize| -> AppResult<String> {
        row.get(index)
            .map_err(|e| translate_error(e, "candidature"))
    };
    let contract_type = read(6)?;
    let status = read(7)?;
    Ok(Application {
        id: uuid_column(row, 0).map_err(|e| translate_error(e, "candidature"))?,
        job_title: read(1)?,
        company_id: uuid_column(row, 2).map_err(|e| translate_error(e, "candidature"))?,
        company_name: row.get(3).map_err(|e| translate_error(e, "candidature"))?,
        company_city: row.get(4).map_err(|e| translate_error(e, "candidature"))?,
        contact_id: uuid_column_opt(row, 5).map_err(|e| translate_error(e, "candidature"))?,
        contract_type: enum_from_text(&contract_type)?,
        status: enum_from_text(&status)?,
        sent_date: read(8)?,
        job_url: row.get(9).map_err(|e| translate_error(e, "candidature"))?,
        notes: row.get(10).map_err(|e| translate_error(e, "candidature"))?,
        created_at: read(11)?,
        updated_at: read(12)?,
    })
}

/// Ajoute une étape à l'historique de statut.
///
/// L'historique est ce qui permet de compter les candidatures **passées** par l'entretien,
/// y compris celles qui ont ensuite été refusées : le statut courant seul les perdrait, et
/// l'entonnoir de conversion des analyses afficherait un taux faux.
fn save_status(
    conn: &rusqlite::Connection,
    application_id: Uuid,
    status: &str,
    changed_at: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO status_history (id, application_id, status, changed_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            Uuid::new_v4().to_string(),
            application_id.to_string(),
            status,
            changed_at,
        ],
    )
    .map(|_| ())
    .map_err(|e| translate_error(e, "historique du statut"))
}

/// Clauses `WHERE` et paramètres liés correspondant à un filtre.
///
/// Chaque critère est un paramètre lié, jamais une valeur interpolée : le poste, la ville et
/// la recherche libre viennent de champs de saisie, et les concaténer au SQL ouvrirait une
/// injection.
fn clauses(filter: &ApplicationFilter) -> AppResult<(String, Vec<Value>)> {
    let mut clauses = Vec::<String>::new();
    let mut values = Vec::<Value>::new();

    let add = |clause: &str, value: Value, values: &mut Vec<Value>, clauses: &mut Vec<String>| {
        values.push(value);
        clauses.push(clause.replace('?', &format!("?{}", values.len())));
    };

    let pattern = |text: &str| Value::Text(like_contains(text));

    if !filter.search.trim().is_empty() {
        values.push(pattern(&filter.search));
        let first = values.len();
        values.push(pattern(&filter.search));
        let second = values.len();
        clauses.push(format!(
            "(lower(c.job_title) LIKE ?{first} {LIKE_ESCAPE} OR lower(coalesce(e.name, '')) LIKE ?{second} {LIKE_ESCAPE})"
        ));
    }
    if !filter.status.is_empty() {
        let mut placeholders = Vec::new();
        for status in &filter.status {
            values.push(Value::Text(text_from_enum(status)?));
            placeholders.push(format!("?{}", values.len()));
        }
        clauses.push(format!("c.status IN ({})", placeholders.join(", ")));
    }
    if !filter.contract.is_empty() {
        let mut placeholders = Vec::new();
        for contract in &filter.contract {
            values.push(Value::Text(text_from_enum(contract)?));
            placeholders.push(format!("?{}", values.len()));
        }
        clauses.push(format!("c.contract_type IN ({})", placeholders.join(", ")));
    }
    if let Some(company_id) = filter.company_id {
        add(
            "c.company_id = ?",
            Value::Text(company_id.to_string()),
            &mut values,
            &mut clauses,
        );
    }
    if !filter.city.trim().is_empty() {
        add(
            &format!("lower(coalesce(e.city, '')) LIKE ? {LIKE_ESCAPE}"),
            pattern(&filter.city),
            &mut values,
            &mut clauses,
        );
    }
    if !filter.job_title.trim().is_empty() {
        add(
            &format!("lower(c.job_title) LIKE ? {LIKE_ESCAPE}"),
            pattern(&filter.job_title),
            &mut values,
            &mut clauses,
        );
    }
    if let Some(start) = &filter.start_date {
        add(
            "c.sent_date >= ?",
            Value::Text(start.clone()),
            &mut values,
            &mut clauses,
        );
    }
    if let Some(end) = &filter.end_date {
        add(
            "c.sent_date <= ?",
            Value::Text(end.clone()),
            &mut values,
            &mut clauses,
        );
    }
    if !filter.ids.is_empty() {
        let mut placeholders = Vec::new();
        for id in &filter.ids {
            values.push(Value::Text(id.to_string()));
            placeholders.push(format!("?{}", values.len()));
        }
        clauses.push(format!("c.id IN ({})", placeholders.join(", ")));
    }

    let sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    Ok((sql, values))
}

/// Expression `ORDER BY` correspondant à la colonne de tri.
///
/// Le jeu de valeurs est fermé par l'enum : rien de ce qui vient de l'IPC n'atteint le SQL.
const fn sort_column(sort: ApplicationSort) -> &'static str {
    match sort {
        ApplicationSort::JobTitle => "lower(c.job_title)",
        ApplicationSort::Company => "lower(coalesce(e.name, ''))",
        ApplicationSort::Status => "c.status",
        ApplicationSort::Date => "c.sent_date",
    }
}

impl ApplicationRepository for SqliteApplicationRepository {
    fn list(&self) -> AppResult<Vec<Application>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(&format!(
                "SELECT {COLUMNS} {FROM_SQL} ORDER BY c.sent_date DESC, c.created_at DESC"
            ))
            .map_err(|e| translate_error(e, "candidatures"))?;
        let mut rows = query
            .query([])
            .map_err(|e| translate_error(e, "candidatures"))?;
        let mut applications = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| translate_error(e, "candidatures"))?
        {
            applications.push(row_to_application(row)?);
        }
        Ok(applications)
    }

    fn get(&self, id: Uuid) -> AppResult<Application> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(&format!("SELECT {COLUMNS} {FROM_SQL} WHERE c.id = ?1"))
            .map_err(|e| translate_error(e, "candidature"))?;
        let mut rows = query
            .query([id.to_string()])
            .map_err(|e| translate_error(e, "candidature"))?;
        match rows.next().map_err(|e| translate_error(e, "candidature"))? {
            Some(row) => row_to_application(row),
            None => Err(AppError::NotFound(format!("candidature {id}"))),
        }
    }

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &ApplicationFilter,
    ) -> AppResult<Page<Application>> {
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
            .map_err(|e| translate_error(e, "candidatures"))?;

        let direction = if filter.descending { "DESC" } else { "ASC" };
        values.push(Value::Integer(i64::try_from(page_size).unwrap_or(i64::MAX)));
        let index_limite = values.len();
        values.push(Value::Integer(
            i64::try_from(Page::<Application>::offset(page, page_size)).unwrap_or(i64::MAX),
        ));
        let index_offset = values.len();

        // `c.created_at DESC` en second critère : sans lui, deux candidatures de même date
        // d'envoi changeraient d'ordre d'une page à l'autre, et une ligne pourrait
        // apparaître deux fois ou pas du tout à la pagination.
        let sql = format!(
            "SELECT {COLUMNS} {FROM_SQL}{where_sql} ORDER BY {} {direction}, c.created_at DESC \
             LIMIT ?{index_limite} OFFSET ?{index_offset}",
            sort_column(filter.sort)
        );
        let mut query = conn
            .prepare(&sql)
            .map_err(|e| translate_error(e, "candidatures"))?;
        let mut rows = query
            .query(rusqlite::params_from_iter(values.iter()))
            .map_err(|e| translate_error(e, "candidatures"))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| translate_error(e, "candidatures"))?
        {
            items.push(row_to_application(row)?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn breakdown(&self, filter: &ApplicationFilter) -> AppResult<PipelineBreakdown> {
        let conn = connection(&self.pool)?;
        // La répartition ignore le filtre de statut : le Kanban affiche les quatre colonnes,
        // et n'en compter qu'une viderait les trois autres.
        let mut sans_status = filter.clone();
        sans_status.status.clear();
        let (where_sql, mut values) = clauses(&sans_status)?;

        let statuses = [
            text_from_enum(&ApplicationStatus::Pending)?,
            text_from_enum(&ApplicationStatus::FollowedUp)?,
            text_from_enum(&ApplicationStatus::Interview)?,
            text_from_enum(&ApplicationStatus::Rejected)?,
        ];
        let base = values.len();
        for status in &statuses {
            values.push(Value::Text(status.clone()));
        }

        let sql = format!(
            "SELECT coalesce(sum(CASE WHEN c.status = ?{} THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.status = ?{} THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.status = ?{} THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.status = ?{} THEN 1 ELSE 0 END), 0)
             {FROM_SQL}{where_sql}",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
        );
        conn.query_row(&sql, rusqlite::params_from_iter(values.iter()), |row| {
            Ok(PipelineBreakdown {
                pending: row.get(0)?,
                followed_up: row.get(1)?,
                interview: row.get(2)?,
                rejected: row.get(3)?,
            })
        })
        .map_err(|e| translate_error(e, "répartition du pipeline"))
    }

    fn create(&self, input: &NewApplication) -> AppResult<Application> {
        let mut conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let now = now_iso();
        let status = text_from_enum(&input.status)?;
        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "création de la candidature"))?;
        transaction
            .execute(
                "INSERT INTO applications (id, company_id, contact_id, job_title, contract_type,
                    status, sent_date, job_url, notes, created_at, updated_at)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                rusqlite::params![
                    id.to_string(),
                    input.company_id.to_string(),
                    input.job_title,
                    text_from_enum(&input.contract_type)?,
                    status,
                    input.sent_date,
                    input.job_url,
                    input.notes,
                    now
                ],
            )
            .map_err(|e| {
                translate_constraint(
                    e,
                    "L'entreprise liée à cette candidature est introuvable",
                    "candidature",
                )
            })?;
        // Même transaction que l'insertion : une candidature sans étape initiale serait
        // invisible de l'entonnoir de conversion.
        save_status(&transaction, id, &status, &now)?;
        transaction
            .commit()
            .map_err(|e| translate_error(e, "création de la candidature"))?;
        self.get(id)
    }

    fn update(&self, id: Uuid, input: &NewApplication) -> AppResult<Application> {
        let mut conn = connection(&self.pool)?;
        let now = now_iso();
        let status = text_from_enum(&input.status)?;
        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "modification de la candidature"))?;

        let ancien_status: Option<String> = transaction
            .query_row(
                "SELECT status FROM applications WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .ok();
        let Some(ancien_status) = ancien_status else {
            return Err(AppError::NotFound(format!("candidature {id}")));
        };

        transaction
            .execute(
                "UPDATE applications SET company_id = ?2, job_title = ?3, contract_type = ?4,
                    status = ?5, sent_date = ?6, job_url = ?7, notes = ?8, updated_at = ?9
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.company_id.to_string(),
                    input.job_title,
                    text_from_enum(&input.contract_type)?,
                    status,
                    input.sent_date,
                    input.job_url,
                    input.notes,
                    now
                ],
            )
            .map_err(|e| {
                translate_constraint(
                    e,
                    "L'entreprise liée à cette candidature est introuvable",
                    "candidature",
                )
            })?;
        // L'historique n'enregistre que les changements réels : réenregistrer le poste sans
        // toucher au statut ajouterait une étape fictive à l'entonnoir.
        if ancien_status != status {
            save_status(&transaction, id, &status, &now)?;
        }
        transaction
            .commit()
            .map_err(|e| translate_error(e, "modification de la candidature"))?;
        self.get(id)
    }

    fn update_status(&self, id: Uuid, status: ApplicationStatus) -> AppResult<Application> {
        let mut conn = connection(&self.pool)?;
        let now = now_iso();
        let status = text_from_enum(&status)?;
        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "changement de statut"))?;
        let modifiees = transaction
            .execute(
                "UPDATE applications SET status = ?2, updated_at = ?3
                 WHERE id = ?1 AND status <> ?2",
                rusqlite::params![id.to_string(), status, now],
            )
            .map_err(|e| translate_error(e, "changement de statut"))?;
        if modifiees > 0 {
            save_status(&transaction, id, &status, &now)?;
        }
        transaction
            .commit()
            .map_err(|e| translate_error(e, "changement de statut"))?;
        // `modifiees == 0` couvre deux cas : identifiant inconnu, ou statut déjà à la valeur
        // demandée — un glisser-déposer reposant la carte dans sa colonne d'origine. `get`
        // distingue les deux en renvoyant `NotFound` pour le premier.
        self.get(id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        // FollowUps, entretiens et historique de statut partent en cascade (`ON DELETE
        // CASCADE` du schéma) ; l'entreprise et le contact sont conservés.
        let deleted = conn
            .execute("DELETE FROM applications WHERE id = ?1", [id.to_string()])
            .map_err(|e| translate_error(e, "candidature"))?;
        if deleted == 0 {
            return Err(AppError::NotFound(format!("candidature {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
