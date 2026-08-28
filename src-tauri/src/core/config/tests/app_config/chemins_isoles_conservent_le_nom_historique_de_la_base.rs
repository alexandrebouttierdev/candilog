//! Cas de test isolé.

use super::*;

#[test]
fn chemins_isoles_conservent_le_nom_historique_de_la_base() {
    let paths = AppPaths::in_directory(PathBuf::from("/tmp/candilog-test"));
    assert!(paths.database.ends_with("candilog.sqlite"));
    assert!(paths.exports_dir.ends_with("exports"));
}
