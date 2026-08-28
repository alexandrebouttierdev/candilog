//! Cas de test isolé.

use super::*;

/// Le changement de statut ne repasse pas par la validation des champs : c'est le geste du
/// glisser-déposer, qui ne touche à rien d'autre, et exiger une date valide y ferait échouer
/// le déplacement d'une candidature héritée mal formatée.
#[test]
fn test_changer_statut_delegue_au_depot() {
    let service = ApplicationService::new(StubRepo);
    let apres = service
        .change_status(uuid::Uuid::nil(), ApplicationStatus::Interview)
        .unwrap();

    assert_eq!(apres.status, ApplicationStatus::Interview);
}
