//! Indicateurs affichés par le tableau de bord et par les analyses.

use crate::features::candidatures::domain::Candidature;
use serde::Serialize;

/// Compteur assorti de sa part du total.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct Etape {
    /// Libellé de l'étape.
    pub label: String,
    /// Nombre de candidatures ayant atteint l'étape.
    #[ts(type = "number")]
    pub nombre: u64,
    /// Part du total, en pourcentage entier.
    #[ts(type = "number")]
    pub pourcentage: u8,
}

/// Nombre de candidatures envoyées sur une semaine.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct SemaineActivite {
    /// Début de la fenêtre glissante de sept jours (`AAAA-MM-JJ`).
    pub debut: String,
    /// Nombre de candidatures envoyées cette semaine-là.
    #[ts(type = "number")]
    pub nombre: u64,
}

/// Candidature sans réponse depuis un certain temps.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct ARelancer {
    /// Identifiant de la candidature, pour ouvrir sa fiche.
    pub id: uuid::Uuid,
    /// Intitulé du poste.
    pub poste: String,
    /// Nom de l'entreprise.
    pub entreprise_nom: Option<String>,
    /// Date d'envoi (`AAAA-MM-JJ`).
    pub date_envoi: String,
    /// Jours écoulés depuis l'envoi.
    #[ts(type = "number")]
    pub jours: u64,
}

/// Indicateurs chiffrés communs aux deux écrans.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct Indicateurs {
    /// Candidatures envoyées sur la période.
    #[ts(type = "number")]
    pub candidatures: u64,
    /// Candidatures ayant atteint l'étape entretien, refusées ensuite comprises.
    #[ts(type = "number")]
    pub entretiens: u64,
    /// Candidatures ayant reçu une réponse, favorable ou non.
    #[ts(type = "number")]
    pub reponses: u64,
    /// Candidatures refusées.
    #[ts(type = "number")]
    pub refus: u64,
    /// Candidatures encore en attente de réponse.
    #[ts(type = "number")]
    pub en_attente: u64,
    /// Candidatures relancées.
    #[ts(type = "number")]
    pub relancees: u64,
    /// Part des candidatures ayant reçu une réponse, en pourcentage entier.
    #[ts(type = "number")]
    pub taux_reponse: u8,
    /// Part des candidatures ayant atteint l'entretien, en pourcentage entier.
    #[ts(type = "number")]
    pub taux_entretien: u8,
}

/// Indicateurs de rythme et de délai.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct Performance {
    /// Délai moyen en jours entre l'envoi et la première réponse ; `None` sans réponse.
    #[ts(type = "number | null")]
    pub delai_moyen_reponse: Option<u64>,
    /// Candidatures envoyées par semaine, à une décimale près.
    pub candidatures_par_semaine: f64,
    /// Entretiens à venir, tous horizons confondus.
    #[ts(type = "number")]
    pub entretiens_a_venir: u64,
    /// Relances programmées dont la date est passée.
    #[ts(type = "number")]
    pub relances_en_retard: u64,
}

/// Événement à venir affiché par le tableau de bord.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct Echeance {
    /// Identifiant de l'entretien ou de la relance.
    pub id: uuid::Uuid,
    /// `entretien` ou `relance`.
    pub genre: String,
    /// Date ou horodatage de l'événement.
    pub date: String,
    /// Intitulé du poste concerné.
    pub poste: Option<String>,
    /// Nom de l'entreprise concernée.
    pub entreprise_nom: Option<String>,
    /// Format d'entretien ou canal de relance.
    pub detail: String,
}

/// Charge utile du tableau de bord.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct TableauDeBord {
    /// Indicateurs sur trente jours.
    pub indicateurs: Indicateurs,
    /// Rythme et délais.
    pub performance: Performance,
    /// Prochains entretiens et relances, les plus proches d'abord.
    pub echeances: Vec<Echeance>,
    /// Répartition du pipeline, toutes périodes confondues.
    pub pipeline: Vec<Etape>,
    /// Activité hebdomadaire des huit dernières semaines.
    pub activite: Vec<SemaineActivite>,
    /// Candidatures les plus récentes.
    pub recentes: Vec<Candidature>,
}

/// Charge utile de l'écran Analyses.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "analyses.ts")]
pub struct Analyses {
    /// Indicateurs sur la période demandée.
    pub indicateurs: Indicateurs,
    /// Rythme et délais sur la période.
    pub performance: Performance,
    /// Activité hebdomadaire sur la période.
    pub activite: Vec<SemaineActivite>,
    /// Entonnoir de conversion : envoyées, réponses, entretiens, refus.
    pub entonnoir: Vec<Etape>,
    /// Candidatures sans réponse, les plus anciennes d'abord.
    pub a_relancer: Vec<ARelancer>,
}
