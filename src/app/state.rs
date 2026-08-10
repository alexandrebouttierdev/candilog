//! État global de l'application native.

use crate::core::config::AppPaths;
use crate::modules::candidatures::model::Candidature;
use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
use crate::modules::contacts::model::Contact;
use crate::modules::cv::model::CvVersionSummary;
use crate::modules::entreprises::model::Entreprise;
use crate::modules::entretiens::model::{Entretien, TypeEntretien};
use crate::modules::ia::cv_model::{CvGeneration, OfferAnalysis};
use crate::modules::metriques::model::{AppelLlm, ResumeScoresAts, ScoreAts};
use crate::modules::metriques::repository::MetriquesRepository;
use crate::modules::relances::model::Relance;
use crate::modules::settings::model::AppSettings;
use crate::navigation::Route;
use crate::shared::profile::Profile;
use crate::shared::state::AppState as BackendState;
use chrono::{Datelike, Local};
use std::sync::Arc;

use super::message::{CalendarView, CandidateView, Message};

/// Formulaire ou dialogue actuellement ouvert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    /// Création d'une entreprise.
    Entreprise,
    /// Création d'un contact.
    Contact,
    /// Création d'une candidature.
    Candidature,
    /// Création d'un entretien.
    Entretien,
    /// Création d'une relance.
    Relance,
    /// Édition du profil personnel et des compétences.
    Profil,
    /// Confirmation de suppression d'une candidature.
    DeleteCandidature(uuid::Uuid),
    /// Confirmation de suppression d'une entreprise.
    DeleteEntreprise(uuid::Uuid),
    /// Confirmation de suppression d'un contact.
    DeleteContact(uuid::Uuid),
    /// Confirmation de suppression d'un entretien.
    DeleteEntretien(uuid::Uuid),
    /// Confirmation de suppression d'une relance.
    DeleteRelance(uuid::Uuid),
    /// Confirmation de suppression d'une version de CV.
    DeleteCv(uuid::Uuid),
    /// Confirmation de restauration d'un backup.
    ImportBackup,
    /// Confirmation de réinitialisation complète.
    ResetDatabase,
    /// Confirmation de purge du cache IA local.
    ResetAiCache,
    /// Détail relationnel d'une candidature.
    CandidatureDetail(uuid::Uuid),
}

/// État du formulaire entreprise.
#[derive(Debug, Clone, Default)]
pub struct EntrepriseForm {
    pub nom: String,
    pub secteur: String,
    pub type_: String,
    pub site_web: String,
    pub ville: String,
    pub adresse: String,
    pub notes: String,
}

/// État du formulaire contact.
#[derive(Debug, Clone, Default)]
pub struct ContactForm {
    pub entreprise_id: Option<uuid::Uuid>,
    pub prenom: String,
    pub nom: String,
    pub poste: String,
    pub email: String,
    pub telephone: String,
    pub linkedin: String,
    pub notes: String,
}

/// État du formulaire candidature.
#[derive(Debug, Clone)]
pub struct CandidatureForm {
    pub entreprise_id: Option<uuid::Uuid>,
    pub poste: String,
    pub type_contrat: TypeContrat,
    pub statut: StatutCandidature,
    pub date_envoi: String,
    pub lien_offre: String,
    pub notes: String,
}

impl Default for CandidatureForm {
    fn default() -> Self {
        Self {
            entreprise_id: None,
            poste: String::new(),
            type_contrat: TypeContrat::Cdi,
            statut: StatutCandidature::EnAttente,
            date_envoi: chrono::Local::now().format("%Y-%m-%d").to_string(),
            lien_offre: String::new(),
            notes: String::new(),
        }
    }
}

/// État du formulaire entretien.
#[derive(Debug, Clone)]
pub struct EntretienForm {
    pub candidature_id: Option<uuid::Uuid>,
    pub contact_id: Option<uuid::Uuid>,
    pub date_entretien: String,
    pub type_entretien: TypeEntretien,
    pub lieu: String,
    pub notes: String,
    pub compte_rendu: String,
}

