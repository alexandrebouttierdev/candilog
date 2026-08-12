//! Cas de test isolé.

use super::*;
use crate::app::state::CandidateFilters;
use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use crate::navigation::Route;

/// Une ligne « candidature à relancer » doit rejoindre sa fiche, sans qu'un ancien filtre du
/// kanban puisse masquer la candidature ciblée lors du rechargement paginé.
#[test]
fn la_relance_statistique_ouvre_la_candidature() {
    let mut app = app_de_test();
    let id = uuid::Uuid::new_v4();
    app.data.follow_up_candidates = vec![Candidature {
        id,
        poste: "Développeur Rust".into(),
        entreprise_id: uuid::Uuid::new_v4(),
        entreprise_nom: Some("Candilog".into()),
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-01".into(),
        lien_offre: None,
        notes: None,
        created_at: "2026-08-01T08:00:00Z".into(),
        updated_at: "2026-08-01T08:00:00Z".into(),
    }];
    app.candidate_filters = CandidateFilters {
        status: Some(StatutCandidature::Refus),
        city: "Rennes".into(),
        ..CandidateFilters::default()
    };

    envoyer(&mut app, [Message::OpenCandidateFromStats(id)]);

    assert_eq!(app.route, Route::Candidatures);
    assert_eq!(app.search, "Développeur Rust");
    assert_eq!(app.candidate_filters.active_count(), 0);
    assert_eq!(app.candidate_page, 1);
    assert!(matches!(
        app.dialog,
        Some(Dialog::CandidatureDetail(candidate_id)) if candidate_id == id
    ));
}
