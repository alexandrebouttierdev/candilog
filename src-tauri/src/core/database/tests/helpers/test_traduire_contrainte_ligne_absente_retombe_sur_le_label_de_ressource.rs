//! Cas de test isolé.

use super::*;

#[test]
fn test_traduire_contrainte_ligne_absente_retombe_sur_le_label_de_ressource() {
    let traduite = translate_constraint(
        rusqlite::Error::QueryReturnedNoRows,
        "phrase",
        "candidature",
    );
    match traduite {
        AppError::NotFound(label) => assert_eq!(label, "candidature"),
        other => panic!("attendu NotFound, obtenu {other:?}"),
    }
}
