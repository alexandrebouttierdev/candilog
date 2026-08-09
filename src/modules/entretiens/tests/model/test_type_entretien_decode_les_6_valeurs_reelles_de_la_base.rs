//! Cas de test isolé.

use super::*;

#[test]
fn test_type_entretien_decode_les_6_valeurs_reelles_de_la_base() {
    // Libellés exacts de l'enum Postgres `type_entretien` (enum_range).
    for label in [
        "Présentiel",
        "Visio",
        "Téléphonique",
        "Technique",
        "RH",
        "Autre",
    ] {
        let json = format!("\"{label}\"");
        let val: TypeEntretien =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("décodage de {label} : {e}"));
        // Round-trip : la sérialisation doit reproduire le même libellé.
        assert_eq!(
            serde_json::to_string(&val).unwrap(),
            json,
            "round-trip {label}"
        );
    }
}
