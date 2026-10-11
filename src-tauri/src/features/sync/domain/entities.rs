//! Cartographie des données synchronisées.
//!
//! Seules les tables listées ici quittent l'ordinateur. Restent locaux : les réglages
//! (`settings` — fournisseur IA, modèle local, chemins propres à la machine), les clés du
//! coffre système, `app_kv` (compteurs internes), les référentiels semés à l'identique sur
//! chaque installation, les modèles d'IA téléchargés et les journaux.
//!
//! L'ordre de [`SYNCED_TABLES`] est celui des dépendances : un parent précède toujours ses
//! enfants. Les créations et modifications s'appliquent dans cet ordre, les suppressions
//! dans l'ordre inverse.

/// Comportement d'une clé étrangère à la suppression du parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDelete {
    Cascade,
    SetNull,
    Restrict,
}

/// Clé étrangère vers une autre table synchronisée.
#[derive(Debug, Clone, Copy)]
pub struct ParentLink {
    pub column: &'static str,
    pub parent: &'static str,
    pub on_delete: OnDelete,
}

/// Table synchronisée : identifiant, colonnes et liens vers ses parents.
#[derive(Debug, Clone, Copy)]
pub struct SyncedTable {
    pub name: &'static str,
    /// Libellé français d'un enregistrement, pour l'écran des conflits.
    pub label: &'static str,
    /// Colonne identifiante. Toutes les tables portent un UUID texte, sauf `profile`
    /// (ligne unique d'identifiant entier 1).
    pub key: &'static str,
    pub integer_key: bool,
    /// Colonnes copiées telles quelles, identifiant compris. Un test les compare au schéma
    /// réel : une migration qui ajoute une colonne sans l'ajouter ici échoue.
    pub columns: &'static [&'static str],
    pub parents: &'static [ParentLink],
}

impl SyncedTable {
    #[must_use]
    pub fn has_column(&self, column: &str) -> bool {
        self.columns.contains(&column)
    }
}

pub const PROFILE: SyncedTable = SyncedTable {
    name: "profile",
    label: "Profil",
    key: "id",
    integer_key: true,
    columns: &["id", "data", "updated_at"],
    parents: &[],
};

pub const COMPANIES: SyncedTable = SyncedTable {
    name: "companies",
    label: "Entreprise",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "name",
        "sector_id",
        "company_type_id",
        "company_size",
        "website",
        "city",
        "address",
        "notes",
        "created_at",
        "updated_at",
    ],
    parents: &[],
};

pub const CONTACTS: SyncedTable = SyncedTable {
    name: "contacts",
    label: "Contact",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "company_id",
        "first_name",
        "name",
        "job_title",
        "email",
        "phone",
        "linkedin",
        "notes",
        "tracking_role",
        "created_at",
        "updated_at",
    ],
    parents: &[ParentLink {
        column: "company_id",
        parent: "companies",
        on_delete: OnDelete::SetNull,
    }],
};

pub const APPLICATIONS: SyncedTable = SyncedTable {
    name: "applications",
    label: "Candidature",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "company_id",
        "contact_id",
        "job_title",
        "application_type",
        "contract_type_code",
        "weekly_work_schedule",
        "weekly_hours",
        "professional_domain_id",
        "city",
        "address",
        "company_type_id",
        "status",
        "sent_date",
        "job_url",
        "notes",
        "created_at",
        "updated_at",
        "reference_number",
        "channel",
    ],
    parents: &[
        ParentLink {
            column: "company_id",
            parent: "companies",
            on_delete: OnDelete::Restrict,
        },
        ParentLink {
            column: "contact_id",
            parent: "contacts",
            on_delete: OnDelete::SetNull,
        },
    ],
};

pub const STATUS_HISTORY: SyncedTable = SyncedTable {
    name: "status_history",
    label: "Changement de statut",
    key: "id",
    integer_key: false,
    columns: &["id", "application_id", "status", "changed_at"],
    parents: &[ParentLink {
        column: "application_id",
        parent: "applications",
        on_delete: OnDelete::Cascade,
    }],
};

pub const FOLLOW_UPS: SyncedTable = SyncedTable {
    name: "follow_ups",
    label: "Relance",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "application_id",
        "follow_up_date",
        "type",
        "notes",
        "created_at",
        "done_at",
    ],
    parents: &[ParentLink {
        column: "application_id",
        parent: "applications",
        on_delete: OnDelete::Cascade,
    }],
};

