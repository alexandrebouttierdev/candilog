//! Cas de test isolé.

use super::*;

/// Le changement de statut ne repasse pas par la validation des champs : c'est le geste du
/// glisser-déposer, qui ne touche à rien d'autre, et exiger une date valide y ferait échouer
/// le déplacement d'une candidature héritée mal formatée.
#[test]
fn test_changer_statut_delegue_au_depot() {
    let service = CandidatureService::new(StubRepo);
    let apres = service
        .changer_statut(uuid::Uuid::nil(), StatutCandidature::Entretien)
        .unwrap();

    assert_eq!(apres.statut, StatutCandidature::Entretien);
}
