//! Metrics affichés par le tableau de bord et par les analyses.

use crate::features::applications::domain::Application;
use serde::Serialize;

/// Compteur assorti de sa part du total.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct Step {
    /// Libellé de l'étape.
    pub label: String,
    /// Count de candidatures ayant atteint l'étape.
    #[ts(type = "number")]
    pub count: u64,
    /// Part du total, en pourcentage entier.
    #[ts(type = "number")]
    pub percentage: u8,
}

/// Count de candidatures envoyées sur une semaine.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct ActivityWeek {
    /// Début de la fenêtre glissante de sept jours (`AAAA-MM-JJ`).
    pub start: String,
    /// Count de candidatures envoyées cette semaine-là.
    #[ts(type = "number")]
    pub count: u64,
}

/// Application sans réponse depuis un certain temps.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct ToFollowUp {
    /// Id de la candidature, pour ouvrir sa fiche.
    pub id: uuid::Uuid,
    /// Intitulé du poste.
    pub job_title: String,
    /// Name de l'entreprise.
    pub company_name: Option<String>,
    /// Date d'envoi (`AAAA-MM-JJ`).
    pub sent_date: String,
    /// Days écoulés depuis l'envoi.
    #[ts(type = "number")]
    pub days: u64,
}

/// Metrics chiffrés communs aux deux écrans.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct Metrics {
    /// Applications envoyées sur la période.
    #[ts(type = "number")]
    pub applications: u64,
    /// Applications ayant atteint l'étape entretien, refusées ensuite comprises.
    #[ts(type = "number")]
    pub interviews: u64,
    /// Applications ayant reçu une réponse, favorable ou non.
    #[ts(type = "number")]
    pub responses: u64,
    /// Applications refusées.
    #[ts(type = "number")]
    pub rejected: u64,
    /// Applications encore en attente de réponse.
    #[ts(type = "number")]
    pub pending: u64,
    /// Applications relancées.
    #[ts(type = "number")]
    pub followed_up: u64,
    /// Part des candidatures ayant reçu une réponse, en pourcentage entier.
    #[ts(type = "number")]
    pub response_rate: u8,
    /// Part des candidatures ayant atteint l'entretien, en pourcentage entier.
    #[ts(type = "number")]
    pub interview_rate: u8,
}

/// Metrics de rythme et de délai.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct Performance {
    /// Délai moyen en jours entre l'envoi et la première réponse ; `None` sans réponse.
    #[ts(type = "number | null")]
    pub average_response_days: Option<u64>,
    /// Applications envoyées par semaine, à une décimale près.
    pub applications_per_week: f64,
    /// Interviews à venir, tous horizons confondus.
    #[ts(type = "number")]
    pub upcoming_interviews: u64,
    /// FollowUps programmées dont la date est passée.
    #[ts(type = "number")]
    pub overdue_follow_ups: u64,
}

/// Événement à venir affiché par le tableau de bord.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct UpcomingItem {
    /// Id de l'entretien ou de la relance.
    pub id: uuid::Uuid,
    /// `entretien` ou `relance`.
    pub kind: String,
    /// Date ou horodatage de l'événement.
    pub date: String,
    /// Intitulé du poste concerné.
    pub job_title: Option<String>,
    /// Name de l'entreprise concernée.
    pub company_name: Option<String>,
    /// Format d'entretien ou canal de relance.
    pub detail: String,
}

/// Payload utile du tableau de bord.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct Dashboard {
    /// Metrics sur trente jours.
    pub metrics: Metrics,
    /// Rythme et délais.
    pub performance: Performance,
    /// Prochains entretiens et relances, les plus proches d'abord.
    pub upcoming_items: Vec<UpcomingItem>,
    /// Répartition du pipeline, toutes périodes confondues.
    pub pipeline: Vec<Step>,
    /// Activité hebdomadaire des huit dernières semaines.
    pub activity: Vec<ActivityWeek>,
    /// Applications les plus récentes.
    pub recent: Vec<Application>,
}

/// Payload utile de l'écran Analytics.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "analytics.ts")]
pub struct Analytics {
    /// Metrics sur la période demandée.
    pub metrics: Metrics,
    /// Rythme et délais sur la période.
    pub performance: Performance,
    /// Activité hebdomadaire sur la période.
    pub activity: Vec<ActivityWeek>,
    /// Funnel de conversion : envoyées, réponses, entretiens, refus.
    pub funnel: Vec<Step>,
    /// Applications sans réponse, les plus anciennes d'abord.
    pub to_follow_up: Vec<ToFollowUp>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_json_expose_upcoming_interviews() {
        let json = serde_json::to_value(Performance {
            upcoming_interviews: 3,
            ..Performance::default()
        })
        .unwrap();
        assert_eq!(json["upcoming_interviews"], 3);
        assert!(json.get("upcomingInterviews").is_none());
    }
}
