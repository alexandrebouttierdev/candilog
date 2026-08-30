//! Le catalogue des domaines professionnels est semé en entier, sans doublon, et trié.

use super::*;

#[test]
fn les_vingt_deux_domaines_sont_presents_avec_leur_libelle() {
    let domains = repo().load().unwrap().professional_domains;

    assert_eq!(domains.len(), 22);
    assert_eq!(
        label(&domains, "M18").as_deref(),
        Some("Informatique / Télécommunication")
    );
    assert_eq!(
        label(&domains, "M").as_deref(),
        Some("Achats / Comptabilité / Gestion")
    );
    assert_eq!(label(&domains, "C15").as_deref(), Some("Immobilier"));
    assert_eq!(label(&domains, "L14").as_deref(), Some("Sport"));
    assert_eq!(
        label(&domains, "N").as_deref(),
        Some("Transport / Logistique")
    );
}

#[test]
fn aucun_code_ni_libelle_n_est_duplique() {
    let domains = repo().load().unwrap().professional_domains;

    let mut codes: Vec<&str> = domains.iter().map(|item| item.code.as_str()).collect();
    codes.sort_unstable();
    let attendus = codes.len();
    codes.dedup();
    assert_eq!(codes.len(), attendus, "code de domaine dupliqué");

    let mut names: Vec<&str> = domains.iter().map(|item| item.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), attendus, "libellé de domaine dupliqué");
}

#[test]
fn l_ordre_d_affichage_suit_le_referentiel_et_non_l_ordre_d_insertion() {
    let domains = repo().load().unwrap().professional_domains;

    let codes: Vec<&str> = domains.iter().map(|item| item.code.as_str()).collect();
    assert_eq!(codes.first().copied(), Some("M"));
    assert_eq!(codes.get(1).copied(), Some("B"));
    assert_eq!(codes.last().copied(), Some("N"));
}
