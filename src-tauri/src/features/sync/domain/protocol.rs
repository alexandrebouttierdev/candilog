//! Format des objets écrits dans l'espace distant (`docs/SYNC.md`, « Protocole »).
//!
//! L'espace distant n'est qu'un ensemble de fichiers **immuables** : chaque installation
//! n'écrit que ses propres lots, sous un nom qu'elle seule utilise, et ne réécrit jamais un
//! fichier existant. Aucun fournisseur n'a donc à offrir d'écriture conditionnelle, et deux
//! ordinateurs ne peuvent pas s'écraser mutuellement — un dossier Dropbox ou iCloud se
//! comporte comme une API cloud.
//!
//! ```text
//! space.json                                  manifeste en clair, sans donnée personnelle
//! changes/<installation>/<seq:012>.batch      lot de changements chiffré
//! snapshots/<installation>/<seq:012>.snap     instantané complet chiffré
//! blobs/<empreinte>.blob                      fichier joint chiffré (photo du profil)
//! ```

use crate::features::sync::domain::entities::{self, SyncedTable};
use crate::features::sync::domain::version::VersionVector;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Version du format des objets distants. Un objet d'un format supérieur est refusé : il
/// vient d'une version plus récente de Candilog.
pub const FORMAT_VERSION: u32 = 1;

/// Nom du manifeste de l'espace.
pub const MANIFEST_NAME: &str = "space.json";
pub const CHANGES_PREFIX: &str = "changes/";
pub const SNAPSHOTS_PREFIX: &str = "snapshots/";
pub const BLOBS_PREFIX: &str = "blobs/";

/// Plafonds de validation d'un objet reçu. Très au-delà d'un usage réel, ils empêchent un
/// objet forgé d'épuiser la mémoire ou le disque.
pub const MAX_CHANGES_PER_OBJECT: usize = 200_000;
pub const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_ATTACHMENT_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_OBJECT_BYTES: usize = 512 * 1024 * 1024;

/// Contenu d'un enregistrement : colonne → valeur scalaire JSON.
pub type Row = BTreeMap<String, serde_json::Value>;

/// Fichier joint à un enregistrement, transmis à part dans `blobs/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// Nom local du fichier, sous le dossier des photos.
    pub file: String,
    /// Nom de l'objet distant : empreinte chiffrée par clé, qui ne révèle pas le contenu.
    pub blob: String,
    /// SHA-256 du contenu en clair, vérifié après déchiffrement.
    pub sha256: String,
    pub size: u64,
}

/// État publié d'un enregistrement : son contenu, ou sa suppression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordChange {
    pub entity: String,
    pub id: String,
    /// Version de l'enregistrement : union des versions de ses colonnes, ou version de la
    /// suppression.
    pub version: VersionVector,
    /// Version de chaque colonne : l'arbitrage se fait colonne par colonne, si bien que deux
    /// ordinateurs qui modifient deux champs différents d'une même fiche ne sont pas en
    /// conflit.
    #[serde(default)]
    pub columns: BTreeMap<String, VersionVector>,
    /// `None` : l'enregistrement a été supprimé (marqueur de suppression).
    pub data: Option<Row>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

impl RecordChange {
    #[must_use]
    pub const fn is_tombstone(&self) -> bool {
        self.data.is_none()
    }
}

/// Lot de changements publié par une installation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeBatch {
    pub format: u32,
    pub space_id: String,
    pub installation_id: String,
    pub seq: u64,
    pub schema_version: i64,
    pub created_at: String,
    pub changes: Vec<RecordChange>,
}

/// Instantané complet : tout ce qu'une installation connaissait à un instant donné.
///
/// `covered` dit quels lots de chaque installation il incorpore. Un nouvel ordinateur part
/// de l'instantané le plus complet puis rejoue les lots suivants ; un ordinateur resté
/// hors ligne au-delà de la rétention des lots s'en sert pour se remettre à niveau.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub format: u32,
    pub space_id: String,
    pub installation_id: String,
    pub seq: u64,
    pub schema_version: i64,
    pub created_at: String,
    pub covered: VersionVector,
    pub records: Vec<RecordChange>,
}

