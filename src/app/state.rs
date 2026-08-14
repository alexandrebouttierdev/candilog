//! État global de l'application native.

mod data;
mod forms;
mod lifecycle;
mod persistence;
mod queries;
mod runtime;
mod types;

pub use data::DataSnapshot;
pub use forms::{
    CandidateFilters, CandidateSort, CandidatureForm, ContactForm, EntrepriseForm, EntretienForm,
    RecommendationStatus, RelanceForm, SettingsForm, StatisticsTab,
};
pub use runtime::{capture_demandee, Notification, NotificationKind};
pub use types::{DatePickerState, DatePickerTarget, Dialog, ProfileCollection, ProfileSection};

use crate::core::config::AppPaths;
use crate::modules::candidatures::model::Candidature;
use crate::modules::candidatures::model::StatutCandidature;
use crate::modules::contacts::model::Contact;
use crate::modules::cv::model::CvVersionSummary;
use crate::modules::entreprises::model::Entreprise;
use crate::modules::ia::cv_model::{CvGeneration, OfferAnalysis};
use crate::navigation::Route;
use crate::shared::error::AppError;
use crate::shared::state::AppState as BackendState;

/// Durée d'affichage d'un toast avant disparition automatique.
pub const DURATION_AFFICHAGE_TOAST: std::time::Duration = std::time::Duration::from_secs(4);
use chrono::{Datelike, Local};
use std::sync::Arc;

use super::message::{CalendarView, CandidateView, Message};
pub use super::snapshot::{
    charger_instantane, SnapshotRequest, BUSINESS_PAGE_SIZE, RELATION_PAGE_SIZE,
};

