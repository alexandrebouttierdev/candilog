//! Cas de test isolé.

use super::*;

#[test]
fn test_update_modifie_les_champs_editables() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let creee = repo.create(&entree(ent, "Dev")).unwrap();
    let mut modifiee = entree(ent, "Dev Senior");
    modifiee.type_contrat = TypeContrat::Freelance;
    modifiee.statut = StatutCandidature::Entretien;
    modifiee.date_envoi = "2026-03-01".into();
    modifiee.notes = Some("Recontacté après relance".into());
    let resultat = repo.update(creee.id, &modifiee).unwrap();
    assert_eq!(resultat.id, creee.id);
    assert_eq!(resultat.poste, "Dev Senior");
    assert_eq!(resultat.type_contrat, TypeContrat::Freelance);
    assert_eq!(resultat.statut, StatutCandidature::Entretien);
    assert_eq!(resultat.date_envoi, "2026-03-01");
    assert_eq!(resultat.notes.as_deref(), Some("Recontacté après relance"));
}
