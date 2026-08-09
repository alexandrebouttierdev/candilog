//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_analyse_puis_get_restitue_l_analyse() {
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
    let relu = repo.get(cree.id).unwrap();
    assert_eq!(relu.analyse_ia, Some(analyse));
}
