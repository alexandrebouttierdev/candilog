//! Fournisseur « dossier personnalisé » : un dossier local, éventuellement synchronisé par
//! un outil tiers (Dropbox, iCloud Drive, Nextcloud, partage réseau).
//!
//! Candilog n'écrit que des fichiers neufs, sous des noms qu'une seule installation
//! utilise, par écriture dans un fichier temporaire puis renommage : un fichier n'est jamais
//! visible à moitié écrit **par Candilog**. L'outil tiers, lui, peut livrer un fichier
//! incomplet ou en retard ; le moteur traite alors l'objet comme pas encore disponible et
//! réessaie (`docs/SYNC.md`, « Dossier personnalisé »).

use crate::features::sync::domain::remote::{PutOutcome, RemoteError, RemoteObject, RemoteStore};
use async_trait::async_trait;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Sous-dossier créé dans le dossier choisi.
pub const SPACE_DIR: &str = "candilog-sync";

/// Espace de synchronisation dans un dossier local.
#[derive(Debug, Clone)]
pub struct FolderStore {
    root: PathBuf,
}

fn io_error(error: &std::io::Error, context: &str) -> RemoteError {
    tracing::warn!(%error, context, "accès au dossier de synchronisation impossible");
    match error.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
            RemoteError::Unavailable(format!("{context} : dossier introuvable ou non accessible"))
        }
        _ => RemoteError::Provider(format!("{context} : erreur d'écriture ou de lecture")),
    }
}

/// Un nom d'objet ne peut désigner qu'un fichier **sous** l'espace : segments simples,
/// jamais `..`, jamais de fichier caché.
fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.starts_with('.')
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

impl FolderStore {
    /// Ouvre (et crée au besoin) l'espace sous le dossier choisi, après avoir vérifié
    /// qu'on peut y écrire, relire et supprimer un fichier.
    ///
    /// # Errors
    /// `Unavailable` si le dossier n'existe pas ou n'est pas accessible en écriture.
    pub fn open(chosen: &Path) -> Result<Self, RemoteError> {
        if !chosen.is_dir() {
            return Err(RemoteError::Unavailable(
                "dossier introuvable : vérifiez qu'il est monté et accessible".into(),
            ));
        }
        let root = chosen.join(SPACE_DIR);
        std::fs::create_dir_all(&root).map_err(|error| io_error(&error, "création de l'espace"))?;
        let store = Self { root };
        store.probe()?;
        Ok(store)
    }

    /// Ouvre un espace existant sans le créer : un dossier démonté ne doit pas être
    /// recréé vide à son emplacement de montage.
    ///
    /// # Errors
    /// `Unavailable` si l'espace n'existe pas.
    pub fn existing(chosen: &Path) -> Result<Self, RemoteError> {
        let root = chosen.join(SPACE_DIR);
        if root.is_dir() {
            Ok(Self { root })
        } else {
            Err(RemoteError::Unavailable(
                "dossier de synchronisation introuvable : vérifiez qu'il est monté".into(),
            ))
        }
    }

    fn probe(&self) -> Result<(), RemoteError> {
        let path = self
            .root
            .join(format!(".probe-{}.tmp", uuid::Uuid::new_v4()));
        let content = b"candilog";
        std::fs::write(&path, content).map_err(|error| io_error(&error, "test d'écriture"))?;
        let read = std::fs::read(&path).map_err(|error| io_error(&error, "test de lecture"));
        let removed =
            std::fs::remove_file(&path).map_err(|error| io_error(&error, "test de suppression"));
        if read? != content {
            return Err(RemoteError::Unavailable(
                "le dossier ne relit pas ce qui y est écrit".into(),
            ));
        }
        removed
    }

    fn path_for(&self, name: &str) -> Result<PathBuf, RemoteError> {
        let mut path = self.root.clone();
        for segment in name.split('/') {
            if !valid_segment(segment) {
                return Err(RemoteError::Provider(format!(
                    "nom d'objet refusé : {name}"
                )));
            }
            path.push(segment);
        }
        Ok(path)
    }

    fn list_blocking(&self, prefix: &str) -> Result<Vec<RemoteObject>, RemoteError> {
        if !self.root.is_dir() {
            return Err(RemoteError::Unavailable(
                "dossier de synchronisation introuvable".into(),
            ));
        }
        let mut objects = Vec::new();
        let mut stack = vec![(self.root.clone(), String::new(), 0_usize)];
        while let Some((directory, relative, depth)) = stack.pop() {
            let entries = match std::fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(io_error(&error, "lecture du dossier")),
            };
            for entry in entries {
                let entry = entry.map_err(|error| io_error(&error, "lecture du dossier"))?;
                let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                if !valid_segment(&file_name) {
                    continue;
                }
                let name = if relative.is_empty() {
                    file_name
                } else {
                    format!("{relative}/{file_name}")
                };
                let file_type = entry
                    .file_type()
                    .map_err(|error| io_error(&error, "lecture du dossier"))?;
                if file_type.is_dir() && depth < 3 {
                    stack.push((entry.path(), name, depth + 1));
                } else if file_type.is_file() && name.starts_with(prefix) {
                    let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
                    objects.push(RemoteObject { name, size });
                }
            }
        }
        objects.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(objects)
    }

    fn put_blocking(&self, name: &str, bytes: &[u8]) -> Result<PutOutcome, RemoteError> {
        let path = self.path_for(name)?;
        if path.exists() {
            return Ok(PutOutcome::AlreadyExists);
        }
        let parent = path
            .parent()
            .ok_or_else(|| RemoteError::Provider("chemin sans dossier parent".into()))?;
        std::fs::create_dir_all(parent)
            .map_err(|error| io_error(&error, "création d'un dossier"))?;
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("objet");
        let temporary = parent.join(format!(".{file_name}.{}.tmp", uuid::Uuid::new_v4()));
        let written = (|| {
            let mut file = std::fs::File::create(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()
        })();
        if let Err(error) = written {
            let _ = std::fs::remove_file(&temporary);
            return Err(io_error(&error, "écriture d'un objet"));
        }
        // Un seul écrivain par nom : la vérification d'existence ci-dessus ne peut pas être
        // devancée par un autre ordinateur, qui n'écrit jamais sous ce nom.
        if let Err(error) = std::fs::rename(&temporary, &path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(io_error(&error, "écriture d'un objet"));
        }
        #[cfg(unix)]
        if let Ok(directory) = std::fs::File::open(parent) {
            let _ = directory.sync_all();
        }
        Ok(PutOutcome::Created)
    }
}

