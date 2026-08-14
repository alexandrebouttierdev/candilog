//! Types de navigation interne et de dialogue.

/// Formulaire ou dialogue actuellement ouvert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    /// Création d'une entreprise.
    Entreprise,
    /// Création d'un contact.
    Contact,
    /// Création d'une candidature.
    Candidature,
    /// Création d'un entretien.
    Entretien,
    /// Création d'une relance.
    Relance,
    /// Édition d'une seule section du profil.
    Profil(ProfileSection),
    /// Validation des informations extraites d'un CV.
    ProfileImport,
    /// Confirmation de suppression d'une candidature.
    DeleteCandidature(uuid::Uuid),
    /// Confirmation de suppression d'une entreprise.
    DeleteEntreprise(uuid::Uuid),
    /// Confirmation de suppression d'un contact.
    DeleteContact(uuid::Uuid),
    /// Confirmation de suppression d'un entretien.
    DeleteEntretien(uuid::Uuid),
    /// Confirmation de suppression d'une relance.
    DeleteRelance(uuid::Uuid),
    /// Confirmation de suppression d'une version de CV.
    DeleteCv(uuid::Uuid),
    /// Confirmation de restauration d'un backup.
    ImportBackup,
    /// Confirmation de réinitialisation complète.
    ResetDatabase,
    /// Confirmation de purge du cache IA local.
    ResetAiCache,
    /// Détail relationnel d'une candidature.
    CandidatureDetail(uuid::Uuid),
}

/// Champ auquel le calendrier flottant doit appliquer la date choisie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerTarget {
    Candidature,
    Entretien,
    Relance,
    FiltreDebut,
    FiltreFin,
}

#[derive(Debug, Clone, Copy)]
pub struct DatePickerState {
    pub target: DatePickerTarget,
    pub year: i32,
    pub month: u32,
}

/// Collections structurées éditables dans le profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileCollection {
    Experience,
    Formation,
    Langue,
    Projet,
    Certification,
}

/// Partie du profil affichée dans la modale d'édition courante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSection {
    Identite,
    Competences,
    Collection(ProfileCollection),
}
