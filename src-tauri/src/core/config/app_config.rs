//! Résolution multiplateforme des chemins de données Candilog.

use crate::core::errors::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// Identifiant unique de Candilog : celui du paquet (`src-tauri/tauri.conf.json`), du
/// dossier de données et de l'entrée de trousseau (`core::secrets`).
///
/// Un identifiant unique, et non trois : la désinstallation, la sauvegarde et le nettoyage
/// manuel se raisonnent sinon sur trois emplacements que rien ne relie visiblement au
/// paquet installé.
pub const APP_IDENTIFIER: &str = "fr.candilog.desktop";

/// Dossier de données des versions antérieures à l'unification de l'identifiant.
///
/// Aucune version publique n'a jamais utilisé ce nom — il ne subsiste que sur les machines
/// de développement. La reprise existe malgré tout : perdre la base d'un poste de travail
/// parce qu'un dossier a changé de nom serait une régression, et le coût est de trois
/// renommages au premier démarrage.
const LEGACY_DATA_DIR: &str = "com.candilog.desktop";

/// Nom du fichier de base, commun aux deux emplacements.
const DATABASE_FILE: &str = "candilog.sqlite";

/// Paths persistants utilisés par Candilog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Dossier de données de l'application.
    pub data_dir: PathBuf,
    /// Base SQLite compatible avec Candilog Desktop historique.
    pub database: PathBuf,
    /// Dossier d'exports utilisateur.
    pub exports_dir: PathBuf,
    /// Dossier des photos de profil, hors base : une image n'a pas sa place dans une
    /// colonne JSON relue à chaque ouverture d'écran.
    pub photos_dir: PathBuf,
}

impl AppPaths {
    /// Résout et crée les dossiers applicatifs sur Linux, Windows et macOS.
    ///
    /// # Errors
    /// Retourne une erreur si le système ne fournit pas de dossier de données ou si sa création échoue.
    pub fn discover() -> AppResult<Self> {
        Self::discover_dans(&Self::resoudre_dossier()?)
    }

    /// Dossier de données à utiliser, avant toute création.
    ///
    /// Un binaire de développement ne doit jamais ouvrir la base utilisateur : il écrit sous
    /// le projet, ancré sur le manifeste Cargo et non sur le répertoire courant — `cargo run`
    /// depuis `src-tauri/` et `npm run tauri dev` depuis la racine n'ont pas le même `cwd`,
    /// et ouvriraient sinon deux bases de développement distinctes, un écran vide après
    /// saisie que rien de visible n'expliquerait.
    fn resoudre_dossier() -> AppResult<PathBuf> {
        if let Some(override_dir) = std::env::var_os("CANDILOG_DATA_DIR") {
            return Ok(PathBuf::from(override_dir));
        }
        if cfg!(debug_assertions) {
            return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".candilog-dev"));
        }
        let racine = dirs::data_dir().ok_or_else(|| {
            AppError::Validation(
                "Le dossier de données du système est introuvable. Candilog ne sait pas où \
                 enregistrer vos candidatures."
                    .into(),
            )
        })?;
        let dossier = racine.join(APP_IDENTIFIER);
        reprendre_base_heritee(&racine.join(LEGACY_DATA_DIR), &dossier);
        Ok(dossier)
    }

    /// Crée et sécurise l'arborescence de données sous `data_dir`.
    ///
    /// # Errors
    /// Retourne `Validation` si le dossier ne peut pas être créé — disque plein, droits
    /// insuffisants, chemin occupé par un fichier.
    pub fn discover_dans(data_dir: &std::path::Path) -> AppResult<Self> {
        let data_dir = data_dir.to_path_buf();
        let exports_dir = data_dir.join("exports");
        let photos_dir = data_dir.join("photos");
        // `Validation` et non `Database` : à ce stade aucune base n'a été ouverte. La
        // variante `Database` affichait « Le fichier de données de Candilog est illisible ou
        // endommagé » à quelqu'un dont le disque est plein ou le dossier en lecture seule —
        // un diagnostic faux, qui envoie chercher une corruption inexistante.
        //
        // Le chemin figure dans le message : c'est celui de l'utilisateur, pas un chemin
        // interne, et sans lui la phrase n'indique rien à corriger.
        std::fs::create_dir_all(&photos_dir).and_then(|()| std::fs::create_dir_all(&exports_dir)).map_err(|error| {
            // L'erreur système est en anglais et parle en numéros (« os error 13 ») : elle
            // part au journal, où elle sert au diagnostic, et non à l'écran (§1, §13).
            tracing::error!(dossier = %data_dir.display(), %error, "dossier de données non créé");
            AppError::Validation(format!(
                "Candilog n'a pas pu créer son dossier de données ({}). Vérifiez que vous avez \
                 les droits d'écriture sur cet emplacement et qu'il reste de l'espace disque.",
                data_dir.display()
            ))
        })?;
        Self::restreindre_acces(&data_dir);
        Ok(Self {
            database: data_dir.join(DATABASE_FILE),
            data_dir,
            exports_dir,
            photos_dir,
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
        let mut cibles = vec![
            (data_dir.to_path_buf(), 0o700),
            (data_dir.join("exports"), 0o700),
            (data_dir.join("photos"), 0o700),
            (data_dir.join(DATABASE_FILE), 0o600),
            (data_dir.join("candilog.sqlite-wal"), 0o600),
            (data_dir.join("candilog.sqlite-shm"), 0o600),
            (data_dir.join("candilog.log"), 0o600),
        ];
        // Les journaux tournés portent les mêmes lignes que le courant : les laisser au
        // `umask` de session annulerait la protection de celui-ci dès la deuxième session.
        cibles.extend((1..8).map(|rang| (data_dir.join(format!("candilog.log.{rang}")), 0o600)));
        for (path, mode) in cibles {
            if !path.exists() {
                continue;
            }
            if let Err(error) =
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
            {
                tracing::warn!(?path, %error, "permissions non appliquées");
            }
        }
    }

    /// Sans équivalent portable hors Unix : les ACL Windows héritent du profil utilisateur.
    #[cfg(not(unix))]
    const fn restreindre_acces(_data_dir: &std::path::Path) {}
}

/// Restreint un fichier de données à son seul propriétaire.
///
/// Complète [`AppPaths::restreindre_acces`], qui ne connaît qu'une liste fixe de chemins :
/// les sauvegardes et la copie de secours prise avant une restauration portent les mêmes
/// données personnelles que la base, mais vivent où l'utilisateur les demande. Un échec est
/// journalisé sans interrompre l'opération — le fichier reste exploitable, simplement moins
/// protégé.
#[cfg(unix)]
pub fn restreindre_fichier(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Err(error) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
        tracing::warn!(?path, %error, "permissions du fichier non appliquées");
    }
}