impl Default for EntretienForm {
    fn default() -> Self {
        Self {
            candidature_id: None,
            contact_id: None,
            date_entretien: chrono::Local::now().format("%Y-%m-%dT%H:%M").to_string(),
            type_entretien: TypeEntretien::Presentiel,
            lieu: String::new(),
            notes: String::new(),
            compte_rendu: String::new(),
        }
    }
}

/// État du formulaire relance.
#[derive(Debug, Clone)]
pub struct RelanceForm {
    pub candidature_id: Option<uuid::Uuid>,
    pub date_relance: String,
    pub type_relance: String,
    pub notes: String,
}

/// Filtres cumulables des candidatures.
#[derive(Debug, Clone, Default)]
pub struct CandidateFilters {
    pub status: Option<StatutCandidature>,
    pub contract: Option<TypeContrat>,
    pub company_id: Option<uuid::Uuid>,
    pub city: String,
    pub position: String,
    pub date_from: String,
    pub date_to: String,
}

impl CandidateFilters {
    /// Nombre de critères réellement actifs, hors recherche globale.
    #[must_use]
    pub fn active_count(&self) -> usize {
        usize::from(self.status.is_some())
            + usize::from(self.contract.is_some())
            + usize::from(self.company_id.is_some())
            + usize::from(!self.city.trim().is_empty())
            + usize::from(!self.position.trim().is_empty())
            + usize::from(!self.date_from.trim().is_empty())
            + usize::from(!self.date_to.trim().is_empty())
    }
}

/// Colonne de tri de la vue Liste des candidatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CandidateSort {
    /// Intitulé du poste.
    Poste,
    /// Nom de l'entreprise.
    Entreprise,
    /// Statut de la candidature.
    Statut,
    /// Date d'envoi, ordre par défaut.
    #[default]
    Date,
}

impl CandidateSort {
    /// Colonnes triables, dans l'ordre d'affichage de la table.
    pub const ALL: [Self; 4] = [Self::Poste, Self::Entreprise, Self::Statut, Self::Date];

    /// Colonne correspondant à un index d'en-tête.
    #[must_use]
    pub fn from_column(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    /// Index d'en-tête de la colonne.
    #[must_use]
    pub fn column(self) -> usize {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .unwrap_or_default()
    }
}

/// Sections de l'écran Paramètres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    /// Apparence de l'application.
    #[default]
    Apparence,
    /// Fournisseur et réglages IA.
    Ia,
    /// Base locale, exports et sauvegardes.
    Donnees,
    /// Canal de mise à jour.
    MisesAJour,
    /// Informations sur l'application.
    APropos,
}

impl SettingsSection {
    /// Sections dans l'ordre du sommaire.
    pub const ALL: [Self; 5] = [
        Self::Apparence,
        Self::Ia,
        Self::Donnees,
        Self::MisesAJour,
        Self::APropos,
    ];

    /// Intitulé affiché dans le sommaire et l'en-tête.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Apparence => "Apparence",
            Self::Ia => "Intelligence artificielle",
            Self::Donnees => "Données locales",
            Self::MisesAJour => "Mises à jour",
            Self::APropos => "À propos",
        }
    }
}

/// Onglet actif de l'écran Statistiques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatisticsTab {
    /// Suivi du pipeline et des relances.
    #[default]
    Candidatures,
    /// Scores ATS et appels IA.
    PerformanceCv,
}

/// Décision utilisateur sur une recommandation ATS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationStatus {
    /// Aucune décision.
    Pending,
    /// Proposition appliquée.
    Accepted,
    /// Proposition refusée.
    Rejected,
}

impl Default for RelanceForm {
    fn default() -> Self {
        Self {
            candidature_id: None,
            date_relance: chrono::Local::now().format("%Y-%m-%d").to_string(),
            type_relance: "Email".into(),
            notes: String::new(),
        }
    }
}

