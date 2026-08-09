//! Cas de test isolé.

use super::*;

#[test]
fn test_update_preserve_l_analyse_ia_enregistree() {
    let repo = repo();
    let cand = candidature(&repo);
    let cree = repo.create(&entree(cand, "2026-03-01T10:00:00Z")).unwrap();
    let analyse = AnalyseEntretien {
        resume: "Bon échange".into(),
        points_forts: vec!["clarté".into()],
        points_faibles: vec![],
        suggestions: vec!["préparer des chiffres".into()],
    };
    repo.enregistrer_analyse(cree.id, &analyse).unwrap();
    let mut modifie = entree(cand, "2026-03-01T10:00:00Z");
    modifie.compte_rendu = Some("Entretien technique de 45 min".into());
    let resultat = repo.update(cree.id, &modifie).unwrap();
    assert_eq!(resultat.analyse_ia, Some(analyse));
}