/// Sans équivalent portable hors Unix : les ACL Windows héritent du profil utilisateur.
#[cfg(not(unix))]
pub const fn restreindre_fichier(_path: &std::path::Path) {}

impl AppPaths {
    /// Construit des chemins isolés, notamment pour les tests.
    #[must_use]
    pub fn in_directory(data_dir: PathBuf) -> Self {
        Self {
            database: data_dir.join(DATABASE_FILE),
            exports_dir: data_dir.join("exports"),
            photos_dir: data_dir.join("photos"),
            data_dir,
        }
    }
}

/// Déplace la base d'un ancien dossier de données vers le nouveau, une seule fois.
///
/// La reprise ne s'exécute que si la destination n'a **pas** encore de base : une base
/// existante appartient à l'installation courante et n'a jamais à être écrasée. Chaque
/// fichier est déplacé par renommage — les deux dossiers partagent leur parent, l'opération
/// est donc atomique et ne duplique rien. Un échec est journalisé sans interrompre le
/// démarrage : l'application ouvrira alors une base neuve, et l'ancienne reste intacte,
/// récupérable à la main.
fn reprendre_base_heritee(ancien: &Path, nouveau: &Path) {
    let source = ancien.join(DATABASE_FILE);
    let cible = nouveau.join(DATABASE_FILE);
    if cible.exists() || !source.exists() {
        return;
    }
    if let Err(error) = std::fs::create_dir_all(nouveau) {
        tracing::warn!(%error, "dossier de données non créé, reprise abandonnée");
        return;
    }
    // Les journaux WAL et SHM accompagnent la base : les laisser derrière ferait perdre
    // les transactions non encore intégrées au fichier principal.
    for nom in [DATABASE_FILE, "candilog.sqlite-wal", "candilog.sqlite-shm"] {
        let depuis = ancien.join(nom);
        if !depuis.exists() {
            continue;
        }
        match std::fs::rename(&depuis, nouveau.join(nom)) {
            Ok(()) => tracing::info!(fichier = nom, "donnée héritée reprise"),
            Err(error) => tracing::warn!(fichier = nom, %error, "reprise impossible"),
        }
    }
}

#[cfg(test)]
#[path = "tests/app_config/mod.rs"]
mod tests;
