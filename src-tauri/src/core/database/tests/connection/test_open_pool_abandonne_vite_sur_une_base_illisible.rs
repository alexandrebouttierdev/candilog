//! Cas de test isolé.

use super::*;

/// `Pool::builder()` sans `connection_timeout` retient le défaut de r2d2 : **30 secondes**.
/// `build()` étant bloquant et appelé depuis `App::new()` avant que la première fenêtre ne soit
/// rendue, une base illisible laisse l'utilisateur devant un écran vide une demi-minute, sans
/// fenêtre ni message — indiscernable d'un plantage silencieux.
#[test]
fn test_open_pool_abandonne_vite_sur_une_base_illisible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.sqlite");
    std::fs::write(&path, vec![b'x'; 4096]).unwrap();

    let debut = std::time::Instant::now();
    let resultat = open_pool(Some(&path));
    let duree = debut.elapsed();

    assert!(resultat.is_err(), "une base illisible doit être refusée");
    assert!(
        duree < std::time::Duration::from_secs(10),
        "l'ouverture a mis {duree:?} : le délai par défaut de r2d2 n'est pas redéfini"
    );
}
