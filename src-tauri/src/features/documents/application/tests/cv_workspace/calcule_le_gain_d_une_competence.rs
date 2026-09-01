//! Le gain d'une compétence manquante mesure l'effet réel de son ajout sur le score.

use super::*;

#[test]
fn calcule_le_gain_d_une_competence() {
    let workspace = workspace_avec_offre(vec!["Rust", "Docker"], vec!["Rust"]);
    let docker = workspace
        .proposals
        .iter()
        .find(|p| p.proposed_text == "Docker")
        .unwrap();
    assert_eq!(docker.kind, ResumeProposalKind::MissingSkill);
    assert!(docker.gain > 0);
}
