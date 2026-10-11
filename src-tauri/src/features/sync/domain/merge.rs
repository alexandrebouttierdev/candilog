//! Arbitrage d'un changement distant face à l'état local (`docs/SYNC.md`, « Conflits »).
//!
//! L'arbitrage est **colonne par colonne** et repose sur les vecteurs de versions, jamais
//! sur les horloges :
//!
//! - une colonne dont la version distante a vu la version locale prend la valeur distante,
//!   sauf si l'utilisateur l'a modifiée ici depuis — c'est alors un conflit ;
//! - une colonne dont la version locale a vu la version distante garde la valeur locale ;
//! - deux écritures parallèles de valeurs différentes sont un conflit, présenté à
//!   l'utilisateur ; aucune n'est écrasée en silence.
//!
//! Deux exceptions déterministes, symétriques sur tous les ordinateurs : `updated_at`
//! (simple horodatage technique) et `reference_number` (numéro d'affichage) prennent le
//! maximum des deux valeurs — sans quoi deux modifications de champs différents d'une même
//! fiche entreraient toujours en conflit sur leur horodatage.

use crate::features::sync::domain::entities::SyncedTable;
use crate::features::sync::domain::protocol::{RecordChange, Row};
use crate::features::sync::domain::version::{Causality, VersionVector};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Ce que l'ordinateur sait déjà d'un enregistrement synchronisé.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordMeta {
    pub version: VersionVector,
    pub columns: BTreeMap<String, VersionVector>,
    /// Contenu à la dernière synchronisation : une colonne qui en diffère a été modifiée
    /// ici et doit partir au prochain lot.
    pub base: Option<Row>,
    /// Colonnes à republier même si leur valeur n'a pas changé (résolution d'un conflit,
    /// renumérotation) : les autres ordinateurs doivent apprendre la décision.
    pub forced: BTreeSet<String>,
    pub deleted: bool,
}

/// État local d'un enregistrement au moment d'arbitrer.
#[derive(Debug, Clone, Copy)]
pub struct LocalView<'a> {
    /// Ligne actuellement en base, absente si elle n'existe pas ou plus.
    pub row: Option<&'a Row>,
    pub meta: Option<&'a RecordMeta>,
    /// Modifié localement depuis le dernier lot publié (présent dans `sync_outbox`).
    pub dirty: bool,
}

/// Désaccord sur une colonne.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConflict {
    pub column: String,
    pub remote: serde_json::Value,
    pub remote_version: VersionVector,
}

/// Nature d'un conflit d'enregistrement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "sync.ts")]
pub enum ConflictKind {
    /// Les deux ordinateurs ont modifié la même colonne.
    Edit,
    /// Supprimé sur un autre ordinateur, modifié ici.
    DeletedRemotely,
    /// Supprimé ici, modifié sur un autre ordinateur.
    DeletedLocally,
}

impl ConflictKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::DeletedRemotely => "deleted_remotely",
            Self::DeletedLocally => "deleted_locally",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "edit" => Some(Self::Edit),
            "deleted_remotely" => Some(Self::DeletedRemotely),
            "deleted_locally" => Some(Self::DeletedLocally),
            _ => None,
        }
    }
}

/// Décision prise pour un changement distant.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Déjà connu : rien à faire.
    Ignore,
    /// Écrire cette ligne et retenir cette version.
    Write { row: Row, meta: RecordMeta },
    /// Supprimer la ligne locale.
    Delete { meta: RecordMeta },
    /// Aucune ligne locale : retenir seulement le marqueur de suppression.
    RecordTombstone { meta: RecordMeta },
    /// Conflit à présenter. `partial` porte les colonnes fusionnées sans désaccord, déjà
    /// applicables ; les colonnes en conflit gardent leur valeur locale.
    Conflict {
        kind: ConflictKind,
        fields: Vec<FieldConflict>,
        partial: Option<(Row, RecordMeta)>,
    },
}

/// Colonnes arbitrées par le maximum plutôt que présentées en conflit.
fn is_max_resolved(column: &str) -> bool {
    matches!(column, "updated_at" | "reference_number")
}