/// Données chargées pour les différents écrans.
#[derive(Debug, Clone, Default)]
pub struct DataSnapshot {
    /// Candidatures.
    pub candidatures: Vec<Candidature>,
    /// Entreprises.
    pub entreprises: Vec<Entreprise>,
    /// Contacts.
    pub contacts: Vec<Contact>,
    /// Entretiens.
    pub entretiens: Vec<Entretien>,
    /// Relances.
    pub relances: Vec<Relance>,
    /// Versions de CV.
    pub cv_versions: Vec<CvVersionSummary>,
    /// Profil structuré.
    pub profile: Profile,
    /// Paramètres applicatifs sans secret en clair.
    pub settings: AppSettings,
    /// Historique des appels IA.
    pub llm_calls: Vec<AppelLlm>,
    /// Historique des scores ATS.
    pub ats_scores: Vec<ScoreAts>,
    /// Agrégats ATS globaux.
    pub ats_summary: Option<ResumeScoresAts>,
}

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
    /// Notification utilisateur non bloquante.
    pub notification: Option<String>,
    /// Mise à jour disponible.
    pub available_update: Option<crate::core::updater::UpdateInfo>,
    /// Progression du téléchargement de mise à jour.
    pub update_progress: Option<u8>,
    /// Paquet de mise à jour téléchargé et vérifié.
    pub verified_update_path: Option<std::path::PathBuf>,
    /// Backup sélectionné avant confirmation de restauration.
    pub pending_backup_import: Option<std::path::PathBuf>,
    /// Dialogue métier ouvert.
    pub dialog: Option<Dialog>,
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
    /// Candidature actuellement glissée dans le Kanban.
    pub dragging_candidate: Option<uuid::Uuid>,
    /// Colonne survolée pendant le glisser-déposer.
    pub drag_target_status: Option<StatutCandidature>,
    /// Informations personnelles éditées.
    pub profile_personal_form: crate::shared::profile::PersonalInfo,
    /// Compétences éditées, séparées par des virgules.
    pub profile_skills_form: String,
    /// PDF choisi pour importer le profil.
    pub profile_import_path: Option<std::path::PathBuf>,
    /// Profil extrait en attente de validation explicite.
    pub extracted_profile: Option<crate::shared::profile::Profile>,
    /// Candidature sélectionnée dans la liste ou le pipeline.
    pub selected_candidate: Option<uuid::Uuid>,
    /// Entreprise sélectionnée dans le répertoire.
    pub selected_company: Option<uuid::Uuid>,
    /// Contact sélectionné dans le réseau.
    pub selected_contact: Option<uuid::Uuid>,
    /// Version de CV sélectionnée dans la bibliothèque.
    pub selected_cv: Option<uuid::Uuid>,
    /// Feuille de filtres dépliée.
    pub filters_open: bool,
    /// Colonne de tri de la vue Liste.
    pub candidate_sort: CandidateSort,
    /// Sens de tri de la vue Liste.
    pub candidate_sort_descending: bool,
    /// Section active de l'écran Paramètres.
    pub settings_section: SettingsSection,
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
}

