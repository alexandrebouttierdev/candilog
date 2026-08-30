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

/// Ville effective : surcharge de la candidature, sinon celle de l'entreprise.
const EFFECTIVE_CITY: &str = "coalesce(c.city, e.city)";
/// Type d'entreprise effectif : surcharge de la candidature, sinon celui de l'entreprise.
const EFFECTIVE_COMPANY_TYPE: &str = "coalesce(c.company_type_id, e.company_type_id)";

/// Colonnes lues par [`row_to_application`], dans l'ordre.
const COLUMNS: &str =
    "c.id, c.job_title, c.company_id, e.name, coalesce(e.company_size, 'UNKNOWN'), \
                        c.contact_id, c.application_type, c.contract_type_code, ct.name, \
                        c.weekly_work_schedule, c.weekly_hours, \
                        c.professional_domain_id, pd.name, \
                        c.city, c.address, c.company_type_id, \
                        coalesce(c.city, e.city), coalesce(c.address, e.address), \
                        coalesce(c.company_type_id, e.company_type_id), cty.name, \
                        c.status, c.sent_date, c.job_url, c.notes, c.created_at, c.updated_at";

/// Source des colonnes.
///
/// Toutes les valeurs affichées — nom d'entreprise, libellés des référentiels, valeurs
/// héritées — sont résolues ici, en une requête. Les relire ligne par ligne côté Rust ou
/// React rendrait chaque page de liste proportionnelle en allers-retours à son nombre de
/// lignes.
///
/// La dernière jointure porte sur le type d'entreprise **effectif** : c'est le libellé que
/// l'interface affiche, surcharge comprise.
const FROM_SQL: &str = "FROM applications c \
     LEFT JOIN companies e ON e.id = c.company_id \
     LEFT JOIN contract_types ct ON ct.code = c.contract_type_code \
     LEFT JOIN professional_domains pd ON pd.code = c.professional_domain_id \
     LEFT JOIN company_types cty ON cty.code = coalesce(c.company_type_id, e.company_type_id)";