fn max_value(left: &serde_json::Value, right: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if a.as_f64().unwrap_or(0.0) >= b.as_f64().unwrap_or(0.0) {
                left.clone()
            } else {
                right.clone()
            }
        }
        (Value::String(a), Value::String(b)) => {
            if a >= b {
                left.clone()
            } else {
                right.clone()
            }
        }
        (Value::Null, _) => right.clone(),
        _ => left.clone(),
    }
}

/// Version locale d'une colonne : celle retenue, sinon aucune (jamais synchronisée).
fn local_column_version(meta: Option<&RecordMeta>, column: &str) -> VersionVector {
    meta.and_then(|meta| meta.columns.get(column))
        .cloned()
        .unwrap_or_default()
}

/// Colonnes modifiées localement et pas encore publiées.
#[must_use]
pub fn pending_columns(
    table: &SyncedTable,
    row: &Row,
    meta: Option<&RecordMeta>,
) -> BTreeSet<String> {
    let null = serde_json::Value::Null;
    table
        .columns
        .iter()
        .filter(|column| **column != table.key)
        .filter(|column| {
            let Some(meta) = meta else { return true };
            if meta.forced.contains(**column) {
                return true;
            }
            meta.base.as_ref().is_none_or(|base| {
                base.get(**column).unwrap_or(&null) != row.get(**column).unwrap_or(&null)
            })
        })
        .map(|column| (*column).to_owned())
        .collect()
}

/// Arbitre un changement distant face à l'état local.
#[must_use]
pub fn decide(table: &SyncedTable, local: LocalView<'_>, remote: &RecordChange) -> Decision {
    let local_version = local
        .meta
        .map(|meta| meta.version.clone())
        .unwrap_or_default();
    if local.meta.is_some() && remote.version.is_covered_by(&local_version) {
        return Decision::Ignore;
    }
    let merged_version = local_version.merged(&remote.version);

    let Some(remote_row) = &remote.data else {
        return decide_tombstone(local, &local_version, &remote.version, merged_version);
    };

    let Some(local_row) = local.row else {
        return decide_absent(local, &local_version, merged_version, remote, remote_row);
    };

    merge_columns(table, local, local_row, merged_version, remote, remote_row)
}

fn tombstone_meta(local: LocalView<'_>, version: VersionVector) -> RecordMeta {
    RecordMeta {
        version,
        columns: BTreeMap::new(),
        base: local.meta.and_then(|meta| meta.base.clone()),
        forced: BTreeSet::new(),
        deleted: true,
    }
}

fn decide_tombstone(
    local: LocalView<'_>,
    local_version: &VersionVector,
    remote_version: &VersionVector,
    merged_version: VersionVector,
) -> Decision {
    if local.row.is_none() {
        // Supprimé des deux côtés, ou jamais connu ici : retenir le marqueur suffit.
        return Decision::RecordTombstone {
            meta: tombstone_meta(local, merged_version),
        };
    }
    let unforced = local.meta.is_some_and(|meta| meta.forced.is_empty());
    // La suppression ne s'applique que si son auteur avait vu la version locale : sinon
    // elle effacerait une modification qu'il ignorait.
    if unforced && !local.dirty && local_version.is_covered_by(remote_version) {
        return Decision::Delete {
            meta: tombstone_meta(local, merged_version),
        };
    }
    Decision::Conflict {
        kind: ConflictKind::DeletedRemotely,
        fields: Vec::new(),
        partial: None,
    }
}

fn decide_absent(
    local: LocalView<'_>,
    local_version: &VersionVector,
    merged_version: VersionVector,
    remote: &RecordChange,
    remote_row: &Row,
) -> Decision {
    let resurrect = || Decision::Write {
        row: remote_row.clone(),
        meta: RecordMeta {
            version: merged_version.clone(),
            columns: remote.columns.clone(),
            base: Some(remote_row.clone()),
            forced: BTreeSet::new(),
            deleted: false,
        },
    };
    let Some(meta) = local.meta else {
        return resurrect();
    };
    // La suppression locale a déjà été vue par l'auteur du changement distant : il a
    // délibérément recréé l'enregistrement.
    let deletion_seen = local_version.is_covered_by(&remote.version);
    if meta.deleted && !local.dirty && deletion_seen {
        return resurrect();
    }
    if !meta.deleted && !local.dirty {
        // Ligne absente sans suppression journalisée : rien à préserver ici.
        return resurrect();
    }
    Decision::Conflict {
        kind: ConflictKind::DeletedLocally,
        fields: Vec::new(),
        partial: None,
    }
}

