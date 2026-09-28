//! Régime du lien de l'offre selon le canal de la candidature.

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
    input.channel = ApplicationChannel::Spontaneous;
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

    input.channel = ApplicationChannel::Spontaneous;
    service.update(uuid::Uuid::nil(), &input).unwrap();

    assert_eq!(service.repository().recu().job_url, None);
}

/// Site de l'entreprise ou réseau : une cooptation n'a pas toujours d'annonce publique.
/// Le lien est facultatif, mais un lien donné reste contrôlé.
#[test]
fn le_site_et_le_reseau_rendent_le_lien_facultatif_mais_valide() {
    let service = ApplicationService::new(StubRepo::default());
    for channel in [ApplicationChannel::CompanySite, ApplicationChannel::Network] {
        let mut input = new("Développeur");
        input.channel = channel;
        input.job_url = None;
        assert!(
            service.create(&input).is_ok(),
            "{channel:?} sans lien refusé"
        );

        input.job_url = Some("javascript:alert(1)".into());
        assert!(
            matches!(service.create(&input), Err(AppError::Validation(_))),
            "{channel:?} : un lien non HTTP aurait dû être refusé"
        );
    }
}

#[test]
fn le_canal_determine_la_nature_de_la_demarche() {
    assert_eq!(
        ApplicationChannel::Spontaneous.application_type(),
        ApplicationType::Unsolicited
    );
    for channel in [
        ApplicationChannel::Offer,
        ApplicationChannel::CompanySite,
        ApplicationChannel::Network,
    ] {
        assert_eq!(channel.application_type(), ApplicationType::JobOffer);
    }
}
