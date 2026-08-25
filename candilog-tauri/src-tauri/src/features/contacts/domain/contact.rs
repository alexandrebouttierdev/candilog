//! Entité et champs éditables d'un contact du réseau.

use serde::{Deserialize, Serialize};

/// Contact du réseau, tel que persisté.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "contacts.ts")]
pub struct Contact {
    /// Identifiant du contact.
    pub id: uuid::Uuid,
    /// Identifiant de l'entreprise rattachée, s'il existe.
    pub entreprise_id: Option<uuid::Uuid>,
    /// Nom de l'entreprise rattachée, aplati depuis la jointure pour l'affichage en liste.
    ///
    /// Sans lui, afficher « Nova Digital » sous chaque contact de la liste demanderait une
    /// requête par ligne, ou de charger tout le répertoire des entreprises côté React.
    pub entreprise_nom: Option<String>,
    /// Prénom (requis).
    pub prenom: String,
    /// Nom (requis).
    pub nom: String,
    /// Poste occupé, s'il est renseigné.
    pub poste: Option<String>,
    /// Rôle du contact dans le suivi de candidature — recruteur, manager, référent.
    ///
    /// Texte libre, introduit par la migration 009 pour le champ « Rôle dans le suivi » des
    /// maquettes. Absent des contacts saisis avant cette migration.
    pub role_suivi: Option<String>,
    /// Adresse e-mail, si renseignée.
    pub email: Option<String>,
    /// Téléphone, si renseigné.
    pub telephone: Option<String>,
    /// Profil `LinkedIn`, s'il est renseigné.
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
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "contacts.ts")]
pub struct NouveauContact {
    /// Identifiant de l'entreprise rattachée.
    pub entreprise_id: Option<uuid::Uuid>,
    /// Prénom (requis).
    pub prenom: String,
    /// Nom (requis).
    pub nom: String,
    /// Poste occupé.
    pub poste: Option<String>,
    /// Rôle dans le suivi de candidature.
    pub role_suivi: Option<String>,
    /// Adresse e-mail.
    pub email: Option<String>,
    /// Téléphone.
    pub telephone: Option<String>,
    /// Profil `LinkedIn`.
    pub linkedin: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

/// Édition d'un contact : remplacement complet, identique à la création.
pub type MajContact = NouveauContact;
