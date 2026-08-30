//! Le catalogue des types d'entreprise est semé en entier, sans doublon.

use super::*;

#[test]
fn les_trente_huit_types_sont_presents_avec_leur_libelle() {
    let types = repo().load().unwrap().company_types;

    assert_eq!(types.len(), 38);
    assert_eq!(
        label(&types, "FINAL_CLIENT").as_deref(),
        Some("Client final")
    );
    assert_eq!(
        label(&types, "IT_SERVICES_COMPANY").as_deref(),
        Some("ESN / Société de services numériques")
    );
    assert_eq!(
        label(&types, "PUBLIC_INSTITUTION").as_deref(),
        Some("Établissement public")
    );
    assert_eq!(
        label(&types, "UNIVERSITY").as_deref(),
        Some("Université / Enseignement supérieur")
    );
    assert_eq!(label(&types, "ASSOCIATION").as_deref(), Some("Association"));
    assert_eq!(label(&types, "OTHER").as_deref(), Some("Autre"));
}

#[test]
fn aucun_code_n_est_duplique_et_l_ordre_est_stable() {
    let types = repo().load().unwrap().company_types;

    let mut codes: Vec<&str> = types.iter().map(|item| item.code.as_str()).collect();
    let attendus = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), attendus, "code de type d'entreprise dupliqué");

    assert_eq!(
        types.first().map(|item| item.code.as_str()),
        Some("FINAL_CLIENT")
    );
    assert_eq!(types.last().map(|item| item.code.as_str()), Some("OTHER"));
}
