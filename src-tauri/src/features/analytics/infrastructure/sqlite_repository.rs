//! Agrégats d'analyse calculés par `SQLite`.

use crate::core::database::helpers::{connection, translate_error, uuid_column};
use crate::core::database::SqlitePool;
use crate::core::errors::AppResult;
use crate::features::analytics::domain::{
    ActivityWeek, AnalyticsRepository, Metrics, Performance, Step, ToFollowUp, UpcomingItem,
};
use crate::features::applications::domain::{
    Application, ApplicationFilter, ApplicationRepository, ApplicationSort,
};
use crate::features::applications::infrastructure::SqliteApplicationRepository;

/// Dépôt des analyses sur la base locale.
pub struct SqliteAnalyticsRepository {
    pool: SqlitePool,
}

impl SqliteAnalyticsRepository {
    /// Construit le dépôt à partir du pool partagé.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn percentage(part: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part.min(total) as f64 / total as f64) * 100.0).round() as u8
}

impl AnalyticsRepository for SqliteAnalyticsRepository {
    fn metrics(&self, from: Option<&str>) -> AppResult<Metrics> {
        let conn = connection(&self.pool)?;
        let (applications, interviews, responses, rejected, pending, followed_up) = conn
            .query_row(
                "SELECT count(*),
                    coalesce(sum(CASE WHEN
                        c.status = 'ENTRETIEN'
                        OR EXISTS (SELECT 1 FROM status_history h
                                   WHERE h.application_id = c.id AND h.status = 'ENTRETIEN')
                        OR EXISTS (SELECT 1 FROM interviews e WHERE e.application_id = c.id)
                    THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN
                        c.status IN ('ENTRETIEN', 'REFUS')
                        OR EXISTS (SELECT 1 FROM status_history h
                                   WHERE h.application_id = c.id
                                     AND h.status IN ('ENTRETIEN', 'REFUS'))
                        OR EXISTS (SELECT 1 FROM interviews e WHERE e.application_id = c.id)
                    THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.status = 'REFUS' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.status = 'EN_ATTENTE' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN
                        c.status = 'RELANCEE'
                        OR EXISTS (SELECT 1 FROM status_history h
                                   WHERE h.application_id = c.id AND h.status = 'RELANCEE')
                        OR EXISTS (SELECT 1 FROM follow_ups r WHERE r.application_id = c.id)
                    THEN 1 ELSE 0 END), 0)
                 FROM applications c
                 WHERE ?1 IS NULL OR substr(c.sent_date, 1, 10) >= ?1",
                rusqlite::params![from],
                |row| {
                    Ok((
                        row.get::<_, u64>(0)?,
                        row.get::<_, u64>(1)?,
                        row.get::<_, u64>(2)?,
                        row.get::<_, u64>(3)?,
                        row.get::<_, u64>(4)?,
                        row.get::<_, u64>(5)?,
                    ))
                },
            )
            .map_err(|error| translate_error(error, "indicateurs"))?;
        Ok(Metrics {
            applications,
            interviews,
            responses,
            rejected,
            pending,
            followed_up,
            response_rate: percentage(responses, applications),
            interview_rate: percentage(interviews, applications),
        })
    }

    fn performance(&self, from: Option<&str>) -> AppResult<Performance> {
        let conn = connection(&self.pool)?;
        let (count, premiere): (u64, Option<String>) = conn
            .query_row(
                "SELECT count(*), min(substr(sent_date, 1, 10)) FROM applications
                 WHERE ?1 IS NULL OR substr(sent_date, 1, 10) >= ?1",
                rusqlite::params![from],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| translate_error(error, "rythme des candidatures"))?;

        let delay: Option<f64> = conn
            .query_row(
                "WITH responses(application_id, day) AS (
                    SELECT application_id, substr(changed_at, 1, 10)
                    FROM status_history WHERE status IN ('ENTRETIEN', 'REFUS')
                    UNION ALL
                    SELECT application_id, substr(interview_date, 1, 10) FROM interviews
                    UNION ALL
                    SELECT id, substr(updated_at, 1, 10) FROM applications
                    WHERE status IN ('ENTRETIEN', 'REFUS')
                 ), firsts AS (
                    SELECT application_id, min(day) AS day
                    FROM responses GROUP BY application_id
                 )
                 SELECT avg(max(0, julianday(p.day) - julianday(substr(c.sent_date, 1, 10))))
                 FROM applications c
                 JOIN firsts p ON p.application_id = c.id
                 WHERE ?1 IS NULL OR substr(c.sent_date, 1, 10) >= ?1",
                rusqlite::params![from],
                |row| row.get(0),
            )
            .map_err(|error| translate_error(error, "délai de réponse"))?;

