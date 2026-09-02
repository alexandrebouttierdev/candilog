//! Cas de test isolé.

use super::*;

/// Sans marque d'ordre d'octets, Excel décode le fichier en ANSI : « Société » devient
/// « SociÃ©tÃ© », c'est-à-dire presque chaque ligne d'un export français.
#[test]
fn le_csv_commence_par_la_marque_utf8() {
    let csv = vers_csv(&[cand("Développeuse Rust", None)]).unwrap();

    assert!(
        csv.starts_with('\u{feff}'),
        "marque d'ordre d'octets absente"
    );
    assert!(csv.contains("Développeuse Rust"));
}

/// Un champ ouvrant par `=` devient une formule à l'ouverture du fichier, y compris chez la
/// personne à qui l'export est transmis. Les notes accueillent des offres collées, donc du
/// texte que l'utilisateur n'a pas écrit lui-même.
#[test]
fn un_champ_ouvrant_une_formule_ne_reste_pas_executable() {
    let csv = vers_csv(&[cand("Poste", Some("=cmd|'/c calc'!A1"))]).unwrap();

    assert!(
        csv.contains("'=cmd"),
        "la formule n'a pas été neutralisée : {csv}"
    );
    assert!(
        !csv.contains(";=cmd"),
        "un champ commence encore par `=` : {csv}"
    );
}