/// Paramètres de dérivation de la clé de chiffrement depuis la phrase secrète.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub algorithm: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    /// Sel aléatoire, en base64.
    pub salt: String,
}

/// Manifeste d'un espace de synchronisation, écrit une seule fois à sa création.
///
/// Il est en clair parce qu'il faut le lire avant de connaître la clé ; il ne contient que
/// des paramètres techniques et la clé de l'espace **chiffrée** par la phrase secrète.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceManifest {
    pub format: u32,
    pub app: String,
    pub space_id: String,
    pub created_at: String,
    pub kdf: KdfParams,
    /// Clé de l'espace chiffrée par la clé dérivée de la phrase secrète (base64).
    pub wrapped_key: String,
    /// Témoin chiffré par la clé de l'espace : vérifie une clé de récupération saisie.
    pub key_check: String,
}

/// Erreur de validation d'un objet reçu : il est refusé en entier.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct InvalidObject(pub String);

fn invalid(message: impl Into<String>) -> InvalidObject {
    InvalidObject(message.into())
}

/// Identifiant d'installation ou d'espace : UUID en minuscules.
#[must_use]
pub fn is_valid_identifier(value: &str) -> bool {
    value.len() == 36 && uuid::Uuid::parse_str(value).is_ok() && value == value.to_lowercase()
}

/// Nom de l'objet d'un lot.
#[must_use]
pub fn batch_name(installation_id: &str, seq: u64) -> String {
    format!("{CHANGES_PREFIX}{installation_id}/{seq:012}.batch")
}

/// Nom de l'objet d'un instantané.
#[must_use]
pub fn snapshot_name(installation_id: &str, seq: u64) -> String {
    format!("{SNAPSHOTS_PREFIX}{installation_id}/{seq:012}.snap")
}

/// Nom de l'objet d'un fichier joint.
#[must_use]
pub fn blob_name(blob: &str) -> String {
    format!("{BLOBS_PREFIX}{blob}.blob")
}

/// Type d'objet reconnu dans un nom distant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectKind {
    Batch { installation_id: String, seq: u64 },
    Snapshot { installation_id: String, seq: u64 },
    Blob { blob: String },
}

/// Analyse un nom d'objet. Tout nom inattendu — copie de conflit créée par un outil de
/// synchronisation, fichier temporaire, ajout manuel — est ignoré plutôt que lu.
#[must_use]
pub fn parse_object_name(name: &str) -> Option<ObjectKind> {
    let numbered = |rest: &str, extension: &str| -> Option<(String, u64)> {
        let (installation, file) = rest.split_once('/')?;
        let digits = file.strip_suffix(extension)?;
        if !is_valid_identifier(installation)
            || digits.len() != 12
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        let seq = digits.parse().ok()?;
        (seq > 0).then(|| (installation.to_owned(), seq))
    };
    if let Some(rest) = name.strip_prefix(CHANGES_PREFIX) {
        let (installation_id, seq) = numbered(rest, ".batch")?;
        return Some(ObjectKind::Batch {
            installation_id,
            seq,
        });
    }
    if let Some(rest) = name.strip_prefix(SNAPSHOTS_PREFIX) {
        let (installation_id, seq) = numbered(rest, ".snap")?;
        return Some(ObjectKind::Snapshot {
            installation_id,
            seq,
        });
    }
    let blob = name.strip_prefix(BLOBS_PREFIX)?.strip_suffix(".blob")?;
    is_hex_digest(blob).then(|| ObjectKind::Blob {
        blob: blob.to_owned(),
    })
}

/// Empreinte SHA-256 en hexadécimal minuscule.
#[must_use]
pub fn is_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Nom d'une photo de profil tel que `profile::domain::photo` le génère. Un nom reçu de
/// l'espace distant devient un chemin local : rien d'autre que cette forme n'est accepté.
#[must_use]
pub fn is_valid_photo_file(name: &str) -> bool {
    name.strip_prefix("photo-")
        .and_then(|rest| rest.strip_suffix(".png"))
        .is_some_and(|uuid| uuid::Uuid::parse_str(uuid).is_ok() && uuid.len() == 36)
}

