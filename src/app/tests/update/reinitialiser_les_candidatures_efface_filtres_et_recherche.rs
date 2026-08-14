use super::*;

#[test]
fn reinitialiser_les_candidatures_efface_filtres_et_recherche() {
    let mut app = app_de_test();
    app.search = "administrateur".into();
    app.candidate_filters.city = "Rennes".into();
    app.candidate_filters.contract =
        Some(crate::modules::candidatures::model::TypeContrat::Alternance);

    envoyer(&mut app, [Message::ResetCandidateFilters]);

    assert!(app.search.is_empty());
    assert_eq!(app.candidate_filters.active_count(), 0);
    assert_eq!(app.candidate_page, 1);
}
