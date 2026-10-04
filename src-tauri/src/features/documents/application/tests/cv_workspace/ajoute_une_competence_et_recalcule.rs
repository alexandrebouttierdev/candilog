//! La boucle que l'éditeur fait tourner à chaque « Ajouter » : le document change, le score
//! est recalculé dessus.
//!
//! Elle n'était couverte nulle part. Les tests voisins vérifient la composition initiale
//! (`reprend_les_competences_du_profil`) et le gain simulé d'une proposition
//! (`calcule_le_gain_d_une_competence`), mais aucun ne constatait que le score **monte**
//! réellement après l'ajout — or c'est le seul chemin par lequel un CV sort de son score
//! d'ouverture, qui est bas par construction puisque les compétences commencent dans la
//! bibliothèque (`docs/AI.md`).

use super::*;
use crate::features::documents::application::recalculate;
use crate::features::documents::domain::ResumeSkillGroup;

/// Reproduit `insertProfileItem` de `model/resumeWorkspace.ts`, côté document : l'éditeur
/// insère dans le premier groupe, et en crée un quand le document n'en a aucun.
fn inserer_competences(workspace: &mut ResumeWorkspace, noms: &[&str]) {
    match workspace.document.skill_groups.first_mut() {
        Some(groupe) => groupe
            .items
            .extend(noms.iter().map(|nom| (*nom).to_owned())),
        None => workspace.document.skill_groups.push(ResumeSkillGroup {
            id: "profile-skills".into(),
            name: "Compétences".into(),
            items: noms.iter().map(|nom| (*nom).to_owned()).collect(),
        }),
    }
}

#[test]
fn le_document_s_ouvre_sans_competences_mais_les_propose() {
    let workspace = workspace_avec_offre(vec!["Rust", "Docker"], vec!["Rust", "Docker"]);

    // Le socle initial ne porte pas les compétences : elles restent dans la bibliothèque
    // tant que l'utilisateur ne les choisit pas. C'est voulu, et c'est aussi la raison pour
    // laquelle un profil parfaitement adapté ouvre l'éditeur sur un score bas.
    assert!(
        workspace
            .document
            .skill_groups
            .iter()
            .all(|groupe| groupe.items.is_empty()),
        "le socle ne doit porter aucune compétence, trouvé {:?}",
        workspace.document.skill_groups
    );
    assert!(
        !workspace.content_recommendations.is_empty(),
        "les compétences de l'offre présentes au profil doivent être proposées"
    );
}

#[test]
fn ajouter_les_competences_recommandees_fait_monter_le_score() {
    let workspace = workspace_avec_offre(vec!["Rust", "Docker"], vec!["Rust", "Docker"]);
    let ouverture = workspace.score.total;

    let mut suivant = workspace.clone();
    inserer_competences(&mut suivant, &["Rust", "Docker"]);
    let recalcule = recalculate(suivant, None).unwrap();

    assert!(
        recalcule.score.total > ouverture,
        "le score doit monter après l'ajout : {ouverture} → {}",
        recalcule.score.total
    );
    // Les deux exigences de l'offre sont désormais couvertes par le document.
    assert!(
        recalcule.score.missing.is_empty(),
        "plus aucune exigence ne doit manquer, trouvé {:?}",
        recalcule.score.missing
    );
}

#[test]
fn une_competence_ajoutee_sort_des_recommandations() {
    let workspace = workspace_avec_offre(vec!["Rust", "Docker"], vec!["Rust", "Docker"]);

    let mut suivant = workspace.clone();
    inserer_competences(&mut suivant, &["Rust"]);
    let recalcule = recalculate(suivant, None).unwrap();

    assert!(
        recalcule
            .content_recommendations
            .iter()
            .all(|recommandation| recommandation.label != "Rust"),
        "une compétence déjà dans le document ne doit plus être proposée"
    );
    assert!(
        recalcule
            .content_recommendations
            .iter()
            .any(|recommandation| recommandation.label == "Docker"),
        "celle qui reste hors du document doit continuer d'être proposée"
    );
}