fn validate_version(version: &VersionVector, what: &str) -> Result<(), InvalidObject> {
    // 2^53 : au-delà, un compteur ne survit plus à un aller-retour par un nombre JSON.
    const MAX_SEQ: u64 = 1 << 53;
    for (installation, seq) in version.entries() {
        if !is_valid_identifier(installation) || *seq == 0 || *seq > MAX_SEQ {
            return Err(invalid(format!("version invalide ({what})")));
        }
    }
    Ok(())
}

/// Identifiant d'enregistrement attendu pour une table.
fn validate_record_id(table: &SyncedTable, id: &str) -> Result<(), InvalidObject> {
    let valid = if table.integer_key {
        id == "1"
    } else {
        !id.is_empty()
            && id.len() <= 64
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    };
    if valid {
        Ok(())
    } else {
        Err(invalid(format!("identifiant invalide pour {}", table.name)))
    }
}

/// Valeur de la clé d'un enregistrement, telle que portée par son contenu.
#[must_use]
pub fn key_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Number(number) => number.as_i64().map(|value| value.to_string()),
        _ => None,
    }
}

/// Valide un changement reçu : table connue, identifiant, colonnes, valeurs et versions.
///
/// # Errors
/// [`InvalidObject`] dès qu'un élément sort du format attendu.
pub fn validate_change(change: &RecordChange) -> Result<(), InvalidObject> {
    let table = entities::table(&change.entity)
        .ok_or_else(|| invalid(format!("table inconnue : {}", change.entity)))?;
    validate_record_id(table, &change.id)?;
    if change.version.is_empty() {
        return Err(invalid("version absente"));
    }
    validate_version(&change.version, "enregistrement")?;
    for (column, version) in &change.columns {
        if !table.has_column(column) {
            return Err(invalid(format!(
                "colonne inconnue : {}.{column}",
                table.name
            )));
        }
        validate_version(version, "colonne")?;
        if !version.is_covered_by(&change.version) {
            return Err(invalid("version de colonne incohérente"));
        }
    }
    if let Some(data) = &change.data {
        for (column, value) in data {
            if !table.has_column(column) {
                return Err(invalid(format!(
                    "colonne inconnue : {}.{column}",
                    table.name
                )));
            }
            match value {
                serde_json::Value::Null
                | serde_json::Value::Bool(_)
                | serde_json::Value::Number(_) => {}
                serde_json::Value::String(text) if text.len() <= MAX_TEXT_BYTES => {}
                _ => return Err(invalid(format!("valeur refusée : {}.{column}", table.name))),
            }
        }
        let key = data
            .get(table.key)
            .and_then(key_text)
            .ok_or_else(|| invalid("identifiant absent du contenu"))?;
        if key != change.id {
            return Err(invalid("identifiant du contenu incohérent"));
        }
    } else if !change.attachments.is_empty() {
        return Err(invalid("fichier joint sur une suppression"));
    }
    for attachment in &change.attachments {
        if table.name != entities::PROFILE.name
            || !is_valid_photo_file(&attachment.file)
            || !is_hex_digest(&attachment.blob)
            || !is_hex_digest(&attachment.sha256)
            || attachment.size == 0
            || attachment.size > MAX_ATTACHMENT_BYTES
        {
            return Err(invalid("fichier joint invalide"));
        }
    }
    Ok(())
}

/// En-tête commun aux lots et aux instantanés, comparé au nom sous lequel l'objet a été lu.
struct Header<'a> {
    format: u32,
    space_id: &'a str,
    installation_id: &'a str,
    seq: u64,
    count: usize,
}

fn validate_header(
    header: &Header<'_>,
    space_id: &str,
    installation_id: &str,
    seq: u64,
) -> Result<(), InvalidObject> {
    if header.format > FORMAT_VERSION {
        return Err(invalid(
            "objet écrit par une version plus récente de Candilog",
        ));
    }
    if header.format == 0 {
        return Err(invalid("format inconnu"));
    }
    if header.space_id != space_id {
        return Err(invalid("objet d'un autre espace de synchronisation"));
    }
    if header.installation_id != installation_id || header.seq != seq {
        return Err(invalid("en-tête incohérent avec le nom de l'objet"));
    }
    if header.count > MAX_CHANGES_PER_OBJECT {
        return Err(invalid("objet trop volumineux"));
    }
    Ok(())
}

