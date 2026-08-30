//! Les codes persistés ne remontent jamais seuls : leur libellé français les accompagne.

use super::*;

#[test]
fn le_contrat_et_le_domaine_remontent_avec_leur_libelle() {
    let (repo, company_id) = context();
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.contract_type_code = "MIS".into();
    input.professional_domain_id = Some("M18".into());

    let creee = repo.create(&input).unwrap();

    assert_eq!(creee.contract_type_code, "MIS");
    assert_eq!(creee.contract_type_name.as_deref(), Some("Intérim"));
    assert_eq!(creee.professional_domain_id.as_deref(), Some("M18"));
    assert_eq!(
        creee.professional_domain_name.as_deref(),
        Some("Informatique / Télécommunication")
    );
}

/// Le domaine professionnel décrit le poste, le secteur décrit l'entreprise : un domaine
/// absent reste absent, il n'est jamais déduit du secteur.
#[test]
fn un_domaine_absent_n_est_pas_deduit_du_secteur_de_l_entreprise() {
    let (repo, company_id) = context();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "UPDATE companies SET sector_id = '5ec70000-0000-4000-8000-000000000003'
             WHERE id = ?1",
            [company_id.to_string()],
        )
        .unwrap();

    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    assert_eq!(creee.professional_domain_id, None);
    assert_eq!(creee.professional_domain_name, None);
}

/// La taille appartient à l'entreprise : la candidature l'expose sans la dupliquer.
#[test]
fn la_taille_de_l_entreprise_est_aplatie_depuis_la_jointure() {
    let (repo, company_id) = context();

    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    assert_eq!(
        creee.company_size,
        crate::features::companies::domain::CompanySize::Pme
    );
}
