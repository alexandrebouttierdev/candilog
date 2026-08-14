//! Sélections, compteurs et cycle de vie des opérations IA.

use super::*;

impl App {
    /// Renvoie la page déjà filtrée et triée par SQLite.
    #[must_use]
    pub fn sorted_candidates(&self) -> Vec<&Candidature> {
        self.data.candidatures.iter().collect()
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

    /// Lettre sélectionnée dans la bibliothèque.
    #[must_use]
    pub fn focused_letter(&self) -> Option<&crate::modules::lettres::model::LettreMotivation> {
        self.selected_letter
            .and_then(|id| self.data.letters.iter().find(|item| item.id == id))
    }

    /// Ouvre une opération IA et renvoie son numéro de séquence.
    ///
    /// Le numéro accompagne le message de résultat : c'est lui qui permet d'ignorer celui
    /// d'une opération que l'utilisateur a abandonnée au profit d'une autre.
    pub fn commencer_operation_ia(&mut self, jeton: tokio_util::sync::CancellationToken) -> u64 {
        self.ai_sequence = self.ai_sequence.wrapping_add(1);
        self.ai_cancellation = Some(jeton);
        self.ai_is_running = true;
        self.ai_elapsed_seconds = 0;
        self.ai_sequence
    }

    /// Clôt l'opération `sequence` si elle est bien celle en cours.
    ///
    /// Renvoie `false` quand le résultat est périmé : l'appelant doit alors l'écarter sans
    /// toucher ni à l'indicateur d'activité, ni au jeton d'annulation, ni à l'écran.
    pub fn terminer_operation_ia(&mut self, sequence: u64) -> bool {
        if self.ai_sequence != sequence {
            tracing::debug!(sequence, courante = self.ai_sequence, "résultat IA périmé");
            return false;
        }
        self.ai_is_running = false;
        self.ai_cancellation = None;
        true
    }

    /// Nombre de pages de l'historique des scores ATS, au minimum 1.
    #[must_use]
    pub const fn ats_total_pages(&self) -> u64 {
        self.data.ats_scores.total_pages
    }

    /// Nombre de pages de l'historique des appels IA, au minimum 1.
    #[must_use]
    pub const fn llm_total_pages(&self) -> u64 {
        self.data.llm_calls.total_pages
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
