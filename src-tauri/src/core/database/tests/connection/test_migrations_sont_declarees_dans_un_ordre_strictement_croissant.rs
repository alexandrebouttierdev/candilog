//! Cas de test isolé.

use super::*;

#[test]
fn test_migrations_sont_declarees_dans_un_ordre_strictement_croissant() {
    // Le runner ignore toute migration dont la version est inférieure ou égale au curseur :
    // une version inversée ou dupliquée serait silencieusement sautée à l'ajout du prochain
    // fichier. Ce test verrouille l'invariant plutôt que la vigilance du relecteur.
    let versions: Vec<i64> = MIGRATIONS.iter().map(|(version, _)| *version).collect();
    let mut triees = versions.clone();
    triees.sort_unstable();
    triees.dedup();
    assert_eq!(
        versions, triees,
        "versions non strictement croissantes : {versions:?}"
    );
    assert_eq!(
        versions.last().copied(),
        Some(LATEST_SCHEMA_VERSION),
        "LATEST_SCHEMA_VERSION ne correspond pas à la dernière migration déclarée"
    );
}
