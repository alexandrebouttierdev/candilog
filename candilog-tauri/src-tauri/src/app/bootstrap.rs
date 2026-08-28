//! Construction et lancement de l'application Tauri.

use crate::app::state::AppState;
use crate::features::analyses::presentation::commands as analyses;
use crate::features::candidatures::presentation::commands as candidatures;
use crate::features::contacts::presentation::commands as contacts;
use crate::features::documents::presentation::commands as documents;
use crate::features::entreprises::presentation::commands as entreprises;
use crate::features::entretiens::presentation::commands as entretiens;
use crate::features::ia::presentation::commands as ia;
use crate::features::profil::presentation::commands as profil;
use crate::features::relances::presentation::commands as relances;
use crate::features::secteurs::presentation::commands as secteurs;

/// Démarre Candilog : journal, état applicatif, plugins, commandes.
///
/// Sans base de données lisible il n'y a rien à afficher : l'application s'arrête en
/// journalisant la cause plutôt que d'ouvrir une fenêtre vide dont l'utilisateur ne pourrait
/// rien tirer. C'est le seul point du programme où un arrêt est le bon comportement.
pub fn run() {
    let _garde = crate::core::logging::initialiser();

    let state = match AppState::persistent() {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, "état applicatif non initialisable");
            eprintln!(
                "Candilog n'a pas pu ouvrir ses données : {}",
                error.message_utilisateur()
            );
            std::process::exit(1);
        }
    };

    if let Err(error) = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            analyses::analyses_tableau_de_bord,
            analyses::analyses_charger,
            analyses::analyses_exporter_csv,
            candidatures::candidatures_lister_page,
            candidatures::candidatures_repartition,
            candidatures::candidatures_obtenir,
            candidatures::candidatures_creer,
            candidatures::candidatures_modifier,
            candidatures::candidatures_changer_statut,
            candidatures::candidatures_supprimer,
            candidatures::candidatures_exporter_csv,
            entreprises::entreprises_lister,
            entreprises::entreprises_lister_page,
            entreprises::entreprises_lister_types,
            entreprises::entreprises_obtenir,
            entreprises::entreprises_creer,
            entreprises::entreprises_modifier,
            entreprises::entreprises_supprimer,
            contacts::contacts_lister,
            contacts::contacts_lister_page,
            contacts::contacts_obtenir,
            contacts::contacts_creer,
            contacts::contacts_modifier,
            contacts::contacts_supprimer,
            documents::documents_cv_lister,
            documents::documents_cv_obtenir,
            documents::documents_cv_enregistrer,
            documents::documents_cv_supprimer,
            documents::documents_lettres_lister,
            documents::documents_lettre_obtenir,
            documents::documents_lettre_enregistrer,
            documents::documents_lettre_supprimer,
            entretiens::entretiens_lister_entre,
            entretiens::entretiens_obtenir,
            entretiens::entretiens_enregistrer,
            entretiens::entretiens_supprimer,
            ia::ia_analyser_offre,
            ia::ia_generer_cv,
            ia::ia_generer_lettre,
            ia::ia_analyser_cv,
            ia::ia_importer_profil,
            ia::ia_annuler,
            profil::profil_charger,
            profil::profil_enregistrer,
            relances::relances_lister_entre,
            relances::relances_creer,
            relances::relances_modifier,
            relances::relances_supprimer,
            secteurs::secteurs_lister,
        ])
        .run(tauri::generate_context!())
    {
        // `run` n'échoue qu'à l'initialisation du moteur de rendu système : rien que
        // l'application puisse corriger, mais la cause doit rester dans le journal.
        tracing::error!(%error, "démarrage de la fenêtre impossible");
        std::process::exit(1);
    }
}
