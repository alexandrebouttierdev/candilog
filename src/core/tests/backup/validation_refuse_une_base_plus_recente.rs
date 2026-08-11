//! Cas de test isolé.

use super::*;

/// `docs/DATA.md` demande de « vérifier les versions ». Un backup produit par une version
/// ultérieure de Candilog porte un schéma que cette version ne connaît pas : `run_local_migrations`
/// n'a alors plus rien à appliquer (sa boucle ignore les cibles `<= user_version`), et
/// l'application lit un schéma inconnu. Le refus doit intervenir avant de toucher la base active.
#[test]
fn validation_refuse_une_base_plus_recente() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("futur.sqlite");
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    export(&pool, &path).unwrap();

    {
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .pragma_update(None, "user_version", DERNIERE_VERSION + 1)
            .unwrap();
    }

    let erreur = validate(&path).unwrap_err();
    assert!(
        matches!(erreur, AppError::Validation(_)),
        "une base plus récente doit être refusée par la validation, pas par une erreur technique : {erreur:?}"
    );
    let message = erreur.to_string();
    assert!(
        message.contains("mise à jour"),
        "le message doit indiquer qu'une mise à jour de Candilog est nécessaire : {message}"
    );
}
