//! Nature de l'organisation et taille sont deux axes indépendants.

use super::*;

/// « ESN + PME », « Éditeur SaaS + grande entreprise », « Association + TPE » : toutes les
/// combinaisons doivent être exprimables, ce qu'un enum unique interdirait.
#[test]
fn toutes_les_combinaisons_de_type_et_de_taille_sont_exprimables() {
    let repo = repo();

    for (name, company_type_id, company_size) in [
        ("Alpha", "IT_SERVICES_COMPANY", CompanySize::Pme),
        ("Beta", "SAAS_COMPANY", CompanySize::Large),
        ("Gamma", "ASSOCIATION", CompanySize::Tpe),
        ("Delta", "PUBLIC_INSTITUTION", CompanySize::Eti),
        ("Epsilon", "SELF_EMPLOYED", CompanySize::Micro),
    ] {
        let mut entree = entree(name);
        entree.company_type_id = Some(company_type_id.into());
        entree.company_size = company_size;

        let creee = repo.create(&entree).unwrap();

        assert_eq!(creee.company_type_id.as_deref(), Some(company_type_id));
        assert_eq!(creee.company_size, company_size);
    }
}

#[test]
fn le_filtre_par_taille_ne_depend_pas_du_type() {
    let repo = repo();
    let mut esn_pme = entree("Alpha");
    esn_pme.company_size = CompanySize::Pme;
    let mut esn_grande = entree("Beta");
    esn_grande.company_size = CompanySize::Large;
    repo.create(&esn_pme).unwrap();
    repo.create(&esn_grande).unwrap();

    let page = repo
        .list_page(
            1,
            24,
            &CompanyFilter {
                company_type_id: Some("IT_SERVICES_COMPANY".into()),
                company_size: Some(CompanySize::Large),
                ..CompanyFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].name, "Beta");
}

#[test]
fn un_type_hors_referentiel_est_refuse() {
    let repo = repo();
    let mut entree = entree("ACME");
    entree.company_type_id = Some("INEXISTANT".into());

    assert!(matches!(repo.create(&entree), Err(AppError::Validation(_))));
}
