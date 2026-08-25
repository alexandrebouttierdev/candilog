//! État applicatif partagé, construit une fois au démarrage et injecté par Tauri.

use crate::core::config::AppPaths;
use crate::core::database::{open_pool, run_local_migrations, SqlitePool};
use crate::core::errors::AppResult;
use std::path::PathBuf;

/// Dépendances partagées par toutes les commandes.
///
/// Un unique exemplaire est construit au démarrage puis passé à Tauri via `manage` :
/// les commandes le reçoivent en `State<'_, AppState>` et ne recréent jamais ni connexion,
/// ni dépôt, ni client HTTP (MIGRATION.md §23).
///
/// Les services métier y sont ajoutés au fur et à mesure des tranches de migration ; le
/// champ `sqlite` reste le seul socle commun.
pub struct AppState {
    /// Pool `SQLite` local.
    pub sqlite: SqlitePool,
    /// Chemin du fichier de base, nécessaire à l'export et à la restauration de sauvegarde.
    pub db_path: PathBuf,
}

impl AppState {
    /// Construit l'état sur le fichier de données de l'utilisateur et applique les migrations.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si le pool ne peut pas être ouvert ou si une migration
    /// échoue, et `AppError::Validation` si le dossier de données est introuvable.
    pub fn persistent() -> AppResult<Self> {
        let paths = AppPaths::discover()?;
        let pool = open_pool(Some(&paths.database))?;
        run_local_migrations(&pool)?;
        // Le fichier de base n'existe pas encore au moment où les chemins sont résolus :
        // ses permissions ne peuvent être restreintes qu'une fois la base ouverte.
        paths.securiser();
        Ok(Self {
            sqlite: pool,
            db_path: paths.database,
        })
    }

    /// Construit l'état sur une base **en mémoire**, réservé aux tests.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si le pool ne peut pas être initialisé.
    pub fn in_memory() -> AppResult<Self> {
        let pool = open_pool(None)?;
        run_local_migrations(&pool)?;
        Ok(Self {
            sqlite: pool,
            db_path: PathBuf::new(),
        })
    }
}