impl App {
    /// Initialise les chemins, la base et toutes les données visibles.
    pub fn new() -> (Self, iced::Task<Message>) {
        let now = Local::now();
        let mut app = Self {
            route: Route::Dashboard,
            candidate_view: CandidateView::Kanban,
            candidate_filters: CandidateFilters::default(),
            search: String::new(),
            calendar_year: now.year(),
            calendar_month: now.month(),
            calendar_date: now.date_naive(),
            calendar_view: CalendarView::Month,
            is_dark: true,
            paths: None,
            backend: None,
            data: DataSnapshot::default(),
            initialized: false,
            fatal_error: None,
            notification: None,
            available_update: None,
            update_progress: None,
            verified_update_path: None,
            pending_backup_import: None,
            dialog: None,
            editing_id: None,
            entreprise_form: EntrepriseForm::default(),
            contact_form: ContactForm::default(),
            candidature_form: CandidatureForm::default(),
            entretien_form: EntretienForm::default(),
            relance_form: RelanceForm::default(),
            offer_editor: iced::widget::text_editor::Content::new(),
            offer_analysis: None,
            cv_generation: None,
            cv_version_name: "CV optimisé".into(),
            recommendation_states: Vec::new(),
            ai_is_running: false,
            ai_elapsed_seconds: 0,
            ai_cancellation: None,
            import_offer_editor: iced::widget::text_editor::Content::new(),
            import_pdf_path: None,
            imported_cv_analysis: None,
            letter_company: String::new(),
            letter_job_title: String::new(),
            letter_editor: iced::widget::text_editor::Content::new(),
            letter_tone: "formal".into(),
            letter_length: "medium".into(),
            letter_output: String::new(),
            dragging_candidate: None,
            drag_target_status: None,
            profile_personal_form: crate::shared::profile::PersonalInfo::default(),
            profile_skills_form: String::new(),
            profile_import_path: None,
            extracted_profile: None,
            selected_candidate: None,
            selected_company: None,
            selected_contact: None,
            selected_cv: None,
            filters_open: false,
            candidate_sort: CandidateSort::default(),
            candidate_sort_descending: true,
            settings_section: SettingsSection::default(),
            statistics_tab: StatisticsTab::default(),
            ats_page: 1,
            llm_page: 1,
            document_width: crate::ui::components::document::DEFAULT_WIDTH,
            window_size: iced::Size::new(
                crate::ui::theme::layout::MIN_WIDTH,
                crate::ui::theme::layout::MIN_HEIGHT,
            ),
        };
        if let Ok(route) = std::env::var("CANDILOG_CAPTURE_ROUTE") {
            app.route = match route.as_str() {
                "candidatures" => Route::Candidatures,
                "cv" => Route::Cv,
                "entreprises" => Route::Entreprises,
                "reseau" => Route::Reseau,
                "calendrier" => Route::Calendrier,
                "statistiques" => Route::Statistiques,
                "cv-generator" => Route::CvGenerator,
                "lettre" => Route::LettreMotivation,
                "cv-import" => Route::CvImport,
                "profil" => Route::Profil,
                "parametres" => Route::Parametres,
                _ => Route::Dashboard,
            };
        }
        if std::env::var("CANDILOG_CAPTURE_THEME").as_deref() == Ok("light") {
            app.is_dark = false;
        }
        if std::env::var("CANDILOG_CAPTURE_CANDIDATE_VIEW").as_deref() == Ok("list") {
            app.candidate_view = CandidateView::List;
        }
        if let Ok(dialog) = std::env::var("CANDILOG_CAPTURE_DIALOG") {
            app.dialog = match dialog.as_str() {
                "candidature" => Some(Dialog::Candidature),
                "entreprise" => Some(Dialog::Entreprise),
                "contact" => Some(Dialog::Contact),
                "entretien" => Some(Dialog::Entretien),
                "relance" => Some(Dialog::Relance),
                "profil" => Some(Dialog::Profil),
                _ => None,
            };
        }
        match AppPaths::discover() {
            Ok(paths) => match BackendState::persistent(&paths.database) {
                Ok(backend) => {
                    app.paths = Some(paths);
                    app.backend = Some(Arc::new(backend));
                    app.reload();
                }
                Err(error) => app.fatal_error = Some(error.to_string()),
            },
            Err(error) => app.fatal_error = Some(error.to_string()),
        }
        if std::env::var("CANDILOG_CAPTURE_DIALOG").as_deref() == Ok("detail") {
            app.dialog = app
                .data
                .candidatures
                .first()
                .map(|candidate| Dialog::CandidatureDetail(candidate.id));
        }
        match std::env::var("CANDILOG_CAPTURE_CALENDAR_VIEW").as_deref() {
            Ok("week") => app.calendar_view = CalendarView::Week,
            Ok("day") => app.calendar_view = CalendarView::Day,
            _ => {}
        }
        if std::env::var_os("CANDILOG_CAPTURE_AI_RUNNING").is_some() {
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 18;
        }
        if let Ok(message) = std::env::var("CANDILOG_CAPTURE_NOTIFICATION") {
            app.notification = Some(message);
        }
        if std::env::var_os("CANDILOG_CAPTURE_FATAL_ERROR").is_some() {
            app.fatal_error = Some(
                "Impossible d’ouvrir la base locale. Vérifiez les droits du dossier de données."
                    .into(),
            );
        }
        let initial = if std::env::var_os("CANDILOG_CAPTURE_PATH").is_some() {
            iced::Task::perform(
                async {
                    tokio::time::sleep(std::time::Duration::from_millis(1_200)).await;
                },
                |()| super::Message::CaptureForReview,
            )
        } else {
            iced::Task::none()
        };
        // La capture visuelle ne maximise pas la fenêtre (dimensions maîtrisées).
        let maximize = if std::env::var_os("CANDILOG_CAPTURE_PATH").is_some() {
            iced::Task::none()
        } else {
            iced::Task::done(super::Message::MaximizeWindow)
        };
        (app, iced::Task::batch([initial, maximize]))
    }