fn merge_columns(
    table: &SyncedTable,
    local: LocalView<'_>,
    local_row: &Row,
    merged_version: VersionVector,
    remote: &RecordChange,
    remote_row: &Row,
) -> Decision {
    let null = serde_json::Value::Null;
    let pending = if local.dirty || local.meta.is_none() {
        pending_columns(table, local_row, local.meta)
    } else {
        local
            .meta
            .map(|meta| meta.forced.clone())
            .unwrap_or_default()
    };
    let mut row = local_row.clone();
    let mut meta = local.meta.cloned().unwrap_or_default();
    meta.version = merged_version;
    meta.deleted = false;
    let mut base = meta.base.clone().unwrap_or_default();
    let mut fields = Vec::new();

    for column in table.columns {
        let Some(remote_value) = remote_row.get(*column) else {
            // Colonne absente d'un objet écrit par un schéma plus ancien : la valeur locale
            // reste, rien n'indique qu'elle ait changé ailleurs.
            continue;
        };
        if *column == table.key {
            continue;
        }
        let local_value = local_row.get(*column).unwrap_or(&null);
        let remote_version = remote
            .columns
            .get(*column)
            .cloned()
            .unwrap_or_else(|| remote.version.clone());
        let local_version = local_column_version(local.meta, column);
        let is_pending = pending.contains(*column);

        let take_remote = |row: &mut Row, base: &mut Row, meta: &mut RecordMeta| {
            row.insert((*column).to_owned(), remote_value.clone());
            base.insert((*column).to_owned(), remote_value.clone());
            meta.columns
                .insert((*column).to_owned(), remote_version.clone());
        };

        let agreement = local_value == remote_value;
        match remote_version.compare(&local_version) {
            Causality::Before | Causality::Equal => {}
            Causality::After if !is_pending || agreement => {
                take_remote(&mut row, &mut base, &mut meta);
                if agreement {
                    meta.forced.remove(*column);
                }
            }
            _ if agreement => {
                // Même valeur écrite des deux côtés : rien à arbitrer.
                meta.columns
                    .insert((*column).to_owned(), local_version.merged(&remote_version));
                base.insert((*column).to_owned(), remote_value.clone());
                meta.forced.remove(*column);
            }
            _ if is_max_resolved(column) => {
                let chosen = max_value(local_value, remote_value);
                meta.columns
                    .insert((*column).to_owned(), local_version.merged(&remote_version));
                row.insert((*column).to_owned(), chosen.clone());
                base.insert((*column).to_owned(), chosen);
            }
            _ => fields.push(FieldConflict {
                column: (*column).to_owned(),
                remote: remote_value.clone(),
                remote_version,
            }),
        }
    }

    // Pour un enregistrement local jamais publié, la base part vide : seules les colonnes
    // arbitrées y entrent, les autres restent à publier.
    meta.base = Some(base);

    if fields.is_empty() {
        Decision::Write { row, meta }
    } else {
        Decision::Conflict {
            kind: ConflictKind::Edit,
            fields,
            partial: Some((row, meta)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::domain::entities::COMPANIES;

    const A: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const B: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

    fn row(name: &str, city: &str) -> Row {
        Row::from([
            ("id".into(), serde_json::json!("c1")),
            ("name".into(), serde_json::json!(name)),
            ("city".into(), serde_json::json!(city)),
            (
                "updated_at".into(),
                serde_json::json!("2026-01-01T00:00:00Z"),
            ),
        ])
    }

    fn columns(version: &VersionVector) -> BTreeMap<String, VersionVector> {
        ["name", "city", "updated_at"]
            .into_iter()
            .map(|column| (column.to_owned(), version.clone()))
            .collect()
    }

    fn synced(data: &Row, version: &VersionVector) -> RecordMeta {
        RecordMeta {
            version: version.clone(),
            columns: columns(version),
            base: Some(data.clone()),
            forced: BTreeSet::new(),
            deleted: false,
        }
    }

    fn remote(
        data: Option<Row>,
        version: VersionVector,
        cols: BTreeMap<String, VersionVector>,
    ) -> RecordChange {
        RecordChange {
            entity: "companies".into(),
            id: "c1".into(),
            version,
            columns: cols,
            data,
            attachments: vec![],
        }
    }

    #[test]
    fn un_changement_deja_connu_est_ignore() {
        let v1 = VersionVector::single(A, 1);
        let data = row("Acme", "Lyon");
        let meta = synced(&data, &v1);
        let change = remote(Some(data.clone()), v1.clone(), columns(&v1));
        let local = LocalView {
            row: Some(&data),
            meta: Some(&meta),
            dirty: false,
        };
        assert_eq!(decide(&COMPANIES, local, &change), Decision::Ignore);
    }

    #[test]
    fn un_changement_posterieur_sans_modification_locale_est_applique() {
        let v1 = VersionVector::single(A, 1);
        let data = row("Acme", "Lyon");
        let meta = synced(&data, &v1);
        let v2 = v1.merged(&VersionVector::single(B, 1));
        let mut cols = columns(&v1);
        cols.insert("city".into(), v2.clone());
        let change = remote(Some(row("Acme", "Paris")), v2.clone(), cols);
        let local = LocalView {
            row: Some(&data),
            meta: Some(&meta),
            dirty: false,
        };
        let Decision::Write { row: written, meta } = decide(&COMPANIES, local, &change) else {
            panic!("écriture attendue");
        };
        assert_eq!(written["city"], "Paris");
        assert_eq!(meta.version, v2);
        assert_eq!(meta.base.unwrap()["city"], "Paris");
    }

    #[test]
    fn deux_champs_differents_modifies_en_parallele_fusionnent_sans_conflit() {
        let v1 = VersionVector::single(A, 1);
        let base = row("Acme", "Lyon");
        let meta = synced(&base, &v1);
        // Ici : le nom a changé, pas encore publié.
        let mut local_row = row("Acme SA", "Lyon");
        local_row.insert(
            "updated_at".into(),
            serde_json::json!("2026-01-02T00:00:00Z"),
        );
        // Ailleurs : la ville a changé.
        let vb = v1.merged(&VersionVector::single(B, 1));
        let mut cols = columns(&v1);
        cols.insert("city".into(), vb.clone());
        cols.insert("updated_at".into(), vb.clone());
        let mut remote_row = row("Acme", "Paris");
        remote_row.insert(
            "updated_at".into(),
            serde_json::json!("2026-01-03T00:00:00Z"),
        );
        let change = remote(Some(remote_row), vb, cols);

        let local = LocalView {
            row: Some(&local_row),
            meta: Some(&meta),
            dirty: true,
        };
        let Decision::Write { row: merged, meta } = decide(&COMPANIES, local, &change) else {
            panic!("fusion attendue");
        };
        assert_eq!(
            merged["name"], "Acme SA",
            "la modification locale est conservée"
        );
        assert_eq!(
            merged["city"], "Paris",
            "la modification distante est appliquée"
        );
        assert_eq!(
            merged["updated_at"], "2026-01-03T00:00:00Z",
            "horodatage le plus récent"
        );
        // Le nom reste à publier.
        assert!(pending_columns(&COMPANIES, &merged, Some(&meta)).contains("name"));
        assert!(!pending_columns(&COMPANIES, &merged, Some(&meta)).contains("city"));
    }

    #[test]
    fn le_meme_champ_modifie_des_deux_cotes_est_un_conflit() {
        let v1 = VersionVector::single(A, 1);
        let base = row("Acme", "Lyon");
        let meta = synced(&base, &v1);
        let local_row = row("Acme", "Marseille");
        let vb = v1.merged(&VersionVector::single(B, 1));
        let mut cols = columns(&v1);
        cols.insert("city".into(), vb.clone());
        let change = remote(Some(row("Acme", "Paris")), vb, cols);

        let local = LocalView {
            row: Some(&local_row),
            meta: Some(&meta),
            dirty: true,
        };
        let Decision::Conflict {
            kind,
            fields,
            partial,
        } = decide(&COMPANIES, local, &change)
        else {
            panic!("conflit attendu");
        };
        assert_eq!(kind, ConflictKind::Edit);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].column, "city");
        assert_eq!(fields[0].remote, "Paris");
        let (kept, _) = partial.unwrap();
        assert_eq!(
            kept["city"], "Marseille",
            "la valeur locale n'est jamais écrasée"
        );
    }

    #[test]
    fn des_versions_publiees_en_parallele_sont_un_conflit_meme_sans_modification_en_attente() {
        // A a publié « Marseille » (A:2), B a publié « Paris » (B:1) sans voir A:2.
        let v1 = VersionVector::single(A, 1);
        let va = VersionVector::single(A, 2);
        let local_row = row("Acme", "Marseille");
        let mut meta = synced(&local_row, &va);
        meta.columns.insert("name".into(), v1.clone());
        let vb = v1.merged(&VersionVector::single(B, 1));
        let mut cols = columns(&v1);
        cols.insert("city".into(), vb.clone());
        let change = remote(Some(row("Acme", "Paris")), vb, cols);
        let local = LocalView {
            row: Some(&local_row),
            meta: Some(&meta),
            dirty: false,
        };
        assert!(matches!(
            decide(&COMPANIES, local, &change),
            Decision::Conflict {
                kind: ConflictKind::Edit,
                ..
            }
        ));
    }

    #[test]
    fn la_meme_valeur_ecrite_des_deux_cotes_n_est_pas_un_conflit() {
        let v1 = VersionVector::single(A, 1);
        let base = row("Acme", "Lyon");
        let meta = synced(&base, &v1);
        let local_row = row("Acme", "Paris");
        let vb = v1.merged(&VersionVector::single(B, 1));
        let mut cols = columns(&v1);
        cols.insert("city".into(), vb.clone());
        let change = remote(Some(row("Acme", "Paris")), vb, cols);
        let local = LocalView {
            row: Some(&local_row),
            meta: Some(&meta),
            dirty: true,
        };
        assert!(matches!(
            decide(&COMPANIES, local, &change),
            Decision::Write { .. }
        ));
    }

    #[test]
    fn une_suppression_qui_a_vu_la_version_locale_s_applique() {
        let v1 = VersionVector::single(A, 1);
        let data = row("Acme", "Lyon");
        let meta = synced(&data, &v1);
        let deletion = remote(
            None,
            v1.merged(&VersionVector::single(B, 1)),
            BTreeMap::new(),
        );
        let local = LocalView {
            row: Some(&data),
            meta: Some(&meta),
            dirty: false,
        };
        assert!(matches!(
            decide(&COMPANIES, local, &deletion),
            Decision::Delete { .. }
        ));
    }

    #[test]
    fn une_suppression_distante_d_une_fiche_modifiee_ici_est_un_conflit() {
        let v1 = VersionVector::single(A, 1);
        let data = row("Acme", "Lyon");
        let meta = synced(&data, &v1);
        let edited = row("Acme", "Paris");
        let deletion = remote(
            None,
            v1.merged(&VersionVector::single(B, 1)),
            BTreeMap::new(),
        );
        let local = LocalView {
            row: Some(&edited),
            meta: Some(&meta),
            dirty: true,
        };
        assert!(matches!(
            decide(&COMPANIES, local, &deletion),
            Decision::Conflict {
                kind: ConflictKind::DeletedRemotely,
                ..
            }
        ));
    }

    #[test]
    fn une_modification_distante_d_une_fiche_supprimee_ici_est_un_conflit() {
        let v1 = VersionVector::single(A, 1);
        let data = row("Acme", "Lyon");
        let mut meta = synced(&data, &VersionVector::single(A, 2));
        meta.deleted = true;
        let vb = v1.merged(&VersionVector::single(B, 1));
        let change = remote(Some(row("Acme", "Paris")), vb.clone(), columns(&vb));
        let local = LocalView {
            row: None,
            meta: Some(&meta),
            dirty: false,
        };
        assert!(matches!(
            decide(&COMPANIES, local, &change),
            Decision::Conflict {
                kind: ConflictKind::DeletedLocally,
                ..
            }
        ));
    }

    #[test]
    fn une_suppression_ne_fait_pas_reapparaitre_une_version_anterieure() {
        // Marqueur A:2 connu ; une version A:1 rejouée ne ressuscite rien.
        let data = row("Acme", "Lyon");
        let mut meta = synced(&data, &VersionVector::single(A, 2));
        meta.deleted = true;
        let v1 = VersionVector::single(A, 1);
        let stale = remote(Some(data), v1.clone(), columns(&v1));
        let local = LocalView {
            row: None,
            meta: Some(&meta),
            dirty: false,
        };
        assert_eq!(decide(&COMPANIES, local, &stale), Decision::Ignore);
    }

    #[test]
    fn une_recreation_qui_a_vu_la_suppression_ressuscite_la_fiche() {
        let data = row("Acme", "Lyon");
        let mut meta = synced(&data, &VersionVector::single(A, 2));
        meta.deleted = true;
        let vb = VersionVector::single(A, 2).merged(&VersionVector::single(B, 1));
        let change = remote(Some(row("Acme", "Paris")), vb.clone(), columns(&vb));
        let local = LocalView {
            row: None,
            meta: Some(&meta),
            dirty: false,
        };
        assert!(matches!(
            decide(&COMPANIES, local, &change),
            Decision::Write { .. }
        ));
    }

    #[test]
    fn une_fiche_inconnue_est_creee() {
        let v1 = VersionVector::single(B, 1);
        let change = remote(Some(row("Acme", "Lyon")), v1.clone(), columns(&v1));
        let local = LocalView {
            row: None,
            meta: None,
            dirty: false,
        };
        let Decision::Write { meta, .. } = decide(&COMPANIES, local, &change) else {
            panic!("création attendue");
        };
        assert!(!meta.deleted);
        assert_eq!(meta.version, v1);
    }

    #[test]
    fn une_fiche_locale_jamais_publiee_differente_est_un_conflit_de_fusion() {
        let v1 = VersionVector::single(B, 1);
        let local_row = row("Acme", "Lyon");
        let change = remote(Some(row("Acme", "Paris")), v1.clone(), columns(&v1));
        let local = LocalView {
            row: Some(&local_row),
            meta: None,
            dirty: true,
        };
        let Decision::Conflict { fields, .. } = decide(&COMPANIES, local, &change) else {
            panic!("conflit attendu");
        };
        assert_eq!(
            fields.iter().map(|f| f.column.as_str()).collect::<Vec<_>>(),
            ["city"]
        );
    }

    #[test]
    fn la_decision_est_symetrique_pour_les_horodatages() {
        let base = VersionVector::single(A, 1);
        let va = base.merged(&VersionVector::single(A, 2));
        let vb = base.merged(&VersionVector::single(B, 1));
        let mut left = row("Acme", "Lyon");
        left.insert(
            "updated_at".into(),
            serde_json::json!("2026-02-01T00:00:00Z"),
        );
        let mut right = row("Acme", "Lyon");
        right.insert(
            "updated_at".into(),
            serde_json::json!("2026-03-01T00:00:00Z"),
        );
        let meta_a = synced(&left, &va);
        let meta_b = synced(&right, &vb);
        let on_a = decide(
            &COMPANIES,
            LocalView {
                row: Some(&left),
                meta: Some(&meta_a),
                dirty: false,
            },
            &remote(Some(right.clone()), vb.clone(), columns(&vb)),
        );
        let on_b = decide(
            &COMPANIES,
            LocalView {
                row: Some(&right),
                meta: Some(&meta_b),
                dirty: false,
            },
            &remote(Some(left.clone()), va.clone(), columns(&va)),
        );
        let (Decision::Write { row: ra, meta: ma }, Decision::Write { row: rb, meta: mb }) =
            (on_a, on_b)
        else {
            panic!("fusions attendues");
        };
        assert_eq!(ra, rb);
        assert_eq!(ma.columns, mb.columns);
        assert_eq!(ma.version, mb.version);
    }
}
