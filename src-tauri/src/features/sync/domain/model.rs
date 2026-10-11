//! Contrat IPC de la synchronisation : état affiché, choix de l'utilisateur, conflits.

use crate::features::sync::domain::merge::ConflictKind;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Mode de stockage choisi. `Local` est le mode par défaut : aucune donnée ne quitte
/// l'ordinateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub enum SyncProviderKind {
    #[default]
    Local,
    GoogleDrive,
    OneDrive,
    Folder,
}

impl SyncProviderKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::GoogleDrive => "google_drive",
            Self::OneDrive => "one_drive",
            Self::Folder => "folder",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "local" => Some(Self::Local),
            "google_drive" => Some(Self::GoogleDrive),
            "one_drive" => Some(Self::OneDrive),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Local => "Stockage local",
            Self::GoogleDrive => "Google Drive",
            Self::OneDrive => "Microsoft OneDrive",
            Self::Folder => "Dossier personnalisé",
        }
    }
}

/// État réel de la synchronisation, tel que l'interface l'affiche.
///
/// `UpToDate` n'est annoncé que lorsque plus aucune modification locale n'attend et que le
/// fournisseur a confirmé l'écriture du dernier lot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub enum SyncPhase {
    /// Mode local : aucun espace distant.
    NotConfigured,
    /// Espace rattaché, synchronisation automatique suspendue par l'utilisateur.
    Disabled,
    UpToDate,
    Syncing,
    /// Modifications locales pas encore confirmées par le fournisseur.
    Pending,
    Offline,
    /// Session du fournisseur expirée ou révoquée.
    AuthRequired,
    /// Clé de l'espace absente de ce poste : phrase secrète ou clé de récupération requise.
    KeyRequired,
    TemporaryError,
    Conflict,
    Restoring,
    /// L'espace a été écrit par une version plus récente de Candilog.
    Incompatible,
}

/// Instantané de l'état de la synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct SyncStatus {
    pub provider: SyncProviderKind,
    pub phase: SyncPhase,
    pub auto_sync: bool,
    /// Emplacement lisible : chemin du dossier, ou nom du service.
    pub location: Option<String>,
    pub pending_changes: u32,
    pub conflicts: u32,
    /// Changements reçus en attente d'un enregistrement parent pas encore arrivé.
    pub waiting_changes: u32,
    pub missing_files: u32,
    /// Ordinateurs ayant déjà publié dans l'espace, celui-ci compris.
    pub computers: u32,
    pub last_success_at: Option<String>,
    pub last_attempt_at: Option<String>,
    pub next_attempt_at: Option<String>,
    /// Explication en français de l'état, et de l'action attendue s'il y en a une.
    pub message: Option<String>,
}

impl SyncStatus {
    /// État du mode local.
    #[must_use]
    pub const fn local() -> Self {
        Self {
            provider: SyncProviderKind::Local,
            phase: SyncPhase::NotConfigured,
            auto_sync: false,
            location: None,
            pending_changes: 0,
            conflicts: 0,
            waiting_changes: 0,
            missing_files: 0,
            computers: 0,
            last_success_at: None,
            last_attempt_at: None,
            next_attempt_at: None,
            message: None,
        }
    }
}

/// Fournisseurs utilisables par cette version de Candilog.
///
/// Google Drive et OneDrive exigent un identifiant d'application OAuth, fourni à la
/// compilation : une version construite sans lui ne les propose pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct ProviderAvailability {
    pub google_drive: bool,
    pub one_drive: bool,
}

/// Résultat de la connexion à un fournisseur, avant tout échange de données.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct RemoteProbe {
    pub provider: SyncProviderKind,
    pub location: String,
    /// Un espace Candilog existe déjà à cet emplacement.
    pub space_exists: bool,
    pub space_created_at: Option<String>,
    /// Cet ordinateur contient déjà des données (candidatures, profil, documents…).
    pub local_has_data: bool,
}

/// Scénario de première synchronisation, toujours choisi explicitement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub enum SetupChoice {
    /// Créer un espace vide puis y envoyer les données de cet ordinateur.
    CreateSpace,
    /// Rejoindre l'espace et fusionner avec les données locales.
    JoinMerge,
    /// Rejoindre l'espace et remplacer les données locales (sauvegarde prise avant).
    JoinReplaceLocal,
}

/// Demande de mise en service de la synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct SetupRequest {
    pub choice: SetupChoice,
    pub passphrase: Option<String>,
    pub recovery_key: Option<String>,
}

/// Issue de la mise en service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct SetupOutcome {
    /// Clé de récupération, affichée **une seule fois** à la création de l'espace.
    pub recovery_key: Option<String>,
    /// Sauvegarde prise avant de remplacer les données locales.
    pub backup_file: Option<String>,
}

/// Valeur d'une colonne en conflit, des deux côtés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct ConflictField {
    pub column: String,
    pub label: String,
    pub local: Option<String>,
    pub remote: Option<String>,
}

/// Conflit présenté à l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub struct SyncConflict {
    pub id: String,
    pub entity: String,
    pub entity_label: String,
    pub record_label: String,
    pub kind: ConflictKind,
    pub detected_at: String,
    pub fields: Vec<ConflictField>,
}

/// Décision de l'utilisateur sur un conflit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub enum ConflictChoice {
    /// Garder la version de cet ordinateur (ou confirmer sa suppression).
    KeepLocal,
    /// Prendre la version de l'autre ordinateur (ou sa suppression).
    TakeRemote,
}

/// Libellé français d'une colonne, pour l'écran des conflits.
#[must_use]
pub fn column_label(column: &str) -> &str {
    match column {
        "name" => "Nom",
        "first_name" => "Prénom",
        "job_title" => "Poste",
        "status" => "Statut",
        "notes" => "Notes",
        "city" => "Ville",
        "address" => "Adresse",
        "website" => "Site web",
        "email" => "E-mail",
        "phone" => "Téléphone",
        "linkedin" => "LinkedIn",
        "company_id" => "Entreprise",
        "contact_id" => "Contact",
        "sent_date" => "Date d'envoi",
        "job_url" => "Lien de l'offre",
        "channel" => "Canal",
        "contract_type_code" => "Type de contrat",
        "follow_up_date" => "Date de relance",
        "done_at" => "Relance faite le",
        "interview_date" => "Date de l'entretien",
        "location" => "Lieu",
        "minutes" => "Compte rendu",
        "content" => "Contenu",
        "body" => "Note",
        "data" => "Contenu du profil",
        "filter" => "Filtre",
        "is_current" => "Version courante",
        "type" => "Type",
        _ => column,
    }
}
