//! Surcharges de candidature : héritage, surcharge, retrait.

use super::*;

/// Sans surcharge, la candidature affiche les valeurs de son entreprise — et la base ne
/// les recopie pas : `applications.city` reste `NULL`.
#[test]
fn sans_surcharge_les_valeurs_sont_heritees_sans_etre_recopiees() {
    let (repo, company_id) = context();

    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    assert_eq!(creee.city, None);
    assert_eq!(creee.address, None);
    assert_eq!(creee.company_type_id, None);
    assert_eq!(creee.effective_city.as_deref(), Some("Rennes"));
    assert_eq!(creee.effective_address.as_deref(), Some("12 rue des Lilas"));
    assert_eq!(
        creee.effective_company_type_id.as_deref(),
        Some("IT_SERVICES_COMPANY")
    );
    assert_eq!(
        creee.effective_company_type_name.as_deref(),
        Some("ESN / Société de services numériques")
    );

    // La colonne elle-même doit rester nulle : une valeur recopiée se figerait, et le
    // changement d'entreprise laisserait derrière lui la ville de la précédente.
    let stockee: Option<String> = connection(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT city FROM applications WHERE id = ?1",
            [creee.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stockee, None);
}

#[test]
fn une_surcharge_prime_sur_la_valeur_de_l_entreprise() {
    let (repo, company_id) = context();
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.city = Some("Nantes".into());
    input.company_type_id = Some("FINAL_CLIENT".into());

    let creee = repo.create(&input).unwrap();

    assert_eq!(creee.city.as_deref(), Some("Nantes"));
    assert_eq!(creee.effective_city.as_deref(), Some("Nantes"));
    assert_eq!(
        creee.effective_company_type_name.as_deref(),
        Some("Client final")
    );
    // L'adresse, elle, n'est pas surchargée : elle reste héritée.
    assert_eq!(creee.address, None);
    assert_eq!(creee.effective_address.as_deref(), Some("12 rue des Lilas"));
}

#[test]
fn retirer_une_surcharge_restitue_la_valeur_de_l_entreprise() {
    let (repo, company_id) = context();
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.city = Some("Nantes".into());
    input.company_type_id = Some("FINAL_CLIENT".into());
    let creee = repo.create(&input).unwrap();

    input.city = None;
    input.company_type_id = None;
    let modifiee = repo.update(creee.id, &input).unwrap();

    assert_eq!(modifiee.city, None);
    assert_eq!(modifiee.company_type_id, None);
    assert_eq!(modifiee.effective_city.as_deref(), Some("Rennes"));
    assert_eq!(
        modifiee.effective_company_type_name.as_deref(),
        Some("ESN / Société de services numériques")
    );
}
