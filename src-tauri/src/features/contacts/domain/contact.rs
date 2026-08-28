//! Entité et champs éditables d'un contact du réseau.

use serde::{Deserialize, Serialize};

/// Contact du réseau, tel que persisté.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "contacts.ts")]
pub struct Contact {
    /// Id du contact.
    pub id: uuid::Uuid,
    /// Id de l'entreprise rattachée, s'il existe.
    pub company_id: Option<uuid::Uuid>,
    /// Name de l'entreprise rattachée, aplati depuis la jointure pour l'affichage en liste.
    ///
    /// Sans lui, afficher « Nova Digital » sous chaque contact de la liste demanderait une
    /// requête par ligne, ou de charger tout le répertoire des entreprises côté React.
    pub company_name: Option<String>,
    /// Prénom (requis).
    pub first_name: String,
    /// Name (requis).
    pub name: String,
    /// JobTitle occupé, s'il est renseigné.
    pub job_title: Option<String>,
    /// Rôle du contact dans le suivi de candidature — recruteur, manager, référent.
    ///
    /// Text libre, introduit par la migration 009 pour le champ « Rôle dans le suivi » des
    /// maquettes. Absent des contacts saisis avant cette migration.
    pub tracking_role: Option<String>,
    /// Address e-mail, si renseignée.
    pub email: Option<String>,
    /// Téléphone, si renseigné.
    pub phone: Option<String>,
    /// Profile `LinkedIn`, s'il est renseigné.
    pub linkedin: Option<String>,
    /// Notes libres, si renseignées.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs de création et d'édition d'un contact : prénom et nom requis.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "contacts.ts")]
pub struct NewContact {
    /// Id de l'entreprise rattachée.
    pub company_id: Option<uuid::Uuid>,
    /// Prénom (requis).
    pub first_name: String,
    /// Name (requis).
    pub name: String,
    /// JobTitle occupé.
    pub job_title: Option<String>,
    /// Rôle dans le suivi de candidature.
    pub tracking_role: Option<String>,
    /// Address e-mail.
    pub email: Option<String>,
    /// Téléphone.
    pub phone: Option<String>,
    /// Profile `LinkedIn`.
    pub linkedin: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

/// Édition d'un contact : remplacement complet, identique à la création.
pub type ContactUpdate = NewContact;