    /// Décisions de mise en page pour la taille de fenêtre courante.
    #[must_use]
    pub fn layout(&self) -> crate::ui::theme::Layout {
        crate::ui::theme::Layout::from_size(self.window_size)
    }

    /// Recharge les données sans remplacer un instantané valide en cas d'erreur.
    pub fn reload(&mut self) {
        let Some(backend) = &self.backend else {
            return;
        };
        let loaded = (|| {
            Ok::<DataSnapshot, crate::shared::error::AppError>(DataSnapshot {
                candidatures: backend.candidatures.lister()?,
                entreprises: backend.entreprises.lister()?,
                contacts: backend.contacts.lister()?,
                entretiens: backend.entretiens.lister()?,
                relances: backend.relances.lister()?,
                cv_versions: backend.cv.list()?,
                profile: backend.profil.get()?,
                settings: backend.settings.get()?,
                llm_calls: backend.metriques.lister_appels()?,
                ats_scores: backend.metriques.lister_scores()?,
                ats_summary: Some(backend.metriques.resumer_scores()?),
            })
        })();
        match loaded {
            Ok(snapshot) => {
                self.data = snapshot;
                self.initialized = true;
            }
            Err(error) => self.notification = Some(error.to_string()),
        }
    }

    /// Applique la recherche et les six catégories de filtres aux candidatures.
    #[must_use]
    pub fn filtered_candidates(&self) -> Vec<&Candidature> {
        let search = self.search.trim().to_lowercase();
        let position = self.candidate_filters.position.trim().to_lowercase();
        let city = self.candidate_filters.city.trim().to_lowercase();
        self.data
            .candidatures
            .iter()
            .filter(|candidate| {
                let company = self
                    .data
                    .entreprises
                    .iter()
                    .find(|company| company.id == candidate.entreprise_id);
                let matches_search = search.is_empty()
                    || candidate.poste.to_lowercase().contains(&search)
                    || candidate
                        .entreprise_nom
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&search);
                matches_search
                    && self
                        .candidate_filters
                        .status
                        .is_none_or(|status| candidate.statut == status)
                    && self
                        .candidate_filters
                        .contract
                        .is_none_or(|contract| candidate.type_contrat == contract)
                    && self
                        .candidate_filters
                        .company_id
                        .is_none_or(|id| candidate.entreprise_id == id)
                    && (position.is_empty() || candidate.poste.to_lowercase().contains(&position))
                    && (city.is_empty()
                        || company
                            .and_then(|item| item.ville.as_deref())
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&city))
                    && (self.candidate_filters.date_from.is_empty()
                        || candidate.date_envoi >= self.candidate_filters.date_from)
                    && (self.candidate_filters.date_to.is_empty()
                        || candidate.date_envoi <= self.candidate_filters.date_to)
            })
            .collect()
    }

    /// Applique le tri de la vue Liste aux candidatures déjà filtrées.
    #[must_use]
    pub fn sorted_candidates(&self) -> Vec<&Candidature> {
        let mut candidates = self.filtered_candidates();
        candidates.sort_by(|left, right| {
            let ordering = match self.candidate_sort {
                CandidateSort::Poste => left.poste.to_lowercase().cmp(&right.poste.to_lowercase()),
                CandidateSort::Entreprise => left
                    .entreprise_nom
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .cmp(
                        &right
                            .entreprise_nom
                            .as_deref()
                            .unwrap_or_default()
                            .to_lowercase(),
                    ),
                CandidateSort::Statut => left.statut.to_string().cmp(&right.statut.to_string()),
                CandidateSort::Date => left.date_envoi.cmp(&right.date_envoi),
            };
            if self.candidate_sort_descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
        candidates
    }

    /// Candidature couramment mise en avant par la sélection ou le tri.
    #[must_use]
    pub fn focused_candidate(&self) -> Option<&Candidature> {
        self.selected_candidate
            .and_then(|id| self.data.candidatures.iter().find(|item| item.id == id))
    }

    /// Entreprise sélectionnée, ou la première du répertoire.
    #[must_use]
    pub fn focused_company(&self) -> Option<&Entreprise> {
        self.selected_company
            .and_then(|id| self.data.entreprises.iter().find(|item| item.id == id))
    }

    /// Contact sélectionné dans le réseau.
    #[must_use]
    pub fn focused_contact(&self) -> Option<&Contact> {
        self.selected_contact
            .and_then(|id| self.data.contacts.iter().find(|item| item.id == id))
    }

    /// Version de CV sélectionnée dans la bibliothèque.
    #[must_use]
    pub fn focused_cv(&self) -> Option<&CvVersionSummary> {
        self.selected_cv
            .and_then(|id| self.data.cv_versions.iter().find(|item| item.id == id))
    }

    /// Nombre de relances arrivées à échéance à la date donnée.
    #[must_use]
    pub fn due_reminders(&self, today: &str) -> usize {
        self.data
            .relances
            .iter()
            .filter(|item| item.date_relance.as_str() <= today)
            .count()
    }

    /// Nombre d'entretiens planifiés à partir de la date donnée.
    #[must_use]
    pub fn upcoming_interviews(&self, today: &str) -> usize {
        self.data
            .entretiens
            .iter()
            .filter(|item| item.date_entretien.as_str() >= today)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use std::sync::Mutex;

    /// `App::new()` lit des variables d'environnement globales : les tests qui
    /// l'appellent sont sérialisés pour ne pas interférer entre eux.
    static APP_NEW_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn le_reload_reussi_marque_l_application_initialisee() {
        // App::new ouvre la base persistante de développement
        // (.candilog-dev/candilog.sqlite sous le répertoire du projet), pas une
        // base mémoire. Ce test exige donc une base de développement valide ;
        // il est sérialisé avec le test d'échec par le verrou.
        let _guard = APP_NEW_LOCK.lock().unwrap();
        let (app, _) = App::new();
        assert!(
            app.initialized,
            "le chargement initial a réussi sur la base de développement"
        );
    }

    #[test]
    fn l_echec_d_ouverture_de_la_base_laisse_l_application_non_initialisee() {
        // Branche d'échec du contrat : quand la base ne peut pas s'ouvrir,
        // fatal_error est renseigné et initialized reste faux (reload n'est
        // jamais appelé). On force l'échec sans toucher à la base de
        // développement réelle : CANDILOG_DATA_DIR pointe sur un fichier
        // ordinaire, ce qui rend le dossier de données inconstructible.
        let _guard = APP_NEW_LOCK.lock().unwrap();
        let marker = std::env::temp_dir().join(format!("candilog-failure-{}", std::process::id()));
        std::fs::write(&marker, "blocage volontaire").unwrap();
        std::env::set_var("CANDILOG_DATA_DIR", &marker);
        let (app, _) = App::new();
        std::env::remove_var("CANDILOG_DATA_DIR");
        std::fs::remove_file(&marker).ok();
        assert!(
            app.fatal_error.is_some(),
            "une erreur fatale est signalée quand la base ne peut pas s'ouvrir"
        );
        assert!(!app.initialized, "l'application reste non initialisée");
    }
}
