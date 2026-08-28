//! Cas de test isolé.

use super::*;

#[test]
fn test_le_poste_et_l_entreprise_sont_aplatis() {
    let (repo, application_id) = context();

    let creee = repo.create(&entree(application_id, "2026-08-27")).unwrap();

    assert_eq!(creee.application_job_title.as_deref(), Some("Product Designer"));
    assert_eq!(creee.company_name.as_deref(), Some("Atlas Studio"));
}
