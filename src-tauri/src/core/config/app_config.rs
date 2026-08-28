//! Résolution multiplateforme des chemins de données Candilog.

use crate::core::errors::{AppError, AppResult};
use std::path::PathBuf;

/// Chemins persistants utilisés par Candilog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Dossier de données de l'application.
    pub data_dir: PathBuf,
    /// Base SQLite compatible avec Candilog Desktop historique.
    pub database: PathBuf,
    /// Dossier d'exports utilisateur.
    pub exports_dir: PathBuf,
}

impl AppPaths {
    /// Résout et crée les dossiers applicatifs sur Linux, Windows et macOS.
    ///
    /// # Errors
    /// Retourne une erreur si le système ne fournit pas de dossier de données ou si sa création échoue.
    pub fn discover() -> AppResult<Self> {
        // Un binaire de développement ne doit jamais ouvrir la base utilisateur historique.
        // Il écrit sous le projet ; les releases gardent le dossier historique.
        //
        // Le dossier est ancré sur le manifeste Cargo et non sur le répertoire courant :
        // `cargo run` depuis `src-tauri/` et `npm run tauri dev` depuis la racine n'ont pas
        // le même `cwd`, et ouvriraient donc deux bases de développement distinctes — un
        // écran vide après avoir saisi des données ne s'explique alors par rien de visible.
        let data_dir = if let Some(override_dir) = std::env::var_os("CANDILOG_DATA_DIR") {
            PathBuf::from(override_dir)
        } else if cfg!(debug_assertions) {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".candilog-dev")
        } else {
            dirs::data_dir()
                .ok_or_else(|| {
                    AppError::Validation("Le dossier de données du système est introuvable.".into())
                })?
                .join("com.candilog.desktop")
        };
        let exports_dir = data_dir.join("exports");
        std::fs::create_dir_all(&exports_dir).map_err(|error| {
            AppError::Database(format!("Impossible de créer le dossier Candilog : {error}"))
        })?;
        Self::restreindre_acces(&data_dir);
        Ok(Self {
            database: data_dir.join("candilog.sqlite"),
            data_dir,
            exports_dir,
        })
    }

    /// Restreint les fichiers Candilog à leur seul propriétaire.
    ///
    /// Rappelée après l'ouverture de la base : au tout premier lancement, le fichier
    /// `candilog.sqlite` n'existe pas encore quand les chemins sont résolus.
    pub fn securiser(&self) {
        Self::restreindre_acces(&self.data_dir);
    }

    /// Restreint le dossier de données à son seul propriétaire.
    ///
    /// Ni le dossier ni le fichier de base ne se voyaient appliquer de permissions
    /// explicites : `create_dir_all` et l'ouverture SQLite retenaient le `umask` de la
    /// session, ce qui donnait en pratique `755` sur le dossier et `644` sur la base — laquelle
    /// contient l'intégralité des données personnelles : profil complet, contenu des CV
    /// générés, coordonnées des contacts, notes et comptes rendus d'entretiens.
    ///
    /// Le mode `700` de `~/.local/share` atténuait le risque sous Linux, mais c'est une
    /// protection du système, pas de l'application : elle ne vaut ni sous un `umask` permissif,
    /// ni sur les autres plateformes, ni si `CANDILOG_DATA_DIR` pointe vers un dossier partagé.
    ///
    /// Sous Windows, l'équivalent relève des ACL héritées du profil utilisateur ; le sujet est
    /// documenté dans `docs/DATA.md`.
    #[cfg(unix)]
    fn restreindre_acces(data_dir: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        // Un échec n'empêche pas le démarrage : le dossier reste utilisable, simplement
        // moins protégé, et l'incident est journalisé.
        // Les journaux WAL portent les mêmes données que la base : les laisser en 644
        // annulerait la protection du fichier principal.
        for (chemin, mode) in [
            (data_dir.to_path_buf(), 0o700),
            (data_dir.join("exports"), 0o700),
            (data_dir.join("candilog.sqlite"), 0o600),
            (data_dir.join("candilog.sqlite-wal"), 0o600),
            (data_dir.join("candilog.sqlite-shm"), 0o600),
            (data_dir.join("candilog.log"), 0o600),
        ] {
            if !chemin.exists() {
                continue;
            }
            if let Err(error) =
                std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(mode))
            {
                tracing::warn!(?chemin, %error, "permissions non appliquées");
            }
        }
    }

    /// Sans équivalent portable hors Unix : les ACL Windows héritent du profil utilisateur.
    #[cfg(not(unix))]
    const fn restreindre_acces(_data_dir: &std::path::Path) {}

    /// Construit des chemins isolés, notamment pour les tests.
    #[must_use]
    pub fn in_directory(data_dir: PathBuf) -> Self {
        Self {
            database: data_dir.join("candilog.sqlite"),
            exports_dir: data_dir.join("exports"),
            data_dir,
        }
    }
}

#[cfg(test)]
#[path = "tests/app_config/mod.rs"]
mod tests;