/// État global rendu par Iced.
pub struct App {
    /// Route active.
    pub route: Route,
    /// Présentation du pipeline.
    pub candidate_view: CandidateView,
    /// Filtres avancés partagés par le Kanban, la liste et l'export.
    pub candidate_filters: CandidateFilters,
    /// Recherche de la page courante.
    pub search: String,
    /// Type d'entreprise retenu dans le répertoire Relations.
    pub company_type_filter: Option<String>,
    /// Année du calendrier.
    pub calendar_year: i32,
    /// Mois du calendrier, de 1 à 12.
    pub calendar_month: u32,
    /// Date pivot du calendrier semaine/jour.
    pub calendar_date: chrono::NaiveDate,
    /// Granularité calendrier active.
    pub calendar_view: CalendarView,
    /// Thème sombre actif.
    pub is_dark: bool,
    /// Chemins persistants résolus.
    pub paths: Option<AppPaths>,
    /// Services métier et base SQLite.
    pub backend: Option<Arc<BackendState>>,
    /// Instantané courant des données.
    pub data: DataSnapshot,
    /// Instantané de données initial chargé.
    pub initialized: bool,
    /// Erreur bloquante d'initialisation.
    pub fatal_error: Option<String>,
    /// Formulaire d'édition des paramètres, séparé de l'instantané persisté.
    pub settings_form: SettingsForm,
    /// Dernier thème système détecté, `None` tant que le système ne s'est pas prononcé.
    pub system_dark: Option<bool>,
    /// Numéro de séquence de l'opération IA en cours.
    ///
    /// Un seul couple `ai_is_running` / `ai_cancellation` est partagé par les six opérations
    /// IA. Sans identifiant, le résultat d'une opération abandonnée remettait
    /// `ai_is_running` à `false` et effaçait `ai_cancellation` quelle que soit l'opération
    /// réellement en cours : lancer une analyse d'offre, naviguer ailleurs, y lancer une
    /// extraction de profil, puis voir arriver le résultat de la première faisait disparaître
    /// l'indicateur d'activité et le bouton d'annulation de la seconde — dont le jeton
    /// devenait inaccessible.
    pub ai_sequence: u64,
    /// Notification utilisateur non bloquante.
    pub notification: Option<Notification>,
    /// Instant de pose de la notification courante, source du compte à rebours
    /// automatique. `None` quand aucun toast n'est affiché.
    pub notification_shown_at: Option<std::time::Instant>,
    /// Résultat du dernier contrôle de santé du fournisseur IA.
    ///
    /// Conservé dans l'état plutôt que déduit de l'activité de l'interface : la pastille du
    /// bandeau de titre doit refléter ce qui a été **mesuré**, pas le fait qu'aucune opération
    /// ne tourne. `Unknown` tant qu'aucun contrôle n'a abouti.
    pub provider_health: crate::ui::components::runtime_status::Health,
    /// Modèles réellement annoncés par le fournisseur configuré.
    pub available_models: Vec<String>,
    /// Mise à jour disponible.
    pub available_update: Option<crate::core::updater::UpdateInfo>,
    /// Progression du téléchargement de mise à jour.
    pub update_progress: Option<u8>,
    /// Backup sélectionné avant confirmation de restauration.
    pub pending_backup_import: Option<std::path::PathBuf>,
    /// Dialogue métier ouvert.
    pub dialog: Option<Dialog>,
    /// Une écriture métier ou de maintenance est en cours.
    pub write_in_progress: bool,
    /// Calendrier flottant au-dessus du formulaire ou des filtres.
    pub date_picker: Option<DatePickerState>,
    /// Identifiant édité par le formulaire courant, absent en création.
    pub editing_id: Option<uuid::Uuid>,
    /// Formulaire entreprise.
    pub entreprise_form: EntrepriseForm,
    /// Formulaire contact.
    pub contact_form: ContactForm,
    /// Formulaire candidature.
    pub candidature_form: CandidatureForm,
    /// Formulaire entretien.
    pub entretien_form: EntretienForm,
    /// Formulaire relance.
    pub relance_form: RelanceForm,
    /// Offre saisie dans le générateur.
    pub offer_editor: iced::widget::text_editor::Content,
    /// Analyse structurée de l'offre courante.
    pub offer_analysis: Option<OfferAnalysis>,
    /// Dernière génération de CV.
    pub cv_generation: Option<CvGeneration>,
    /// Génération chargée uniquement pour l'aperçu de la bibliothèque.
    pub cv_preview_generation: Option<CvGeneration>,
    /// Nom de la version de CV à sauvegarder.
    pub cv_version_name: String,
    /// Décisions associées aux recommandations ATS courantes.
    pub recommendation_states: Vec<RecommendationStatus>,
    /// Opération IA actuellement active.
    pub ai_is_running: bool,
    /// Durée affichée de l'opération IA.
    pub ai_elapsed_seconds: u64,
    /// Jeton d'annulation de l'opération IA courante.
    pub ai_cancellation: Option<tokio_util::sync::CancellationToken>,
    /// Offre utilisée pour analyser un CV externe.
    pub import_offer_editor: iced::widget::text_editor::Content,
    /// PDF externe choisi par l'utilisateur.
    pub import_pdf_path: Option<std::path::PathBuf>,
    /// Dernière analyse de CV importé.
    pub imported_cv_analysis: Option<crate::modules::ia::service::ImportedCvAnalysis>,
    /// Entreprise ciblée par la lettre.
    pub letter_company: String,
    /// Poste ciblé par la lettre.
    pub letter_job_title: String,
    /// Offre ou instruction de la lettre.
    pub letter_editor: iced::widget::text_editor::Content,
    /// Ton de la lettre (`formal`, `casual`, `creative`).
    pub letter_tone: String,
    /// Longueur de la lettre.
    pub letter_length: String,
    /// Texte reçu progressivement du provider.
    pub letter_output: String,
    /// Consigne courante de réécriture de la lettre.
    pub letter_iteration_instruction: String,
    /// Historique des consignes déjà appliquées à la lettre.
    pub letter_chat_history: Vec<crate::modules::ia::cv_model::ChatMsg>,
    /// Candidature sur laquelle un appui gauche est posé, avant le seuil de glisser.
    pub press_candidate: Option<uuid::Uuid>,
    /// Position du curseur à l'origine d'un potentiel glisser.
    pub press_origin: Option<iced::Point>,
    /// Candidature actuellement glissée dans le Kanban.
    pub dragging_candidate: Option<uuid::Uuid>,
    /// Colonne survolée pendant le glisser-déposer.
    pub drag_target_status: Option<StatutCandidature>,
    /// Carte de pipeline survolée (surbrillance au survol).
    pub hovered_card: Option<uuid::Uuid>,
    /// Informations personnelles éditées.
    pub profile_personal_form: crate::shared::profile::PersonalInfo,
    /// Copie structurée des collections du profil pendant l'édition.
    pub profile_draft: crate::shared::profile::Profile,
    /// Résumé édité dans une zone multiligne.
    pub profile_summary_editor: iced::widget::text_editor::Content,
    /// Nouvelle compétence à ajouter sous forme de jeton.
    pub profile_skills_form: String,
    /// PDF choisi pour importer le profil.
    pub profile_import_path: Option<std::path::PathBuf>,
    /// Profil extrait en attente de validation explicite.
    pub extracted_profile: Option<crate::shared::profile::Profile>,
    /// Entrées extraites explicitement refusées par l'utilisateur.
    pub profile_import_excluded: std::collections::HashSet<String>,
    /// Candidature sélectionnée dans la liste ou le pipeline.
    pub selected_candidate: Option<uuid::Uuid>,
    /// Entreprise sélectionnée dans le répertoire.
    pub selected_company: Option<uuid::Uuid>,
    /// Contact sélectionné dans le réseau.
    pub selected_contact: Option<uuid::Uuid>,
    /// Version de CV sélectionnée dans la bibliothèque.
    pub selected_cv: Option<uuid::Uuid>,
    /// Lettre sélectionnée dans sa bibliothèque.
    pub selected_letter: Option<uuid::Uuid>,
    /// Feuille de filtres dépliée.
    pub filters_open: bool,
    /// Colonne de tri de la vue Liste.
    pub candidate_sort: CandidateSort,
    /// Sens de tri de la vue Liste.
    pub candidate_sort_descending: bool,
    /// Pages des trois répertoires métier, indexées à partir de 1.
    pub candidate_page: u64,
    pub company_page: u64,
    pub contact_page: u64,
    /// Recherche et page indépendantes des sélecteurs relationnels.
    pub company_option_search: String,
    pub candidate_option_search: String,
    pub contact_option_search: String,
    pub company_option_page: u64,
    pub candidate_option_page: u64,
    pub contact_option_page: u64,
    /// Onglet actif de l'écran Statistiques.
    pub statistics_tab: StatisticsTab,
    /// Page courante de l'historique des scores ATS (1-based).
    pub ats_page: u64,
    /// Page courante de l'historique des appels IA (1-based).
    pub llm_page: u64,
    /// Largeur de la page dans les plans de travail de document.
    pub document_width: f32,
    /// Dernière taille connue de la fenêtre, source des décisions de mise en page.
    pub window_size: iced::Size,
    /// Séquence des recherches/rechargements pour ignorer les réponses devenues obsolètes.
    pub data_request_sequence: u64,
}

#[cfg(test)]
#[path = "tests/state/mod.rs"]
mod tests;
