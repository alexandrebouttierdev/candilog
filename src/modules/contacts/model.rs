//! Types du domaine des contacts (réseau).

use serde::{Deserialize, Serialize};

/// Contact du réseau de l'utilisateur, tel que persisté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Identifiant du contact.
    pub id: uuid::Uuid,
    /// Identifiant de l'entreprise rattachée (FK `entreprises`), s'il existe.
    pub entreprise_id: Option<uuid::Uuid>,
    /// Prénom (requis).
    pub prenom: String,
    /// Nom (requis).
    pub nom: String,
    /// Poste occupé, s'il est renseigné.
    pub poste: Option<String>,
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

/// Champs de création/édition d'un contact (`prenom` et `nom` requis, reste optionnel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NouveauContact {
    /// Identifiant de l'entreprise rattachée (optionnel).
    pub entreprise_id: Option<uuid::Uuid>,
    /// Prénom (requis).
    pub prenom: String,
    /// Nom (requis).
    pub nom: String,
    /// Poste occupé.
    pub poste: Option<String>,
    /// Adresse e-mail.
    pub email: Option<String>,
    /// Téléphone.
    pub telephone: Option<String>,
    /// Profil `LinkedIn`.
    pub linkedin: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

/// Champs d'édition d'un contact (remplacement complet, identique à la création).
pub type MajContact = NouveauContact;
