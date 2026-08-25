//! État applicatif partagé, construit une fois au démarrage et injecté par Tauri.

use crate::core::config::AppPaths;
use crate::core::database::{open_pool, run_local_migrations, SqlitePool};
use crate::core::errors::AppResult;
use crate::features::candidatures::application::CandidatureService;
use crate::features::candidatures::infrastructure::SqliteCandidatureRepository;
use crate::features::contacts::application::ContactService;
use crate::features::contacts::infrastructure::SqliteContactRepository;
use crate::features::entreprises::application::EntrepriseService;
use crate::features::entreprises::infrastructure::SqliteEntrepriseRepository;
use crate::features::secteurs::application::SecteurService;
use crate::features::secteurs::infrastructure::SqliteSecteurRepository;
use std::path::PathBuf;
use std::sync::Arc;

/// Service des candidatures tel que partagé par les commandes.
pub type Candidatures = Arc<CandidatureService<SqliteCandidatureRepository>>;
/// Service des entreprises tel que partagé par les commandes.
pub type Entreprises = Arc<EntrepriseService<SqliteEntrepriseRepository>>;
/// Service des contacts tel que partagé par les commandes.
pub type Contacts = Arc<ContactService<SqliteContactRepository>>;
/// Service du référentiel des secteurs tel que partagé par les commandes.
pub type Secteurs = Arc<SecteurService<SqliteSecteurRepository>>;

/// Dépendances partagées par toutes les commandes.
///
/// Un unique exemplaire est construit au démarrage puis confié à Tauri via `manage` : les
/// commandes le reçoivent en `State<'_, AppState>` et ne recréent jamais ni connexion, ni
/// dépôt, ni client HTTP (MIGRATION.md §23).
///
/// Les services sont derrière `Arc` parce qu'une commande `async` doit s'approprier ce
/// qu'elle déplace vers `spawn_blocking` : elle ne peut pas y emprunter l'état, dont la
/// durée de vie est liée à l'appel.
///
/// D'autres services s'ajoutent au fil des tranches de migration.
pub struct AppState {
    /// Service des candidatures.
    pub candidatures: Candidatures,
    /// Service des entreprises.
    pub entreprises: Entreprises,
    /// Service des contacts du réseau.
    pub contacts: Contacts,
    /// Service du référentiel des secteurs d'activité.
    pub secteurs: Secteurs,
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
        Self::sur_pool(pool, paths.database)
    }

    /// Construit l'état sur une base **en mémoire**, réservé aux tests.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si le pool `SQLite` ne peut pas être initialisé.
    pub fn in_memory() -> AppResult<Self> {
        let pool = open_pool(None)?;
        run_local_migrations(&pool)?;
        Self::sur_pool(pool, PathBuf::new())
    }

    /// Assemble dépôts et services autour d'un pool déjà migré.
    fn sur_pool(pool: SqlitePool, db_path: PathBuf) -> AppResult<Self> {
        let secteurs_repo = SqliteSecteurRepository::new(pool.clone());
        // Le référentiel est garanti au démarrage : le sélecteur du formulaire entreprise
        // serait vide sur une base neuve, et les secteurs saisis librement dans l'ancienne
        // application resteraient sans ligne correspondante.
        secteurs_repo.garantir_referentiel()?;

        Ok(Self {
            candidatures: Arc::new(CandidatureService::new(SqliteCandidatureRepository::new(
                pool.clone(),
            ))),
            entreprises: Arc::new(EntrepriseService::new(SqliteEntrepriseRepository::new(
                pool.clone(),
            ))),
            contacts: Arc::new(ContactService::new(SqliteContactRepository::new(
                pool.clone(),
            ))),
            secteurs: Arc::new(SecteurService::new(secteurs_repo)),
            sqlite: pool,
            db_path,
        })
    }
}
