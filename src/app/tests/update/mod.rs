//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::app::state::Dialog;
use crate::app::update::update;
use crate::modules::ia::cv_model::{GeneratedCv, GeneratedExperience, RecommandationAts};

/// Application hermétique pilotée par `Message`, sur une base **en mémoire**.
///
/// `update()` traite une centaine de variantes de `Message` et concentre toute la logique
/// applicative — écritures, rechargements, machine à états des dialogues, pagination,
/// annulation IA — pour deux tests, tous deux portant sur le même sujet. C'est pourtant le
/// levier d'automatisation le plus fiable d'une application Iced : le socle existe
/// (`AppState::new()`, base mémoire, prévue à cet effet), il n'était pas exploité.
#[cfg(test)]
fn app_de_test() -> App {
    let backend = crate::shared::state::AppState::new().unwrap();
    App::with_backend(
        crate::core::config::AppPaths::in_directory(std::path::PathBuf::from("/inexistant")),
        backend,
    )
}

/// Envoie une séquence de messages, en ignorant les `Task` produites.
///
/// Les `Task` portent les effets asynchrones (écritures, rechargements) ; les transitions
/// d'état synchrones qu'on vérifie ici sont, elles, appliquées immédiatement.
#[cfg(test)]
fn envoyer(app: &mut App, messages: impl IntoIterator<Item = Message>) {
    for message in messages {
        let _ = update(app, message);
    }
}

mod la_pagination_reste_dans_ses_bornes;
mod le_changement_de_statut_conserve_la_fiche;
mod le_cycle_de_vie_des_dialogues_reinitialise_les_formulaires;
mod le_glisser_suit_le_seuil_et_le_clic;
mod recommandation_experience_cible_uniquement_la_ligne_demandee;
mod recommandation_resume_modifie_le_modele_commun;
mod un_deplacement_de_5_px_est_un_glisser;
mod une_soumission_invalide_ne_ferme_pas_le_dialogue;