        let today = chrono::Local::now().date_naive();
        let start = from
            .and_then(|value| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
            .or_else(|| {
                premiere
                    .as_deref()
                    .and_then(|value| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
            })
            .unwrap_or(today);
        let semaines = ((today - start).num_days().max(0) as f64 / 7.0).max(1.0);
        let applications_per_week = ((count as f64 / semaines) * 10.0).round() / 10.0;
        let day = today.format("%Y-%m-%d").to_string();
        let upcoming_interviews = conn
            .query_row(
                "SELECT count(*) FROM interviews WHERE substr(interview_date, 1, 10) >= ?1",
                [&day],
                |row| row.get(0),
            )
            .map_err(|error| translate_error(error, "entretiens à venir"))?;
        let overdue_follow_ups = conn
            .query_row(
                "SELECT count(*) FROM follow_ups WHERE follow_up_date < ?1",
                [&day],
                |row| row.get(0),
            )
            .map_err(|error| translate_error(error, "relances en retard"))?;

        Ok(Performance {
            average_response_days: delay.map(|value| value.round().max(0.0) as u64),
            applications_per_week,
            upcoming_interviews,
            overdue_follow_ups,
        })
    }

    fn activity_hebdomadaire(&self, semaines: u32) -> AppResult<Vec<ActivityWeek>> {
        let conn = connection(&self.pool)?;
        let semaines = semaines.clamp(1, 104);
        let today = chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        let mut query = conn
            .prepare(
                "WITH RECURSIVE periods(period_index, start, period_end) AS (
                    SELECT 0,
                           date(?1, printf('-%d days', (?2 * 7) - 1)),
                           date(?1, printf('-%d days', (?2 * 7) - 7))
                    UNION ALL
                    SELECT period_index + 1, date(start, '+7 days'), date(period_end, '+7 days')
                    FROM periods WHERE period_index + 1 < ?2
                 )
                 SELECT p.start, count(c.id)
                 FROM periods p
                 LEFT JOIN applications c
                   ON substr(c.sent_date, 1, 10) BETWEEN p.start AND p.period_end
                 GROUP BY p.period_index, p.start ORDER BY p.period_index",
            )
            .map_err(|error| translate_error(error, "activité hebdomadaire"))?;
        let rows = query
            .query_map(rusqlite::params![today, semaines], |row| {
                Ok(ActivityWeek {
                    start: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|error| translate_error(error, "activité hebdomadaire"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| translate_error(error, "activité hebdomadaire"))
    }

    fn pipeline(&self) -> AppResult<Vec<Step>> {
        let conn = connection(&self.pool)?;
        let (total, pending, followed_up, interviews, rejected): (u64, u64, u64, u64, u64) = conn
            .query_row(
                "SELECT count(*),
                    coalesce(sum(CASE WHEN status = 'EN_ATTENTE' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN status = 'RELANCEE' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN status = 'ENTRETIEN' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN status = 'REFUS' THEN 1 ELSE 0 END), 0)
                 FROM applications",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .map_err(|error| translate_error(error, "pipeline"))?;
        Ok([
            ("En attente", pending),
            ("Relancées", followed_up),
            ("Entretiens", interviews),
            ("Refusées", rejected),
        ]
        .into_iter()
        .map(|(label, count)| Step {
            label: label.into(),
            count,
            percentage: percentage(count, total),
        })
        .collect())
    }

    fn upcoming_items(&self, today: &str, limite: u64) -> AppResult<Vec<UpcomingItem>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(
                "SELECT e.id, 'entretien', e.interview_date, c.job_title, ent.name, e.type
                 FROM interviews e
                 LEFT JOIN applications c ON c.id = e.application_id
                 LEFT JOIN companies ent ON ent.id = c.company_id
                 WHERE substr(e.interview_date, 1, 10) >= ?1
                 UNION ALL
                 SELECT r.id, 'relance', r.follow_up_date, c.job_title, ent.name, r.type
                 FROM follow_ups r
                 LEFT JOIN applications c ON c.id = r.application_id
                 LEFT JOIN companies ent ON ent.id = c.company_id
                 WHERE r.follow_up_date >= ?1
                 ORDER BY 3 ASC LIMIT ?2",
            )
            .map_err(|error| translate_error(error, "échéances"))?;
        let mut rows = query
            .query(rusqlite::params![today, limite.max(1)])
            .map_err(|error| translate_error(error, "échéances"))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|error| translate_error(error, "échéances"))?
        {
            items.push(UpcomingItem {
                id: uuid_column(row, 0).map_err(|error| translate_error(error, "échéance"))?,
                kind: row
                    .get(1)
                    .map_err(|error| translate_error(error, "échéance"))?,
                date: row
                    .get(2)
                    .map_err(|error| translate_error(error, "échéance"))?,
                job_title: row
                    .get(3)
                    .map_err(|error| translate_error(error, "échéance"))?,
                company_name: row
                    .get(4)
                    .map_err(|error| translate_error(error, "échéance"))?,
                detail: row
                    .get(5)
                    .map_err(|error| translate_error(error, "échéance"))?,
            });
        }
        Ok(items)
    }

    fn to_follow_up(&self, today: &str, days: u64, limite: u64) -> AppResult<Vec<ToFollowUp>> {
        let conn = connection(&self.pool)?;
        let date = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let seuil = (date - chrono::Duration::days(i64::try_from(days).unwrap_or(i64::MAX)))
            .format("%Y-%m-%d")
            .to_string();
        let mut query = conn
            .prepare(
                "SELECT c.id, c.job_title, e.name, substr(c.sent_date, 1, 10),
                        cast(max(0, julianday(?1) - julianday(substr(c.sent_date, 1, 10))) AS INTEGER)
                 FROM applications c
                 LEFT JOIN companies e ON e.id = c.company_id
                 WHERE c.status = 'EN_ATTENTE' AND substr(c.sent_date, 1, 10) <= ?2
                 ORDER BY c.sent_date ASC LIMIT ?3",
            )
            .map_err(|error| translate_error(error, "candidatures à relancer"))?;
        let mut rows = query
            .query(rusqlite::params![today, seuil, limite.max(1)])
            .map_err(|error| translate_error(error, "candidatures à relancer"))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|error| translate_error(error, "candidatures à relancer"))?
        {
            items.push(ToFollowUp {
                id: uuid_column(row, 0)
                    .map_err(|error| translate_error(error, "candidature à relancer"))?,
                job_title: row
                    .get(1)
                    .map_err(|error| translate_error(error, "candidature à relancer"))?,
                company_name: row
                    .get(2)
                    .map_err(|error| translate_error(error, "candidature à relancer"))?,
                sent_date: row
                    .get(3)
                    .map_err(|error| translate_error(error, "candidature à relancer"))?,
                days: row
                    .get(4)
                    .map_err(|error| translate_error(error, "candidature à relancer"))?,
            });
        }
        Ok(items)
    }

    fn recent(&self, limite: u64) -> AppResult<Vec<Application>> {
        // Les candidatures récentes sont exactement celles du suivi, triées autrement : le
        // dépôt des candidatures sait déjà résoudre les jointures et les valeurs héritées,
        // et en recopier la requête ici en ferait une seconde à maintenir.
        let filter = ApplicationFilter {
            sort: ApplicationSort::Date,
            descending: true,
            ..ApplicationFilter::default()
        };
        Ok(SqliteApplicationRepository::new(self.pool.clone())
            .list_page(1, limite.max(1), &filter)?
            .items)
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
