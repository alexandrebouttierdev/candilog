//! Critères inversés (« n'est pas ») : puce cliquable de la barre de filtres v2.

use super::*;
use crate::features::applications::domain::FilterField;

fn titres(repo: &SqliteApplicationRepository, filter: &ApplicationFilter) -> Vec<String> {
    let mut titres: Vec<String> = repo
        .list_page(1, 20, filter)
        .unwrap()
        .items
        .into_iter()
        .map(|item| item.job_title)
        .collect();
    titres.sort();
    titres
}

#[test]
fn un_statut_exclu_retient_tous_les_autres() {
    let (repo, company_id) = context();
    for (titre, status) in [
        ("Alpha", ApplicationStatus::Pending),
        ("Beta", ApplicationStatus::Interview),
        ("Gamma", ApplicationStatus::Rejected),
    ] {
        let mut input = entree(company_id, titre, "2026-08-20");
        input.status = status;
        repo.create(&input).unwrap();
    }

    let filtre = ApplicationFilter {
        status: vec![ApplicationStatus::Rejected],
        excluded: vec![FilterField::Status],
        ..ApplicationFilter::default()
    };
    assert_eq!(titres(&repo, &filtre), vec!["Alpha", "Beta"]);
}

#[test]
fn une_valeur_absente_passe_un_critere_exclu() {
    let (repo, company_id) = context();
    let mut informatique = entree(company_id, "Dev", "2026-08-20");
    informatique.professional_domain_id = Some("M18".into());
    repo.create(&informatique).unwrap();
    // Sans domaine : « n'est pas Informatique » est vrai pour elle.
    repo.create(&entree(company_id, "Sans domaine", "2026-08-20"))
        .unwrap();

    let filtre = ApplicationFilter {
        professional_domain_id: vec!["M18".into()],
        excluded: vec![FilterField::ProfessionalDomain],
        ..ApplicationFilter::default()
    };
    assert_eq!(titres(&repo, &filtre), vec!["Sans domaine"]);
}

#[test]
fn l_exclusion_ne_touche_que_le_critere_designe() {
    let (repo, company_id) = context();
    let mut cdd = entree(company_id, "Ops CDD", "2026-08-20");
    cdd.contract_type_code = "CDD".into();
    repo.create(&cdd).unwrap();
    let mut refus = entree(company_id, "Ops refus", "2026-08-20");
    refus.status = ApplicationStatus::Rejected;
    repo.create(&refus).unwrap();
    repo.create(&entree(company_id, "Ops CDI", "2026-08-20"))
        .unwrap();

    // Contrat CDI (condition directe) ET statut qui n'est pas Refusée (condition inversée).
    let filtre = ApplicationFilter {
        contract_type_code: vec!["CDI".into()],
        status: vec![ApplicationStatus::Rejected],
        excluded: vec![FilterField::Status],
        ..ApplicationFilter::default()
    };
    assert_eq!(titres(&repo, &filtre), vec!["Ops CDI"]);
}

#[test]
fn une_ville_ou_un_intitule_exclus_ecartent_les_correspondances() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Développeur Rust", "2026-08-20"))
        .unwrap();
    repo.create(&entree(company_id, "Technicien support", "2026-08-20"))
        .unwrap();

    let filtre = ApplicationFilter {
        job_title: "develop".into(),
        excluded: vec![FilterField::JobTitle],
        ..ApplicationFilter::default()
    };
    assert_eq!(titres(&repo, &filtre), vec!["Technicien support"]);

    // L'entreprise du contexte est à Rennes : exclure Rennes vide la liste.
    let filtre = ApplicationFilter {
        city: "rennes".into(),
        excluded: vec![FilterField::City],
        ..ApplicationFilter::default()
    };
    assert!(titres(&repo, &filtre).is_empty());
}

#[test]
fn une_entreprise_exclue_ecarte_ses_candidatures() {
    let (repo, company_id) = context();
    repo.create(&entree(company_id, "Chez nous", "2026-08-20"))
        .unwrap();

    let filtre = ApplicationFilter {
        company_id: Some(company_id),
        excluded: vec![FilterField::Company],
        ..ApplicationFilter::default()
    };
    assert!(titres(&repo, &filtre).is_empty());
}
