//! Dépôt `SQLite` des entretiens.

use crate::core::database::helpers::{
    connection, enum_from_text, now_iso, text_from_enum, translate_error, uuid_column,
    uuid_column_opt,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::interviews::domain::{
    InterviewAnalysis, Interview, InterviewRepository, NewInterview,
};
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt d'entretiens.
pub struct SqliteInterviewRepository {
    pool: SqlitePool,
}

impl SqliteInterviewRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Columns lues par [`row_vers_interview`], dans l'ordre.
const COLUMNS: &str = "e.id, e.application_id, c.job_title, ent.name, e.contact_id, \
                        ct.first_name || ' ' || ct.name, e.interview_date, e.type, e.location, e.notes, \
                        e.minutes, e.ai_analysis, e.created_at, e.updated_at";

/// Source des colonnes : trois jointures gauches, la candidature étant la seule obligatoire
/// en base — l'entreprise et le contact peuvent manquer.
const FROM_SQL: &str = "FROM interviews e \
                      LEFT JOIN applications c ON c.id = e.application_id \
                      LEFT JOIN companies ent ON ent.id = c.company_id \
                      LEFT JOIN contacts ct ON ct.id = e.contact_id";

/// Convertit une ligne `SQLite` en entretien du domaine.
fn row_vers_interview(row: &rusqlite::Row) -> AppResult<Interview> {
    let error = |e: rusqlite::Error| translate_error(e, "entretien");
    let type_text: String = row.get(7).map_err(error)?;
    let analysis_json: Option<String> = row.get(11).map_err(error)?;
    let analysis_ai = match analysis_json {
        None => None,
        Some(json) => {
            Some(serde_json::from_str(&json).map_err(|e| AppError::Serialization(e.to_string()))?)
        }
    };
    Ok(Interview {
        id: uuid_column(row, 0).map_err(error)?,
        application_id: uuid_column(row, 1).map_err(error)?,
        application_job_title: row.get(2).map_err(error)?,
        company_name: row.get(3).map_err(error)?,
        contact_id: uuid_column_opt(row, 4).map_err(error)?,
        contact_name: row.get(5).map_err(error)?,
        interview_date: row.get(6).map_err(error)?,
        type_interview: enum_from_text(&type_text)?,
        location: row.get(8).map_err(error)?,
        notes: row.get(9).map_err(error)?,
        minutes: row.get(10).map_err(error)?,
        analysis_ai,
        created_at: row.get(12).map_err(error)?,
        updated_at: row.get(13).map_err(error)?,
    })
}

/// Lit une requête d'entretiens jusqu'au bout.
fn collecter(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> AppResult<Vec<Interview>> {
    let mut query = conn
        .prepare(sql)
        .map_err(|e| translate_error(e, "entretiens"))?;
    let mut rows = query
        .query(params)
        .map_err(|e| translate_error(e, "entretiens"))?;
    let mut items = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|e| translate_error(e, "entretiens"))?
    {
        items.push(row_vers_interview(row)?);
    }
    Ok(items)
}

impl InterviewRepository for SqliteInterviewRepository {
    fn list(&self) -> AppResult<Vec<Interview>> {
        let conn = connection(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLUMNS} {FROM_SQL} ORDER BY e.interview_date ASC"),
            &[],
        )
    }

    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Interview>> {
        let conn = connection(&self.pool)?;
        collecter(
            &conn,
            &format!(
                "SELECT {COLUMNS} {FROM_SQL} \
                 WHERE e.interview_date >= ?1 AND e.interview_date <= ?2 \
                 ORDER BY e.interview_date ASC"
            ),
            &[&from, &to],
        )
    }

    fn get(&self, id: Uuid) -> AppResult<Interview> {
        let conn = connection(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLUMNS} {FROM_SQL} WHERE e.id = ?1"),
            &[&id.to_string()],
        )?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("entretien {id}")))
    }

    fn save_and_mark_candidate(
        &self,
        id: Option<Uuid>,
        input: &NewInterview,
    ) -> AppResult<Interview> {
        let mut conn = connection(&self.pool)?;
        let interview_id = id.unwrap_or_else(Uuid::new_v4);
        let now = now_iso();
        let type_interview = text_from_enum(&input.type_interview)?;
        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "enregistrement de l'entretien"))?;

        // Le statut de la candidature est lu **avant** l'écriture de l'entretien : c'est ce
        // qui permet de n'historiser que les passages réels à l'étape entretien. La lecture
        // vaut aussi contrôle d'existence — la clé étrangère la refuserait plus loin, mais
        // avec un message technique.
        let status_previous: String = transaction
            .query_row(
                "SELECT status FROM applications WHERE id = ?1",
                [input.application_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::Validation(
                    "La candidature liée à cet entretien est introuvable".into(),
                ),
                other => translate_error(other, "candidature"),
            })?;

        let settings = rusqlite::params![
            interview_id.to_string(),
            input.application_id.to_string(),
            input.contact_id.map(|value| value.to_string()),
            input.interview_date,
            type_interview,
            input.location,
            input.notes,
            input.minutes,
            now,
        ];
        let modifiees = if id.is_some() {
            transaction.execute(
                "UPDATE interviews SET application_id = ?2, contact_id = ?3, interview_date = ?4,
                    type = ?5, location = ?6, notes = ?7, minutes = ?8, updated_at = ?9
                 WHERE id = ?1",
                settings,
            )
        } else {
            transaction.execute(
                "INSERT INTO interviews (id, application_id, contact_id, interview_date, type,
                    location, notes, minutes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                settings,
            )
        }
        .map_err(|e| translate_error(e, "entretien"))?;

        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entretien {interview_id}")));
        }

        // Planifier un entretien fait avancer la candidature : c'est la règle métier que
        // l'utilisateur attend, et l'oublier laisserait des candidatures « en attente »
        // alors qu'un entretien est déjà au calendrier.
        let status_interview = "ENTRETIEN";
        transaction
            .execute(
                "UPDATE applications SET status = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![
                    input.application_id.to_string(),
                    status_interview,
                    now
                ],
            )
            .map_err(|e| translate_error(e, "candidature"))?;
        if status_previous != status_interview {
            transaction
                .execute(
                    "INSERT INTO status_history (id, application_id, status, changed_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        Uuid::new_v4().to_string(),
                        input.application_id.to_string(),
                        status_interview,
                        now,
                    ],
                )
                .map_err(|e| translate_error(e, "historique du statut"))?;
        }

        transaction
            .commit()
            .map_err(|e| translate_error(e, "enregistrement de l'entretien"))?;
        self.get(interview_id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        // La candidature garde son statut : supprimer un entretien annulé ne veut pas dire
        // que la candidature n'a jamais atteint cette étape.
        conn.execute("DELETE FROM interviews WHERE id = ?1", [id.to_string()])
            .map_err(|e| translate_error(e, "entretien"))?;
        Ok(())
    }

    fn save_analysis(&self, id: Uuid, analysis: &InterviewAnalysis) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        let json = serde_json::to_string(analysis)?;
        let modifiees = conn
            .execute(
                "UPDATE interviews SET ai_analysis = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id.to_string(), json, now_iso()],
            )
            .map_err(|e| translate_error(e, "entretien"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entretien {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
