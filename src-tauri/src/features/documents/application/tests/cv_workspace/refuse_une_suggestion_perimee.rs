//! Une proposition dont la cible a été modifiée manuellement n'est plus applicable.

use super::*;
use crate::core::errors::AppError;

#[test]
fn refuse_une_suggestion_perimee() {
    let mut workspace = workspace_avec_recommandation("Profil actuel", "Profil avec React");
    workspace.document.profile = "Texte retouché".into();
    let error = apply_proposal(workspace, "ats-0").unwrap_err();
    assert!(matches!(error, AppError::Validation(_)));
}