/// Valide un lot reçu sous un nom donné : l'en-tête doit correspondre au nom, ce qui
/// empêche de rejouer le lot d'une installation sous l'identité d'une autre.
///
/// # Errors
/// [`InvalidObject`] si l'en-tête ou un changement est invalide.
pub fn validate_batch(
    batch: &ChangeBatch,
    space_id: &str,
    installation_id: &str,
    seq: u64,
) -> Result<(), InvalidObject> {
    let header = Header {
        format: batch.format,
        space_id: &batch.space_id,
        installation_id: &batch.installation_id,
        seq: batch.seq,
        count: batch.changes.len(),
    };
    validate_header(&header, space_id, installation_id, seq)?;
    batch.changes.iter().try_for_each(validate_change)
}

/// Valide un instantané reçu sous un nom donné.
///
/// # Errors
/// [`InvalidObject`] si l'en-tête ou un enregistrement est invalide.
pub fn validate_snapshot(
    snapshot: &Snapshot,
    space_id: &str,
    installation_id: &str,
    seq: u64,
) -> Result<(), InvalidObject> {
    let header = Header {
        format: snapshot.format,
        space_id: &snapshot.space_id,
        installation_id: &snapshot.installation_id,
        seq: snapshot.seq,
        count: snapshot.records.len(),
    };
    validate_header(&header, space_id, installation_id, seq)?;
    validate_version(&snapshot.covered, "instantané")?;
    snapshot.records.iter().try_for_each(validate_change)
}

