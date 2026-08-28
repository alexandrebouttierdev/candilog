//! Cas de test isolé.

use super::*;

/// Le calendrier affiche « Développeur Frontend — Nova Digital » sur chaque pastille : sans
/// aplatissement, il faudrait une requête par événement.
#[test]
fn test_le_poste_et_l_entreprise_sont_aplatis() {
    let (repo, application_id) = context();
    let cree = repo
        .save_and_mark_candidate(None, &entree(application_id, "2026-08-25T14:00:00+02:00"))
        .unwrap();

    assert_eq!(
        cree.application_job_title.as_deref(),
        Some("Développeur Frontend")
    );
    assert_eq!(cree.company_name.as_deref(), Some("Nova Digital"));
}