#[async_trait]
impl RemoteStore for FolderStore {
    async fn list(&self, prefix: &str) -> Result<Vec<RemoteObject>, RemoteError> {
        let store = self.clone();
        let prefix = prefix.to_owned();
        tokio::task::spawn_blocking(move || store.list_blocking(&prefix))
            .await
            .map_err(|_| RemoteError::Provider("lecture interrompue".into()))?
    }

    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>, RemoteError> {
        let path = self.path_for(name)?;
        let root = self.root.clone();
        tokio::task::spawn_blocking(move || {
            if !root.is_dir() {
                return Err(RemoteError::Unavailable(
                    "dossier de synchronisation introuvable".into(),
                ));
            }
            match std::fs::read(&path) {
                Ok(bytes) => Ok(Some(bytes)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(io_error(&error, "lecture d'un objet")),
            }
        })
        .await
        .map_err(|_| RemoteError::Provider("lecture interrompue".into()))?
    }

    async fn put_new(&self, name: &str, bytes: &[u8]) -> Result<PutOutcome, RemoteError> {
        let store = self.clone();
        let name = name.to_owned();
        let bytes = bytes.to_vec();
        tokio::task::spawn_blocking(move || store.put_blocking(&name, &bytes))
            .await
            .map_err(|_| RemoteError::Provider("écriture interrompue".into()))?
    }

    async fn delete(&self, name: &str) -> Result<(), RemoteError> {
        let path = self.path_for(name)?;
        tokio::task::spawn_blocking(move || match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error(&error, "suppression d'un objet")),
        })
        .await
        .map_err(|_| RemoteError::Provider("suppression interrompue".into()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn un_objet_ecrit_se_relit_et_se_liste() {
        let directory = tempfile::tempdir().unwrap();
        let store = FolderStore::open(directory.path()).unwrap();
        let name = "changes/11111111-1111-4111-8111-111111111111/000000000001.batch";
        assert_eq!(
            store.put_new(name, b"abc").await.unwrap(),
            PutOutcome::Created
        );
        assert_eq!(store.get(name).await.unwrap().unwrap(), b"abc");
        let listed = store.list("changes/").await.unwrap();
        assert_eq!(
            listed,
            vec![RemoteObject {
                name: name.into(),
                size: 3
            }]
        );
    }

    #[tokio::test]
    async fn un_objet_existant_n_est_jamais_remplace() {
        let directory = tempfile::tempdir().unwrap();
        let store = FolderStore::open(directory.path()).unwrap();
        store.put_new("space.json", b"premier").await.unwrap();
        assert_eq!(
            store.put_new("space.json", b"second").await.unwrap(),
            PutOutcome::AlreadyExists
        );
        assert_eq!(store.get("space.json").await.unwrap().unwrap(), b"premier");
    }

    #[tokio::test]
    async fn un_nom_hors_de_l_espace_est_refuse() {
        let directory = tempfile::tempdir().unwrap();
        let store = FolderStore::open(directory.path()).unwrap();
        for name in ["../evasion", "changes/../../x", "/etc/passwd", ".cache"] {
            assert!(store.put_new(name, b"x").await.is_err(), "{name}");
            assert!(store.get(name).await.is_err(), "{name}");
        }
    }

    #[tokio::test]
    async fn les_fichiers_temporaires_et_inattendus_sont_ignores_a_la_lecture() {
        let directory = tempfile::tempdir().unwrap();
        let store = FolderStore::open(directory.path()).unwrap();
        let root = directory.path().join(SPACE_DIR).join("changes");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(".000000000001.batch.tmp"), b"partiel").unwrap();
        std::fs::write(root.join("copie en conflit.batch"), b"x").unwrap();
        let listed = store.list("changes/").await.unwrap();
        assert!(listed.is_empty(), "{listed:?}");
    }

    #[tokio::test]
    async fn un_dossier_disparu_est_signale_comme_inaccessible() {
        let directory = tempfile::tempdir().unwrap();
        let store = FolderStore::open(directory.path()).unwrap();
        std::fs::remove_dir_all(directory.path().join(SPACE_DIR)).unwrap();
        assert!(matches!(
            store.list("").await,
            Err(RemoteError::Unavailable(_))
        ));
        assert!(matches!(
            FolderStore::existing(directory.path()),
            Err(RemoteError::Unavailable(_))
        ));
    }

    #[test]
    fn un_dossier_inexistant_est_refuse() {
        assert!(FolderStore::open(Path::new("/chemin/qui/n/existe/pas")).is_err());
    }
}
