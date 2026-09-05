//! Appliquer une proposition applicable met à jour le document et son statut.

use super::*;

#[test]
fn applique_une_suggestion_textuelle() {
    let workspace = workspace_avec_recommandation("Profil actuel", "Profil avec React");
    let updated = apply_proposal(workspace, "ats-0", None).unwrap();
    assert_eq!(updated.document.profile, "Profil avec React");
    assert_eq!(updated.proposals[0].status, ResumeProposalStatus::Accepted);
}
