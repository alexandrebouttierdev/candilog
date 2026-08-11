//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::state::AppState as BackendState;

/// Application hermétique : base **en mémoire**, aucun chemin réel, aucune variable
/// d'environnement. Remplace l'ancien recours à `App::new()`, qui ouvrait le fichier
/// `.candilog-dev/candilog.sqlite` du dépôt et mutait `CANDILOG_DATA_DIR` pour tout le
/// processus — donc pour les tests s'exécutant en parallèle.
#[cfg(test)]
fn app_de_test() -> App {
    let backend = BackendState::new().unwrap();
    App::with_backend(
        AppPaths::in_directory(std::path::PathBuf::from("/inexistant")),
        backend,
    )
}

mod test_le_reload_marque_l_application_initialisee;
mod test_un_jeu_illisible_ne_bloque_pas_les_autres_ecrans;
