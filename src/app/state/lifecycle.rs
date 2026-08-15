//! Construction, ouverture et harnais de capture de l'application.

use super::*;

impl App {
    /// Initialise les chemins, la base et toutes les données visibles.
    pub fn new() -> (Self, iced::Task<Message>) {
        let mut app = Self::vierge();
        app.appliquer_harnais_de_capture_avant_ouverture();
        app.bootstrap();
        app.appliquer_harnais_de_capture_apres_ouverture();
        let initial = if capture_demandee() {
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
        let maximize = if capture_demandee() {
            iced::Task::none()
        } else {
            iced::Task::done(super::Message::MaximizeWindow)
        };
        // Santé du fournisseur mesurée dès le démarrage : la pastille reste grise tant que
        // rien n'a été vérifié, plutôt que d'afficher un vert non fondé.
        let sonde = if app.backend.is_some() {
            iced::Task::done(super::Message::ProbeProviderHealth)
        } else {
            iced::Task::none()
        };
        // Le thème du système est demandé au démarrage : « Système » doit suivre le système,
        // pas se comporter comme « Sombre ».
        let theme = if capture_demandee() {
            // Le harnais impose un thème déterministe. Une détection système lancée en
            // parallèle pouvait le réécraser avant la capture et produire deux images
            // sombres malgré `CANDILOG_CAPTURE_THEME=light`.
            iced::Task::none()
        } else {
            iced::Task::perform(
                crate::core::theme_systeme::detecter(),
                super::Message::SystemThemeDetected,
            )
        };
        (app, iced::Task::batch([initial, maximize, sonde, theme]))
    }

    /// Construit l'application **autour d'un backend déjà ouvert**, puis charge les données.
    ///
    /// Seul chemin permettant de tester `App` sans effet de bord : `App::new()` résout lui-même
    /// ses chemins et ouvre lui-même sa base, ce qui obligeait ses tests à dépendre du fichier
    /// réel `.candilog-dev/candilog.sqlite` du dépôt et à muter `CANDILOG_DATA_DIR` — une
    /// variable d'environnement du **processus entier**, lue en parallèle par les autres tests.
    #[must_use]
    pub fn with_backend(paths: AppPaths, backend: BackendState) -> Self {
        let mut app = Self::vierge();
        app.paths = Some(paths);
        app.backend = Some(Arc::new(backend));
        app.reload();
        app
    }

    /// État initial, avant toute résolution de chemin et toute ouverture de base.
    fn vierge() -> Self {
        let now = Local::now();
        Self {
            route: Route::Dashboard,
            candidate_view: CandidateView::Kanban,
            candidate_filters: CandidateFilters::default(),
            search: String::new(),
            company_type_filter: None,
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
            settings_form: SettingsForm::default(),
            system_dark: None,
            ai_sequence: 0,
            notification: None,
            notification_shown_at: None,
            provider_health: crate::ui::components::runtime_status::Health::default(),
            available_models: Vec::new(),
            available_update: None,
            update_progress: None,
            pending_backup_import: None,
            dialog: None,
            write_in_progress: false,
            date_picker: None,
            editing_id: None,
            entreprise_form: EntrepriseForm::default(),
            contact_form: ContactForm::default(),
            candidature_form: CandidatureForm::default(),
            entretien_form: EntretienForm::default(),
            relance_form: RelanceForm::default(),
            offer_editor: iced::widget::text_editor::Content::new(),
            offer_analysis: None,
            cv_generation: None,
            cv_preview_generation: None,
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
            letter_iteration_instruction: String::new(),
            letter_chat_history: Vec::new(),
            press_candidate: None,
            press_origin: None,
            dragging_candidate: None,
            drag_target_status: None,
            hovered_card: None,
            profile_personal_form: crate::shared::profile::PersonalInfo::default(),
            profile_draft: crate::shared::profile::Profile::default(),
            profile_summary_editor: iced::widget::text_editor::Content::new(),
            profile_skills_form: String::new(),
            profile_import_path: None,
            extracted_profile: None,
            profile_import_excluded: std::collections::HashSet::new(),
            selected_candidate: None,
            selected_company: None,
            selected_contact: None,
            selected_cv: None,
            selected_letter: None,
            filters_open: false,
            candidate_sort: CandidateSort::default(),
            candidate_sort_descending: true,
            candidate_page: 1,
            company_page: 1,
            contact_page: 1,
            company_option_search: String::new(),
            candidate_option_search: String::new(),
            contact_option_search: String::new(),
            company_option_page: 1,
            candidate_option_page: 1,
            contact_option_page: 1,
            statistics_tab: StatisticsTab::default(),
            ats_page: 1,
            llm_page: 1,
            document_width: crate::ui::components::document::DEFAULT_WIDTH,
            window_size: iced::Size::new(
                crate::ui::theme::layout::MIN_WIDTH,
                crate::ui::theme::layout::MIN_HEIGHT,
            ),
            data_request_sequence: 0,
        }
    }
    /// Réglages du harnais de capture applicables **avant** l'ouverture de la base.
    ///
    /// Le harnais sert la revue de design (routes, thèmes, dialogues, états particuliers). Il
    /// n'est pas destiné aux utilisateurs : ses variables d'environnement modifient le
    /// comportement de l'application, écrivent un fichier au chemin indiqué et fabriquent de
    /// faux écrans d'erreur. La caractéristique Cargo `capture` le retire du binaire distribué.
    ///
    /// `CANDILOG_DATA_DIR` n'en fait pas partie et reste actif en toutes circonstances : c'est
    /// lui qui permet de travailler sans toucher aux données réelles.
    #[cfg(feature = "capture")]
    fn appliquer_harnais_de_capture_avant_ouverture(&mut self) {
        if let Ok(route) = std::env::var("CANDILOG_CAPTURE_ROUTE") {
            self.route = match route.as_str() {
                "candidatures" => Route::Candidatures,
                "cv" => Route::Cv,
                "entreprises" => Route::Entreprises,
                "reseau" => Route::Reseau,
                "calendrier" => Route::Calendrier,
                "statistiques" => Route::Statistiques,
                "cv-generator" => Route::CvGenerator,
                "lettres" => Route::Lettres,
                "lettre" => Route::LettreMotivation,
                "cv-import" => Route::CvImport,
                "profil" => Route::Profil,
                "parametres" => Route::Parametres,
                "sauvegardes" => Route::Sauvegardes,
                "mises-a-jour" => Route::MisesAJour,
                "a-propos" => Route::APropos,
                _ => Route::Dashboard,
            };
        }
        match std::env::var("CANDILOG_CAPTURE_THEME").as_deref() {
            Ok("light") => self.is_dark = false,
            Ok("dark") => self.is_dark = true,
            _ => {}
        }
        if std::env::var("CANDILOG_CAPTURE_CANDIDATE_VIEW").as_deref() == Ok("list") {
            self.candidate_view = CandidateView::List;
        }
        if let Ok(dialog) = std::env::var("CANDILOG_CAPTURE_DIALOG") {
            self.dialog = match dialog.as_str() {
                "candidature" => Some(Dialog::Candidature),
                "entreprise" => Some(Dialog::Entreprise),
                "contact" => Some(Dialog::Contact),
                "entretien" => Some(Dialog::Entretien),
                "relance" => Some(Dialog::Relance),
                "profil" => Some(Dialog::Profil(ProfileSection::Identite)),
                "profil-competences" => Some(Dialog::Profil(ProfileSection::Competences)),
                "profil-experiences" => Some(Dialog::Profil(ProfileSection::Collection(
                    ProfileCollection::Experience,
                ))),
                "profil-import" => Some(Dialog::ProfileImport),
                _ => None,
            };
        }
    }

    /// Réglages du harnais de capture qui supposent les données chargées.
    #[cfg(feature = "capture")]
    fn appliquer_harnais_de_capture_apres_ouverture(&mut self) {
        // `bootstrap` recharge le thème persisté ; l'override de capture doit donc être
        // réappliqué après l'ouverture de la base pour rester prioritaire.
        match std::env::var("CANDILOG_CAPTURE_THEME").as_deref() {
            Ok("light") => self.is_dark = false,
            Ok("dark") => self.is_dark = true,
            _ => {}
        }
        if std::env::var("CANDILOG_CAPTURE_DIALOG").as_deref() == Ok("detail") {
            self.dialog = self
                .data
                .candidatures
                .first()
                .map(|candidate| Dialog::CandidatureDetail(candidate.id));
        }
        if std::env::var("CANDILOG_CAPTURE_DIALOG").as_deref() == Ok("contact-detail") {
            self.selected_contact = self.data.contacts.first().map(|contact| contact.id);
        }
        if std::env::var("CANDILOG_CAPTURE_DIALOG").as_deref() == Ok("company-detail") {
            self.selected_company = self.data.entreprises.first().map(|company| company.id);
        }
        if matches!(self.dialog, Some(Dialog::Profil(_))) {
            self.profile_personal_form = self.data.profile.personal.clone();
            self.profile_draft = self.data.profile.clone();
            self.profile_summary_editor = iced::widget::text_editor::Content::with_text(
                self.data
                    .profile
                    .personal
                    .summary
                    .as_deref()
                    .unwrap_or_default(),
            );
        }
        if std::env::var_os("CANDILOG_CAPTURE_PROFILE_IMPORT").is_some() {
            self.extracted_profile = Some(profile_import_capture_fixture());
            self.profile_import_path = Some(std::path::PathBuf::from("CV_Camille_Moreau_Demo.pdf"));
            self.dialog = Some(Dialog::ProfileImport);
        }
        if std::env::var("CANDILOG_CAPTURE_DATE_PICKER").as_deref() == Ok("candidature") {
            let today = chrono::Local::now().date_naive();
            self.date_picker = Some(DatePickerState {
                target: DatePickerTarget::Candidature,
                year: today.year(),
                month: today.month(),
            });
        }
        match std::env::var("CANDILOG_CAPTURE_CALENDAR_VIEW").as_deref() {
            Ok("week") => self.calendar_view = CalendarView::Week,
            Ok("day") => self.calendar_view = CalendarView::Day,
            _ => {}
        }
        if std::env::var("CANDILOG_CAPTURE_STATISTICS_TAB").as_deref() == Ok("performance") {
            self.statistics_tab = StatisticsTab::PerformanceCv;
        }
        if std::env::var_os("CANDILOG_CAPTURE_AI_RUNNING").is_some() {
            self.ai_is_running = true;
            self.ai_elapsed_seconds = 18;
        }
        if std::env::var_os("CANDILOG_CAPTURE_CV_PREVIEW").is_some() {
            if let (Some(backend), Some(summary)) =
                (self.backend.as_ref(), self.data.cv_versions.first())
            {
                if let Ok(version) = backend.cv.load(summary.id) {
                    let cv = serde_json::from_value(version.content["cv"].clone());
                    let analysis = serde_json::from_value(version.content["analysis"].clone());
                    if let (Ok(cv), Ok(analysis)) = (cv, analysis) {
                        let generation =
                            crate::modules::ia::cv_model::CvGeneration { cv, analysis };
                        self.cv_generation = Some(generation.clone());
                        self.cv_preview_generation = Some(generation);
                        self.selected_cv = Some(summary.id);
                    }
                }
            }
        }
        if std::env::var_os("CANDILOG_CAPTURE_LETTER_OUTPUT").is_some() {
            self.letter_company = "Boussole Labs".into();
            self.letter_job_title = "Product owner".into();
            self.letter_output = "Madame, Monsieur,\n\nVotre offre de Product owner a retenu toute mon attention. Mon expérience en coordination de projets numériques, en animation d'équipes et en amélioration continue me permettrait de contribuer rapidement à vos produits.\n\nJe serais heureuse d'échanger avec vous afin de détailler ma motivation et la manière dont je pourrais rejoindre votre équipe.\n\nCordialement,\nCamille Moreau".into();
            self.letter_chat_history
                .push(crate::modules::ia::cv_model::ChatMsg {
                    role: "user".into(),
                    content: "Rends l'introduction plus directe et la conclusion plus chaleureuse."
                        .into(),
                });
        }
        if let Ok(message) = std::env::var("CANDILOG_CAPTURE_NOTIFICATION") {
            self.notify(NotificationKind::Info, message);
        }
        if std::env::var_os("CANDILOG_CAPTURE_FATAL_ERROR").is_some() {
            self.fatal_error = Some(
                "Impossible d\u{2019}ouvrir la base locale. Vérifiez les droits du dossier de données."
                    .into(),
            );
        }
    }

    /// Sans la caractéristique `capture`, le harnais n'existe pas dans le binaire.
    #[cfg(not(feature = "capture"))]
    const fn appliquer_harnais_de_capture_avant_ouverture(&mut self) {}

    /// Sans la caractéristique `capture`, le harnais n'existe pas dans le binaire.
    #[cfg(not(feature = "capture"))]
    const fn appliquer_harnais_de_capture_apres_ouverture(&mut self) {}

    /// Décisions de mise en page pour la taille de fenêtre courante.
    #[must_use]
    pub fn layout(&self) -> crate::ui::theme::Layout {
        crate::ui::theme::Layout::from_size(self.window_size)
    }
}

#[cfg(feature = "capture")]
fn profile_import_capture_fixture() -> crate::shared::profile::Profile {
    use crate::shared::profile::{Education, Experience, Language, PersonalInfo, Skill};

    crate::shared::profile::Profile {
        personal: PersonalInfo {
            first_name: "Camille".into(),
            last_name: "Moreau".into(),
            email: "camille.moreau@example.test".into(),
            phone: Some("06 00 00 00 00".into()),
            city: Some("Nantes".into()),
            headline: Some("Cheffe de projet digital".into()),
            summary: Some("Cheffe de projet spécialisée dans la coordination de produits numériques et la collaboration entre équipes métier et design.".into()),
            linkedin: Some("linkedin.com/in/camille-moreau-demo".into()),
            github: None,
            website: Some("camille-portfolio.example".into()),
        },
        experiences: vec![
            Experience {
                title: "Cheffe de projet digital".into(),
                company: "Studio Nébula".into(),
                start_date: "2023-09".into(),
                current: true,
                ..Experience::default()
            },
            Experience {
                title: "Chargée de communication".into(),
                company: "Maison Sépia".into(),
                start_date: "2021-03".into(),
                ..Experience::default()
            },
        ],
        skills: vec![
            Skill { name: "Gestion de projet".into() },
            Skill { name: "Méthodes agiles".into() },
            Skill { name: "Recherche utilisateur".into() },
            Skill { name: "Analyse de données".into() },
        ],
        education: vec![Education {
            degree: "Master stratégie digitale".into(),
            school: "Institut Mercure".into(),
            ..Education::default()
        }],
        languages: vec![
            Language { name: "Français".into(), level: "Natif".into() },
            Language { name: "Anglais".into(), level: "B2".into() },
        ],
        ..crate::shared::profile::Profile::default()
    }
}