/// Convertit une ligne `SQLite` en candidature du domaine.
///
/// Renvoie `AppResult` et non `rusqlite::Result` : les enums serde (`statut`, nature de la
/// candidature, régime horaire) peuvent échouer à la conversion depuis le `TEXT` stocké.
fn row_to_application(row: &rusqlite::Row) -> AppResult<Application> {
    let read = |index: usize| -> AppResult<String> {
        row.get(index)
            .map_err(|e| translate_error(e, "candidature"))
    };
    let opt = |index: usize| -> AppResult<Option<String>> {
        row.get(index)
            .map_err(|e| translate_error(e, "candidature"))
    };
    let company_size = read(4)?;
    let application_type = read(6)?;
    let weekly_work_schedule = read(9)?;
    let status = read(20)?;
    Ok(Application {
        id: uuid_column(row, 0).map_err(|e| translate_error(e, "candidature"))?,
        job_title: read(1)?,
        company_id: uuid_column(row, 2).map_err(|e| translate_error(e, "candidature"))?,
        company_name: opt(3)?,
        company_size: enum_from_text(&company_size)?,
        contact_id: uuid_column_opt(row, 5).map_err(|e| translate_error(e, "candidature"))?,
        application_type: enum_from_text(&application_type)?,
        contract_type_code: read(7)?,
        contract_type_name: opt(8)?,
        weekly_work_schedule: enum_from_text(&weekly_work_schedule)?,
        weekly_hours: row.get(10).map_err(|e| translate_error(e, "candidature"))?,
        professional_domain_id: opt(11)?,
        professional_domain_name: opt(12)?,
        city: opt(13)?,
        address: opt(14)?,
        company_type_id: opt(15)?,
        effective_city: opt(16)?,
        effective_address: opt(17)?,
        effective_company_type_id: opt(18)?,
        effective_company_type_name: opt(19)?,
        status: enum_from_text(&status)?,
        sent_date: read(21)?,
        job_url: opt(22)?,
        notes: opt(23)?,
        created_at: read(24)?,
        updated_at: read(25)?,
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

/// Ajoute une clause `colonne IN (…)` sur une liste de valeurs textuelles.
fn push_in_clause(
    column: &str,
    textes: impl IntoIterator<Item = String>,
    values: &mut Vec<Value>,
    clauses: &mut Vec<String>,
) {
    let mut placeholders = Vec::new();
    for texte in textes {
        values.push(Value::Text(texte));
        placeholders.push(format!("?{}", values.len()));
    }
    if !placeholders.is_empty() {
        clauses.push(format!("{column} IN ({})", placeholders.join(", ")));
    }
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
        let index = values.len();
        clauses.push(format!(
            "(search_key(c.job_title) LIKE ?{index} {LIKE_ESCAPE} \
              OR search_key(coalesce(e.name, '')) LIKE ?{index} {LIKE_ESCAPE})"
        ));
    }

    let mut statuses = Vec::new();
    for status in &filter.status {
        statuses.push(text_from_enum(status)?);
    }
    push_in_clause("c.status", statuses, &mut values, &mut clauses);

    let mut types = Vec::new();
    for application_type in &filter.application_type {
        types.push(text_from_enum(application_type)?);
    }
    push_in_clause("c.application_type", types, &mut values, &mut clauses);

    let mut schedules = Vec::new();
    for schedule in &filter.weekly_work_schedule {
        schedules.push(text_from_enum(schedule)?);
    }
    push_in_clause(
        "c.weekly_work_schedule",
        schedules,
        &mut values,
        &mut clauses,
    );

    let mut sizes = Vec::new();
    for size in &filter.company_size {
        sizes.push(text_from_enum(size)?);
    }
    push_in_clause(
        "coalesce(e.company_size, 'UNKNOWN')",
        sizes,
        &mut values,
        &mut clauses,
    );

    push_in_clause(
        "c.contract_type_code",
        filter.contract_type_code.iter().cloned(),
        &mut values,
        &mut clauses,
    );
    push_in_clause(
        "c.professional_domain_id",
        filter.professional_domain_id.iter().cloned(),
        &mut values,
        &mut clauses,
    );
    push_in_clause(
        EFFECTIVE_COMPANY_TYPE,
        filter.company_type_id.iter().cloned(),
        &mut values,
        &mut clauses,
    );
    push_in_clause(
        "e.sector_id",
        filter.sector_id.iter().map(ToString::to_string),
        &mut values,
        &mut clauses,
    );

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
            &format!("search_key(coalesce({EFFECTIVE_CITY}, '')) LIKE ? {LIKE_ESCAPE}"),
            pattern(&filter.city),
            &mut values,
            &mut clauses,
        );
    }
    if !filter.job_title.trim().is_empty() {
        add(
            &format!("search_key(c.job_title) LIKE ? {LIKE_ESCAPE}"),
            pattern(&filter.job_title),
            &mut values,
            &mut clauses,
        );
    }
    // Une candidature sans volume horaire n'entre dans aucune des deux bornes : le filtre
    // porte sur un nombre d'heures, et « non renseigné » n'en est pas un.
    if let Some(min) = filter.min_weekly_hours {
        add(
            "c.weekly_hours >= ?",
            Value::Real(min),
            &mut values,
            &mut clauses,
        );
    }
    if let Some(max) = filter.max_weekly_hours {
        add(
            "c.weekly_hours <= ?",
            Value::Real(max),
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
    push_in_clause(
        "c.id",
        filter.ids.iter().map(ToString::to_string),
        &mut values,
        &mut clauses,
    );

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
        ApplicationSort::JobTitle => "search_key(c.job_title)",
        ApplicationSort::Company => "search_key(coalesce(e.name, ''))",
        ApplicationSort::Status => "c.status",
        ApplicationSort::Date => "c.sent_date",
    }
}

/// Paramètres d'écriture d'une candidature, dans l'ordre attendu par `INSERT` et `UPDATE`.
///
/// Le lien de l'offre est effacé pour une candidature spontanée : le service le normalise
/// déjà, la base le refuse par un `CHECK`, et le dépôt n'a aucune raison de tenter
/// l'écriture d'une valeur que les deux autres couches interdisent.
fn write_params(input: &NewApplication, status: &str, now: &str) -> AppResult<Vec<Value>> {
    let application_type = text_from_enum(&input.application_type)?;
    let job_url = if input.application_type
        == crate::features::applications::domain::ApplicationType::Unsolicited
    {
        None
    } else {
        input.job_url.clone()
    };
    Ok(vec![
        Value::Text(input.company_id.to_string()),
        input
            .contact_id
            .map_or(Value::Null, |id| Value::Text(id.to_string())),
        Value::Text(input.job_title.clone()),
        Value::Text(application_type),
        Value::Text(input.contract_type_code.clone()),
        Value::Text(text_from_enum(&input.weekly_work_schedule)?),
        input.weekly_hours.map_or(Value::Null, Value::Real),
        input
            .professional_domain_id
            .clone()
            .map_or(Value::Null, Value::Text),
        input.city.clone().map_or(Value::Null, Value::Text),
        input.address.clone().map_or(Value::Null, Value::Text),
        input
            .company_type_id
            .clone()
            .map_or(Value::Null, Value::Text),
        Value::Text(status.to_owned()),
        Value::Text(input.sent_date.clone()),
        job_url.map_or(Value::Null, Value::Text),
        input.notes.clone().map_or(Value::Null, Value::Text),
        Value::Text(now.to_owned()),
    ])
}

/// Phrase rendue à l'utilisateur quand une référence de la candidature est introuvable.
const REFERENCE_INTROUVABLE: &str =
    "L'entreprise, le contact ou l'un des référentiels sélectionnés est introuvable";

impl SqliteApplicationRepository {
    /// Relit une candidature après écriture, jointures et valeurs héritées comprises.
    fn read_one(&self, conn: &rusqlite::Connection, id: Uuid) -> AppResult<Application> {
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
        self.read_one(&conn, id)
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
        let mut params = write_params(input, &status, &now)?;
        params.insert(0, Value::Text(id.to_string()));

        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "création de la candidature"))?;
        transaction
            .execute(
                "INSERT INTO applications (id, company_id, contact_id, job_title,
                    application_type, contract_type_code, weekly_work_schedule, weekly_hours,
                    professional_domain_id, city, address, company_type_id, status, sent_date,
                    job_url, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?17)",
                rusqlite::params_from_iter(params.iter()),
            )
            .map_err(|e| translate_constraint(e, REFERENCE_INTROUVABLE, "candidature"))?;
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
        let mut params = write_params(input, &status, &now)?;
        params.insert(0, Value::Text(id.to_string()));

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
                "UPDATE applications SET company_id = ?2, contact_id = ?3, job_title = ?4,
                    application_type = ?5, contract_type_code = ?6, weekly_work_schedule = ?7,
                    weekly_hours = ?8, professional_domain_id = ?9, city = ?10, address = ?11,
                    company_type_id = ?12, status = ?13, sent_date = ?14, job_url = ?15,
                    notes = ?16, updated_at = ?17
                 WHERE id = ?1",
                rusqlite::params_from_iter(params.iter()),
            )
            .map_err(|e| translate_constraint(e, REFERENCE_INTROUVABLE, "candidature"))?;
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
        // Relances, entretiens et historique de statut partent en cascade (`ON DELETE
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
