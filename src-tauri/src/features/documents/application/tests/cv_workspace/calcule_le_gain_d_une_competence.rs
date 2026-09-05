//! Une exigence absente du profil n'est jamais présentée comme contenu ajoutable.

use super::*;

#[test]
fn ne_recommande_pas_une_competence_absente_du_profil() {
    let workspace = workspace_avec_offre(vec!["Rust", "Docker"], vec!["Rust"]);
    assert_eq!(workspace.score.missing, vec!["Rust", "Docker"]);
    assert!(workspace
        .content_recommendations
        .iter()
        .all(|recommendation| recommendation.label != "Docker"));
    assert!(workspace.proposals.is_empty());
}
