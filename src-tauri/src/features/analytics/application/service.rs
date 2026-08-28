//! Composition des agrégats du tableau de bord et de l'écran Analytics.

use crate::core::errors::{AppError, AppResult};
use crate::features::analytics::domain::{
    Analytics, AnalyticsRepository, Step, Period, Dashboard,
};
use chrono::NaiveDate;

/// Service d'analyses, générique sur le dépôt pour rester testable sans `SQLite`.
pub struct AnalyticsService<R: AnalyticsRepository> {
    repo: R,
}

impl<R: AnalyticsRepository> AnalyticsService<R> {
    /// Construit le service autour du dépôt d'agrégats.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Payload tous les blocs du tableau de bord en un appel IPC.
    ///
    /// # Errors
    /// Propage l'erreur d'un des agrégats du dépôt.
    pub fn dashboard(&self, today: NaiveDate) -> AppResult<Dashboard> {
        let from = (today - chrono::Duration::days(30))
            .format("%Y-%m-%d")
            .to_string();
        let day = today.format("%Y-%m-%d").to_string();
        Ok(Dashboard {
            metrics: self.repo.metrics(Some(&from))?,
            performance: self.repo.performance(Some(&from))?,
            upcoming_items: self.repo.upcoming_items(&day, 6)?,
            pipeline: self.repo.pipeline()?,
            activity: self.repo.activity_hebdomadaire(8)?,
            recent: self.repo.recent(6)?,
        })
    }

    /// Payload les analyses pour la période choisie.
    ///
    /// # Errors
    /// Propage l'erreur d'un des agrégats du dépôt.
    pub fn analytics(&self, period: Period, today: NaiveDate) -> AppResult<Analytics> {
        let from = period.from(today);
        let metrics = self.repo.metrics(from.as_deref())?;
        let total = metrics.applications;
        let funnel = [
            ("Envoyées", metrics.applications),
            ("Réponses", metrics.responses),
            ("Interviews", metrics.interviews),
            ("Rejected", metrics.rejected),
        ]
        .into_iter()
        .map(|(label, count)| Step {
            label: label.into(),
            count,
            percentage: percentage(count, total),
        })
        .collect();
        let day = today.format("%Y-%m-%d").to_string();

        Ok(Analytics {
            metrics,
            performance: self.repo.performance(from.as_deref())?,
            activity: self.repo.activity_hebdomadaire(period.semaines())?,
            funnel,
            to_follow_up: self.repo.to_follow_up(&day, 7, 5)?,
        })
    }

    /// Produit un export CSV lisible dans un tableur pour la période choisie.
    ///
    /// # Errors
    /// Propage l'erreur de chargement ou une erreur de sérialisation CSV.
    pub fn export_csv(&self, period: Period, today: NaiveDate) -> AppResult<Vec<u8>> {
        let donnees = self.analytics(period, today)?;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .from_writer(vec![]);
        writer
            .write_record(["Metric", "Value"])
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        for (label, value) in [
            ("Applications", donnees.metrics.applications.to_string()),
            ("Interviews", donnees.metrics.interviews.to_string()),
            ("Réponses", donnees.metrics.responses.to_string()),
            ("Rejected", donnees.metrics.rejected.to_string()),
            (
                "Taux de réponse",
                format!("{} %", donnees.metrics.response_rate),
            ),
            (
                "Taux d'entretien",
                format!("{} %", donnees.metrics.interview_rate),
            ),
        ] {
            writer
                .write_record([label, &value])
                .map_err(|error| AppError::Serialization(error.to_string()))?;
        }
        writer
            .write_record(["", ""])
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        writer
            .write_record(["Semaine du", "Candidatures envoyées"])
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        for week in donnees.activity {
            writer
                .write_record([week.start, week.count.to_string()])
                .map_err(|error| AppError::Serialization(error.to_string()))?;
        }
        writer
            .into_inner()
            .map_err(|error| AppError::Serialization(error.to_string()))
    }
}

fn percentage(part: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part.min(total) as f64 / total as f64) * 100.0).round() as u8
}
