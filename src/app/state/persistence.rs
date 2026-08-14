//! Ouverture du socle, notifications et rechargement des instantanés.

use super::*;

impl App {
    /// Résout les chemins applicatifs et ouvre la base, puis charge un premier instantané.
    ///
    /// Extrait de `App::new()` pour être **rejouable** : l'écran d'erreur fatale proposait un
    /// bouton « Réessayer » câblé sur `Message::Reload`, dont la première instruction est
    /// `let Some(backend) = … else { return; }` — le backend n'étant justement jamais construit
    /// quand l'ouverture a échoué, le bouton ne faisait rien et n'effaçait même pas l'erreur.
    /// L'utilisateur restait enfermé dans un écran mort.
    pub fn bootstrap(&mut self) {
        match AppPaths::discover().and_then(|paths| {
            let backend = BackendState::persistent(&paths.database)?;
            Ok((paths, backend))
        }) {
            Ok((paths, backend)) => {
                tracing::info!(base = ?paths.database, "base ouverte");
                // Le fichier de base vient peut-être d'être créé : ses permissions n'ont pas
                // pu être posées lors de la résolution des chemins.
                paths.securiser();
                self.paths = Some(paths);
                self.backend = Some(Arc::new(backend));
                self.fatal_error = None;
                self.reload();
            }
            Err(error) => {
                tracing::error!(erreur = %error, "démarrage impossible");
                self.fatal_error = Some(error.message_utilisateur());
            }
        }
    }

    /// Affiche un message d'information ou de succès.
    pub fn notify(&mut self, kind: NotificationKind, message: impl Into<String>) {
        self.notification = Some(Notification {
            kind,
            message: message.into(),
        });
        // Chaque nouveau toast réarme le compte à rebours de 4 secondes.
        self.notification_shown_at = Some(std::time::Instant::now());
    }

    /// Confirme une opération réussie.
    pub fn notify_success(&mut self, message: impl Into<String>) {
        self.notify(NotificationKind::Success, message);
    }

    /// Signale un échec à partir d'une erreur typée : la nature du toast en découle, le détail
    /// technique part au journal et l'écran ne reçoit que la reformulation utilisateur.
    pub fn notify_error(&mut self, error: &AppError) {
        tracing::error!(erreur = %error, "opération en échec");
        self.notify(
            NotificationKind::from_error(error),
            error.message_utilisateur(),
        );
    }

    /// Signale un échec déjà réduit à une chaîne par une couche antérieure. Le message est
    /// affiché tel quel, mais reste classé comme un échec — jamais comme un succès.
    pub fn notify_failure(&mut self, message: impl Into<String>) {
        let message = message.into();
        tracing::error!(message = %message, "opération en échec");
        self.notify(NotificationKind::Error, message);
    }

    /// Recharge les données, **jeu par jeu**.
    ///
    /// Le chargement était une closure unique dont chaque appel propageait par `?` : le moindre
    /// échec sur un seul jeu abandonnait tous les autres et laissait `initialized` à faux,
    /// ce qui fige *tous* les écrans — Paramètres compris, donc sans accès à la restauration de
    /// backup — en squelette de chargement permanent. Une seule ligne de profil au JSON non
    /// conforme suffisait à rendre l'application entière inutilisable.
    ///
    /// Chaque jeu est désormais indépendant : ceux qui échouent retombent sur leur valeur par
    /// défaut, les autres s'affichent, et l'utilisateur est informé de ce qui manque.
    pub fn reload(&mut self) {
        let Some(backend) = self.backend.clone() else {
            return;
        };
        let request = self.snapshot_request();
        let (data, echecs) = charger_instantane(&backend, &request);
        self.appliquer_instantane(data, &echecs);
    }

    /// Capture les paramètres nécessaires au chargement hors du fil de rendu.
    #[must_use]
    pub fn snapshot_request(&self) -> SnapshotRequest {
        SnapshotRequest {
            sequence: self.data_request_sequence,
            route: self.route,
            search: self.search.clone(),
            company_type_filter: self.company_type_filter.clone(),
            candidate_filters: self.candidate_filters.clone(),
            candidate_sort: self.candidate_sort,
            candidate_sort_descending: self.candidate_sort_descending,
            candidate_page: self.candidate_page,
            company_page: self.company_page,
            contact_page: self.contact_page,
            company_option_search: self.company_option_search.clone(),
            candidate_option_search: self.candidate_option_search.clone(),
            contact_option_search: self.contact_option_search.clone(),
            company_option_page: self.company_option_page,
            candidate_option_page: self.candidate_option_page,
            contact_option_page: self.contact_option_page,
            selected_company_option: match self.dialog {
                Some(Dialog::Contact) => self.contact_form.entreprise_id,
                Some(Dialog::Candidature) => self.candidature_form.entreprise_id,
                _ => self.candidate_filters.company_id,
            },
            selected_candidate_option: match self.dialog {
                Some(Dialog::Entretien) => self.entretien_form.candidature_id,
                Some(Dialog::Relance) => self.relance_form.candidature_id,
                _ => None,
            },
            selected_contact_option: matches!(self.dialog, Some(Dialog::Entretien))
                .then_some(self.entretien_form.contact_id)
                .flatten(),
            calendar_year: self.calendar_year,
            calendar_month: self.calendar_month,
            llm_page: self.llm_page,
            ats_page: self.ats_page,
        }
    }

    #[must_use]
    pub fn candidate_total_pages(&self) -> u64 {
        self.data.candidatures_total_pages.max(1)
    }

    #[must_use]
    pub fn company_total_pages(&self) -> u64 {
        self.data.entreprises_total_pages.max(1)
    }

    #[must_use]
    pub fn contact_total_pages(&self) -> u64 {
        self.data.contacts_total_pages.max(1)
    }

    /// Installe un instantané fraîchement chargé et signale ce qui a échoué.
    ///
    /// Séparé du chargement lui-même pour être appelable depuis une `Task` : les écritures
    /// métier rechargent hors du fil de rendu, ce point d'entrée y applique le résultat.
    pub fn appliquer_instantane(&mut self, data: DataSnapshot, echecs: &[&'static str]) {
        // Le thème est résolu au démarrage. Les rechargements métier suivants ne doivent
        // jamais annuler une bascule effectuée depuis la sidebar pendant que sa persistance
        // asynchrone se termine.
        if !self.initialized {
            self.is_dark = crate::core::theme_systeme::resoudre(
                data.settings.theme,
                self.system_dark,
                self.is_dark,
            );
        }
        self.settings_form = SettingsForm::from_settings(&data.settings);
        self.data = data;
        // L'application reste utilisable même partiellement chargée : c'est le seul moyen
        // d'atteindre les Paramètres pour restaurer un backup ou réinitialiser la base.
        self.initialized = true;
        if !echecs.is_empty() {
            let message = format!(
                "Certaines données n'ont pas pu être lues ({}). Le reste de l'application \
                 reste utilisable ; une restauration de backup est proposée dans les Paramètres.",
                echecs.join(", ")
            );
            self.notify(NotificationKind::Warning, message);
        }
    }
}
