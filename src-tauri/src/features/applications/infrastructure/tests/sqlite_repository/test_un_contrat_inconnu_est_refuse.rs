//! Le code de contrat est une clé étrangère : une valeur hors référentiel est refusée.

use super::*;

#[test]
fn un_code_de_contrat_hors_referentiel_retourne_une_phrase_lisible() {
    let (repo, company_id) = context();
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.contract_type_code = "INEXISTANT".into();

    match repo.create(&input) {
        Err(AppError::Validation(message)) => assert!(message.contains("introuvable")),
        other => panic!("attendu Validation, obtenu {other:?}"),
    }
}

#[test]
fn un_domaine_professionnel_hors_referentiel_retourne_une_phrase_lisible() {
    let (repo, company_id) = context();
    let mut input = entree(company_id, "Développeur", "2026-08-20");
    input.professional_domain_id = Some("ZZ99".into());

    assert!(matches!(repo.create(&input), Err(AppError::Validation(_))));
}