pub const INTERVIEWS: SyncedTable = SyncedTable {
    name: "interviews",
    label: "Entretien",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "application_id",
        "contact_id",
        "interview_date",
        "type",
        "location",
        "notes",
        "minutes",
        "calendar_event_id",
        "ai_analysis",
        "created_at",
        "updated_at",
    ],
    parents: &[
        ParentLink {
            column: "application_id",
            parent: "applications",
            on_delete: OnDelete::Cascade,
        },
        ParentLink {
            column: "contact_id",
            parent: "contacts",
            on_delete: OnDelete::SetNull,
        },
    ],
};

pub const RELATION_NOTES: SyncedTable = SyncedTable {
    name: "relation_notes",
    label: "Note de relation",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "company_id",
        "contact_id",
        "body",
        "noted_on",
        "created_at",
    ],
    parents: &[
        ParentLink {
            column: "company_id",
            parent: "companies",
            on_delete: OnDelete::Cascade,
        },
        ParentLink {
            column: "contact_id",
            parent: "contacts",
            on_delete: OnDelete::Cascade,
        },
    ],
};

pub const RESUME_VERSIONS: SyncedTable = SyncedTable {
    name: "resume_versions",
    label: "CV",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "name",
        "content",
        "created_at",
        "document_id",
        "version_number",
        "is_current",
        "version_note",
    ],
    parents: &[],
};

pub const COVER_LETTERS: SyncedTable = SyncedTable {
    name: "cover_letters",
    label: "Lettre de motivation",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "name",
        "company",
        "job_title",
        "tone",
        "length",
        "content",
        "created_at",
        "recipient",
        "recipient_address",
        "job_reference",
        "document_id",
        "version_number",
        "is_current",
        "version_note",
    ],
    parents: &[],
};

pub const SAVED_VIEWS: SyncedTable = SyncedTable {
    name: "saved_views",
    label: "Vue enregistrée",
    key: "id",
    integer_key: false,
    columns: &[
        "id",
        "name",
        "filter",
        "position",
        "created_at",
        "updated_at",
    ],
    parents: &[],
};

/// Tables synchronisées, parents d'abord.
pub const SYNCED_TABLES: &[SyncedTable] = &[
    PROFILE,
    COMPANIES,
    CONTACTS,
    APPLICATIONS,
    STATUS_HISTORY,
    FOLLOW_UPS,
    INTERVIEWS,
    RELATION_NOTES,
    RESUME_VERSIONS,
    COVER_LETTERS,
    SAVED_VIEWS,
];

/// Table synchronisée de ce nom, s'il y en a une. Un nom inconnu reçu de l'espace distant
/// est refusé : il ne désigne jamais une table locale arbitraire.
#[must_use]
pub fn table(name: &str) -> Option<&'static SyncedTable> {
    SYNCED_TABLES.iter().find(|table| table.name == name)
}

/// Rang de dépendance d'une table (0 = sans parent).
#[must_use]
pub fn rank(name: &str) -> usize {
    SYNCED_TABLES
        .iter()
        .position(|table| table.name == name)
        .unwrap_or(usize::MAX)
}

/// Enfants d'une table : (table enfant, lien) pour chaque clé étrangère qui la vise.
pub fn children_of(
    parent: &str,
) -> impl Iterator<Item = (&'static SyncedTable, &'static ParentLink)> + '_ {
    SYNCED_TABLES.iter().flat_map(move |child| {
        child
            .parents
            .iter()
            .filter(move |link| link.parent == parent)
            .map(move |link| (child, link))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_parents_precedent_toujours_leurs_enfants() {
        for table in SYNCED_TABLES {
            for link in table.parents {
                assert!(
                    rank(link.parent) < rank(table.name),
                    "{} doit suivre {}",
                    table.name,
                    link.parent
                );
            }
        }
    }

    #[test]
    fn une_table_inconnue_est_refusee() {
        assert!(table("settings").is_none());
        assert!(table("app_kv").is_none());
        assert!(table("sqlite_master").is_none());
        assert!(table("applications").is_some());
    }

    #[test]
    fn chaque_table_declare_sa_cle_parmi_ses_colonnes() {
        for table in SYNCED_TABLES {
            assert!(table.has_column(table.key), "{}", table.name);
            for link in table.parents {
                assert!(
                    table.has_column(link.column),
                    "{}.{}",
                    table.name,
                    link.column
                );
            }
        }
    }

    #[test]
    fn les_enfants_d_une_candidature_sont_connus() {
        let enfants: Vec<_> = children_of("applications").map(|(t, _)| t.name).collect();
        assert_eq!(enfants, ["status_history", "follow_ups", "interviews"]);
    }
}
