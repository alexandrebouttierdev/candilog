//! Régime du lien de l'offre selon la nature de la candidature.

use super::*;

#[test]
fn une_candidature_a_une_offre_exige_son_lien() {
    let service = ApplicationService::new(StubRepo::default());

    for job_url in [None, Some(String::new()), Some("   ".into())] {
        let mut input = new("Développeur");
        input.job_url = job_url.clone();
        assert!(
            matches!(service.create(&input), Err(AppError::Validation(_))),
            "un lien {job_url:?} aurait dû être refusé pour une offre"
        );
    }
}

#[test]
fn une_candidature_spontanee_n_exige_aucun_lien() {
    let service = ApplicationService::new(StubRepo::default());
    let mut input = new("Développeur");
    input.application_type = ApplicationType::Unsolicited;
    input.job_url = None;

    assert!(service.create(&input).is_ok());
}

/// Le passage d'« offre » à « spontanée » efface le lien : conservé, il pointerait vers une
/// annonce sans rapport avec la démarche, et la fiche relue plus tard induirait en erreur.
#[test]
fn le_passage_en_spontanee_efface_le_lien_de_l_offre() {
    let repo = StubRepo::default();
    let service = ApplicationService::new(repo);
    let mut input = new("Développeur");
    input.job_url = Some("https://example.org/offre".into());

    service.update(uuid::Uuid::nil(), &input).unwrap();
    assert_eq!(
        service.repository().recu().job_url.as_deref(),
        Some("https://example.org/offre")
    );

    input.application_type = ApplicationType::Unsolicited;
    service.update(uuid::Uuid::nil(), &input).unwrap();

    assert_eq!(service.repository().recu().job_url, None);
}
