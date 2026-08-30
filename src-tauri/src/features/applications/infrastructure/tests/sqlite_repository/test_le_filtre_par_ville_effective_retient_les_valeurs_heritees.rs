//! Les filtres portent sur la valeur effective, surcharge et héritage confondus.

use super::*;
use crate::features::companies::domain::CompanySize;

#[test]
fn la_ville_de_l_entreprise_rend_la_candidature_filtrable() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    let page = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                city: "rennes".into(),
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 1, "une ville héritée doit rester filtrable");
}

#[test]
fn une_surcharge_de_ville_sort_la_candidature_du_filtre_herite() {
    let (repo, company_id) = context();
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.city = Some("Nantes".into());
    repo.create(&input).unwrap();

    let rennes = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                city: "Rennes".into(),
                ..ApplicationFilter::default()
            },
        )
        .unwrap();
    let nantes = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                city: "Nantes".into(),
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(rennes.total, 0);
    assert_eq!(nantes.total, 1);
}

#[test]
fn le_filtre_par_type_d_entreprise_couvre_l_heritage_et_la_surcharge() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Hérité", "2026-08-20"))
        .unwrap();
    let mut surcharge = entree(company_id, "Surchargé", "2026-08-20");
    surcharge.company_type_id = Some("FINAL_CLIENT".into());
    repo.create(&surcharge).unwrap();

    let esn = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                company_type_id: vec!["IT_SERVICES_COMPANY".into()],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();
    let client = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                company_type_id: vec!["FINAL_CLIENT".into()],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(esn.total, 1);
    assert_eq!(esn.items[0].job_title, "Hérité");
    assert_eq!(client.total, 1);
    assert_eq!(client.items[0].job_title, "Surchargé");
}

/// La taille appartient à l'entreprise : aucune surcharge n'existe côté candidature, le
/// filtre passe donc par la table `companies`.
#[test]
fn le_filtre_par_taille_passe_par_l_entreprise_liee() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    let pme = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                company_size: vec![CompanySize::Pme],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();
    let eti = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                company_size: vec![CompanySize::Eti],
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(pme.total, 1);
    assert_eq!(eti.total, 0);
}

#[test]
fn les_filtres_se_cumulent() {
    let (repo, company_id) = context();
    let mut interim = entree(company_id, "Intérimaire", "2026-08-20");
    interim.contract_type_code = "MIS".into();
    interim.professional_domain_id = Some("M18".into());
    interim.application_type = ApplicationType::Unsolicited;
    interim.job_url = None;
    repo.create(&interim).unwrap();
    repo.create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    let page = repo
        .list_page(
            1,
            10,
            &ApplicationFilter {
                contract_type_code: vec!["MIS".into()],
                professional_domain_id: vec!["M18".into()],
                application_type: vec![ApplicationType::Unsolicited],
                company_size: vec![CompanySize::Pme],
                city: "Rennes".into(),
                ..ApplicationFilter::default()
            },
        )
        .unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].job_title, "Intérimaire");
}
