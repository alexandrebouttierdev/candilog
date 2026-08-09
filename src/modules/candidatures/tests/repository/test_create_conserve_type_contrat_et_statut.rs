//! Cas de test isolé.

use super::*;

#[test]
fn test_create_conserve_type_contrat_et_statut() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let mut entree = entree(ent, "Dev");
    entree.type_contrat = TypeContrat::Freelance;
    entree.statut = StatutCandidature::Entretien;
    let creee = repo.create(&entree).unwrap();
    assert_eq!(creee.type_contrat, TypeContrat::Freelance);
    assert_eq!(creee.statut, StatutCandidature::Entretien);
}
