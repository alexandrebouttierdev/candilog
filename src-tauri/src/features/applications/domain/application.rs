//! Entité et champs éditables d'une candidature.

use crate::features::applications::domain::application_type::ApplicationType;
use crate::features::applications::domain::schedule::WeeklyWorkSchedule;
use crate::features::applications::domain::status::ApplicationStatus;
use crate::features::companies::domain::CompanySize;
use serde::{Deserialize, Serialize};

/// Candidature telle que persistée, valeurs héritées et libellés résolus par jointure.
///
/// Trois champs — `city`, `address`, `company_type_id` — sont des **surcharges** : `None`
/// signifie « hériter de l'entreprise », jamais « vide ». La valeur réellement affichée est
/// la contrepartie `effective_*`, calculée par `SQLite`. L'interface déduit de la seule
/// nullité de la surcharge s'il faut signaler une valeur héritée.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct Application {
    /// Id de la candidature.
    pub id: uuid::Uuid,
    /// Intitulé du poste visé.
    pub job_title: String,
    /// Id de l'entreprise liée.
    pub company_id: uuid::Uuid,
    /// Nom de l'entreprise liée, aplati depuis la jointure ; `None` si non résolu.
    pub company_name: Option<String>,
    /// Taille de l'entreprise liée : elle appartient à l'entreprise, pas à la candidature.
    pub company_size: CompanySize,
    /// Id du contact lié, s'il existe.
    pub contact_id: Option<uuid::Uuid>,

    /// Réponse à une offre, ou démarche spontanée.
    pub application_type: ApplicationType,

    /// Code du type de contrat (référentiel `contract_types`).
    pub contract_type_code: String,
    /// Libellé français du contrat, aplati depuis la jointure sur `contract_types`.
    pub contract_type_name: Option<String>,

    /// Régime horaire hebdomadaire.
    pub weekly_work_schedule: WeeklyWorkSchedule,
    /// Volume horaire hebdomadaire, en heures par semaine.
    pub weekly_hours: Option<f64>,

    /// Domaine professionnel **du poste** (référentiel `professional_domains`).
    ///
    /// `None` signifie « non renseigné ». Il n'est jamais déduit du secteur de
    /// l'entreprise : une banque recrute aussi des informaticiens.
    pub professional_domain_id: Option<String>,
    /// Libellé du domaine professionnel, aplati depuis la jointure.
    pub professional_domain_name: Option<String>,

    /// Ville propre à la candidature ; `None` = héritée de l'entreprise.
    pub city: Option<String>,
    /// Adresse propre à la candidature ; `None` = héritée de l'entreprise.
    pub address: Option<String>,
    /// Type d'entreprise propre à la candidature ; `None` = hérité de l'entreprise.
    pub company_type_id: Option<String>,

    /// Ville effective : surcharge si elle existe, sinon celle de l'entreprise.
    pub effective_city: Option<String>,
    /// Adresse effective : surcharge si elle existe, sinon celle de l'entreprise.
    pub effective_address: Option<String>,
    /// Type d'entreprise effectif : surcharge si elle existe, sinon celui de l'entreprise.
    pub effective_company_type_id: Option<String>,
    /// Libellé du type d'entreprise effectif, aplati depuis la jointure.
    pub effective_company_type_name: Option<String>,

    /// Statut courant dans le pipeline.
    pub status: ApplicationStatus,
    /// Date d'envoi, au format `AAAA-MM-JJ`.
    pub sent_date: String,
    /// Lien vers l'offre ; toujours `None` pour une candidature spontanée.
    pub job_url: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs éditables d'une candidature, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct NewApplication {
    /// Intitulé du poste visé.
    pub job_title: String,
    /// Id de l'entreprise liée (requis).
    pub company_id: uuid::Uuid,
    /// Id du contact lié, s'il existe.
    pub contact_id: Option<uuid::Uuid>,
    /// Réponse à une offre, ou démarche spontanée.
    #[serde(default)]
    pub application_type: ApplicationType,
    /// Code du type de contrat, choisi dans le référentiel `contract_types`.
    pub contract_type_code: String,
    /// Régime horaire hebdomadaire.
    #[serde(default)]
    pub weekly_work_schedule: WeeklyWorkSchedule,
    /// Volume horaire hebdomadaire, en heures par semaine.
    pub weekly_hours: Option<f64>,
    /// Domaine professionnel du poste, choisi dans `professional_domains`.
    pub professional_domain_id: Option<String>,
    /// Surcharge de ville ; `None` laisse hériter de l'entreprise.
    ///
    /// La valeur héritée n'est **jamais** recopiée ici : elle serait figée, et changer
    /// l'entreprise de la candidature laisserait derrière elle la ville de la précédente.
    pub city: Option<String>,
    /// Surcharge d'adresse ; `None` laisse hériter de l'entreprise.
    pub address: Option<String>,
    /// Surcharge du type d'entreprise ; `None` laisse hériter de l'entreprise.
    pub company_type_id: Option<String>,
    /// Statut initial ou cible.
    pub status: ApplicationStatus,
    /// Date d'envoi choisie par l'utilisateur, au format `AAAA-MM-JJ`.
    pub sent_date: String,
    /// Lien vers l'offre ; ignoré pour une candidature spontanée.
    pub job_url: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}