/// Valide le manifeste d'un espace, paramètres de dérivation compris : un manifeste forgé
/// ne doit pas pouvoir imposer une dérivation de plusieurs gigaoctets.
///
/// # Errors
/// [`InvalidObject`] si le manifeste n'est pas celui d'un espace Candilog lisible.
pub fn validate_manifest(manifest: &SpaceManifest) -> Result<(), InvalidObject> {
    if manifest.app != "candilog" {
        return Err(invalid("ce dossier n'est pas un espace Candilog"));
    }
    if manifest.format > FORMAT_VERSION || manifest.format == 0 {
        return Err(invalid(
            "espace créé par une version plus récente de Candilog",
        ));
    }
    if !is_valid_identifier(&manifest.space_id) {
        return Err(invalid("identifiant d'espace invalide"));
    }
    let kdf = &manifest.kdf;
    if kdf.algorithm != "argon2id"
        || !(19_456..=1_048_576).contains(&kdf.memory_kib)
        || !(1..=10).contains(&kdf.iterations)
        || !(1..=4).contains(&kdf.parallelism)
        || kdf.salt.is_empty()
        || kdf.salt.len() > 64
    {
        return Err(invalid("paramètres de chiffrement non pris en charge"));
    }
    if manifest.wrapped_key.len() > 256 || manifest.key_check.len() > 256 {
        return Err(invalid("manifeste invalide"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTALL: &str = "11111111-1111-4111-8111-111111111111";
    const SPACE: &str = "22222222-2222-4222-8222-222222222222";

    fn change(entity: &str, id: &str) -> RecordChange {
        let mut data = Row::new();
        data.insert("id".into(), serde_json::json!(id));
        data.insert("name".into(), serde_json::json!("Acme"));
        RecordChange {
            entity: entity.into(),
            id: id.into(),
            version: VersionVector::single(INSTALL, 1),
            columns: BTreeMap::from([("name".into(), VersionVector::single(INSTALL, 1))]),
            data: Some(data),
            attachments: vec![],
        }
    }

    #[test]
    fn les_noms_d_objets_font_l_aller_retour() {
        let name = batch_name(INSTALL, 42);
        assert_eq!(
            name,
            "changes/11111111-1111-4111-8111-111111111111/000000000042.batch"
        );
        assert_eq!(
            parse_object_name(&name),
            Some(ObjectKind::Batch {
                installation_id: INSTALL.into(),
                seq: 42
            })
        );
        assert!(matches!(
            parse_object_name(&snapshot_name(INSTALL, 3)),
            Some(ObjectKind::Snapshot { seq: 3, .. })
        ));
    }

    #[test]
    fn les_noms_inattendus_sont_ignores() {
        for name in [
            "changes/11111111-1111-4111-8111-111111111111/000000000042 (copie en conflit).batch",
            "changes/../000000000001.batch",
            "changes/11111111-1111-4111-8111-111111111111/000000000000.batch",
            "changes/pas-un-uuid/000000000001.batch",
            ".tmp-000000000001.batch",
            "blobs/../../etc/passwd.blob",
            "space.json",
        ] {
            assert_eq!(parse_object_name(name), None, "{name}");
        }
    }

    #[test]
    fn un_changement_valide_est_accepte() {
        validate_change(&change("companies", "abc-123")).unwrap();
    }

    #[test]
    fn une_table_ou_une_colonne_inconnue_est_refusee() {
        assert!(validate_change(&change("settings", "abc")).is_err());
        let mut forged = change("companies", "abc");
        forged
            .data
            .as_mut()
            .unwrap()
            .insert("evil; DROP TABLE".into(), serde_json::json!(1));
        assert!(validate_change(&forged).is_err());
    }

    #[test]
    fn un_identifiant_incoherent_est_refuse() {
        let mut forged = change("companies", "abc");
        forged.id = "autre".into();
        assert!(validate_change(&forged).is_err());
        let mut profile = change("profile", "1");
        profile.data = Some(Row::from([("id".into(), serde_json::json!(1))]));
        profile.columns.clear();
        validate_change(&profile).unwrap();
        profile.id = "2".into();
        assert!(validate_change(&profile).is_err());
    }

    #[test]
    fn une_valeur_composite_est_refusee() {
        let mut forged = change("companies", "abc");
        forged
            .data
            .as_mut()
            .unwrap()
            .insert("notes".into(), serde_json::json!({"x": 1}));
        assert!(validate_change(&forged).is_err());
    }

    #[test]
    fn un_fichier_joint_hors_photo_est_refuse() {
        let mut forged = change("profile", "1");
        forged.data = Some(Row::from([("id".into(), serde_json::json!(1))]));
        forged.columns.clear();
        forged.attachments.push(Attachment {
            file: "../../.ssh/id_rsa".into(),
            blob: "a".repeat(64),
            sha256: "b".repeat(64),
            size: 10,
        });
        assert!(validate_change(&forged).is_err());
        forged.attachments[0].file = format!("photo-{INSTALL}.png");
        validate_change(&forged).unwrap();
    }

    #[test]
    fn un_lot_renomme_est_refuse() {
        let batch = ChangeBatch {
            format: FORMAT_VERSION,
            space_id: SPACE.into(),
            installation_id: INSTALL.into(),
            seq: 2,
            schema_version: 9,
            created_at: "2026-10-10T00:00:00Z".into(),
            changes: vec![change("companies", "abc")],
        };
        validate_batch(&batch, SPACE, INSTALL, 2).unwrap();
        assert!(validate_batch(&batch, SPACE, INSTALL, 3).is_err());
        assert!(validate_batch(&batch, INSTALL, INSTALL, 2).is_err());
        let mut newer = batch;
        newer.format = FORMAT_VERSION + 1;
        assert!(validate_batch(&newer, SPACE, INSTALL, 2).is_err());
    }

    #[test]
    fn un_manifeste_aux_parametres_abusifs_est_refuse() {
        let manifest = SpaceManifest {
            format: 1,
            app: "candilog".into(),
            space_id: SPACE.into(),
            created_at: String::new(),
            kdf: KdfParams {
                algorithm: "argon2id".into(),
                memory_kib: 65_536,
                iterations: 3,
                parallelism: 1,
                salt: "c2Vs".into(),
            },
            wrapped_key: "x".into(),
            key_check: "y".into(),
        };
        validate_manifest(&manifest).unwrap();
        let mut abusive = manifest.clone();
        abusive.kdf.memory_kib = 64 * 1024 * 1024;
        assert!(validate_manifest(&abusive).is_err());
        let mut foreign = manifest;
        foreign.app = "autre".into();
        assert!(validate_manifest(&foreign).is_err());
    }
}
