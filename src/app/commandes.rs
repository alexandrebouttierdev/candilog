//! Exécution des écritures métier et des rechargements, hors du fil de rendu.

use super::{App, Message};
use iced::Task;

/// Exécute une écriture métier **hors du fil de rendu**.
///
/// `docs/ARCHITECTURE.md` prescrit « Message → app/update.rs → Task Tokio → Service →
/// Repository → SQLite ». Les neuf écritures métier s'exécutaient en réalité de façon
/// synchrone sur le fil de l'interface, chacune suivie d'un rechargement de onze requêtes
/// SQL : le rendu était bloqué pendant toute la séquence. Imperceptible sur une base locale
/// saine, la durée devient arbitraire si le fichier est sur un support lent ou distant, ou si
/// une autre connexion détient un verrou — `busy_timeout` valant 5 secondes, une contention
/// figeait l'interface d'autant, sans le moindre indicateur.
pub(super) fn ecrire<F>(app: &mut App, succes: &'static str, travail: F) -> Task<Message>
where
    F: FnOnce(&crate::shared::state::AppState) -> Result<(), String> + Send + 'static,
{
    let Some(backend) = app.backend.clone() else {
        app.notify_failure("La base Candilog n'est pas disponible.");
        return Task::none();
    };
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || travail(&backend))
                .await
                .unwrap_or_else(|erreur| Err(format!("Opération interrompue : {erreur}")))
        },
        move |result| Message::WriteFinished(result, succes),
    )
}

/// Recharge l'instantané hors du fil de rendu.
pub(super) fn recharger(app: &App) -> Task<Message> {
    let Some(backend) = app.backend.clone() else {
        return Task::none();
    };
    let (llm_page, ats_page) = (app.llm_page, app.ats_page);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                crate::app::state::charger_instantane(&backend, llm_page, ats_page)
            })
            .await
            .ok()
        },
        |charge| match charge {
            Some((data, echecs)) => Message::DataLoaded(Box::new(data), echecs),
            None => Message::Noop,
        },
    )
}

/// Conclut une écriture métier : ferme le dialogue, recharge et confirme.
///
/// N'émet **pas** de notification de bureau. Une notification système était envoyée à chaque
/// opération — y compris un simple déplacement de carte dans le Kanban — alors qu'un toast
/// interne affiche déjà la même information : le doublon était systématique. Elle l'était de
/// surcroît de façon synchrone sur le fil de l'interface, par un appel D-Bus bloquant dont
/// l'échec était écarté : sur une session sans démon de notification, l'appel peut atteindre
/// son délai d'expiration (25 s dans l'implémentation de référence) et figer l'interface
/// d'autant. Voir [`notifier_le_bureau`] pour les opérations qui la justifient.
pub(super) fn finish_submit(app: &mut App, result: Result<(), String>, success: &str) {
    match result {
        Ok(()) => {
            app.dialog = None;
            app.editing_id = None;
            app.notify_success(success);
        }
        Err(error) => app.notify_failure(error),
    }
}

/// Émet une notification de bureau, hors du fil de l'interface.
///
/// Réservée aux opérations **longues** — fin de génération IA, fin de téléchargement d'une mise
/// à jour — c'est-à-dire à celles dont l'utilisateur a pu détourner le regard. L'échec est
/// journalisé au lieu d'être écarté.
pub(super) fn notifier_le_bureau(corps: String) -> Task<Message> {
    Task::perform(
        async move {
            let envoi = tokio::task::spawn_blocking(move || {
                notify_rust::Notification::new()
                    .summary("Candilog")
                    .body(&corps)
                    .show()
                    .map(|_| ())
            })
            .await;
            match envoi {
                Ok(Ok(())) => {}
                Ok(Err(error)) => tracing::warn!(erreur = %error, "notification de bureau refusée"),
                Err(error) => tracing::warn!(erreur = %error, "notification de bureau interrompue"),
            }
        },
        |()| Message::Noop,
    )
}
