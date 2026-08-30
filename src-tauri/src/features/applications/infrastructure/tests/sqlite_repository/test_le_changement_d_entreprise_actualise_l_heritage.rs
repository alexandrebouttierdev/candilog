//! Changer d'entreprise doit actualiser les valeurs héritées, et elles seules.

use super::*;

#[test]
fn les_valeurs_heritees_suivent_la_nouvelle_entreprise() {
    let (repo, premiere) = context();
    let seconde = autre_entreprise(&repo, "Atlas Studio", "Nantes", "FINAL_CLIENT");
    let creee = repo
        .create(&entree(premiere, "Développeur", "2026-08-20"))
        .unwrap();
    assert_eq!(creee.effective_city.as_deref(), Some("Rennes"));

    let mut input = entree(seconde, "Développeur", "2026-08-20");
    input.city = None;
    let modifiee = repo.update(creee.id, &input).unwrap();

    assert_eq!(modifiee.effective_city.as_deref(), Some("Nantes"));
    assert_eq!(
        modifiee.effective_company_type_name.as_deref(),
        Some("Client final")
    );
    assert_eq!(modifiee.city, None, "la nouvelle ville a été recopiée");
}

#[test]
fn une_surcharge_explicite_survit_au_changement_d_entreprise() {
    let (repo, premiere) = context();
    let seconde = autre_entreprise(&repo, "Atlas Studio", "Nantes", "FINAL_CLIENT");
    let mut input = entree(premiere, "Développeur", "2026-08-20");
    input.city = Some("Brest".into());
    let creee = repo.create(&input).unwrap();

    input.company_id = seconde;
    let modifiee = repo.update(creee.id, &input).unwrap();

    assert_eq!(modifiee.city.as_deref(), Some("Brest"));
    assert_eq!(modifiee.effective_city.as_deref(), Some("Brest"));
}
