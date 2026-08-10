//! Messages produits par les vues Iced.

use super::state::{Dialog, ProfileSection, SettingsSection, StatisticsTab};
use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
use crate::modules::entretiens::model::TypeEntretien;
use crate::navigation::Route;

/// Mode de présentation des candidatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CandidateView {
    /// Pipeline par statut.
    #[default]
    Kanban,
    /// Tableau compact.
    List,
}

/// Granularité du calendrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalendarView {
    /// Grille mensuelle.
    #[default]
    Month,
    /// Semaine détaillée.
    Week,
    /// Journée détaillée.
    Day,
}

/// Événement progressif produit par une génération de lettre.
#[derive(Debug, Clone)]
pub enum LetterStreamEvent {
    /// Fragment immédiatement affichable.
    Chunk(String),
    /// Résultat terminal de la génération.
    Finished(Result<String, String>),
}

/// Progression du téléchargement signé d'une mise à jour.
#[derive(Debug, Clone)]
pub enum UpdateDownloadEvent {
    /// Pourcentage téléchargé.
    Progress(u8),
    /// Paquet vérifié ou erreur terminale.
    Finished(Result<std::path::PathBuf, String>),
}

/// Tous les événements applicatifs.
#[derive(Debug, Clone)]
pub enum Message {
    /// Demande une capture native pour la revue visuelle opt-in.
    CaptureForReview,
    /// Fenêtre résolue pour la capture native.
    CaptureWindow(Option<iced::window::Id>),
    /// Pixels rendus par Iced pour la revue visuelle.
    CapturedForReview(iced::window::Screenshot),
    /// Fenêtre initiale localisée, pour la maximiser au lancement.
    MaximizeWindow,
    /// Maximise la fenêtre identifiée.
    MaximizeWindowId(iced::window::Id),
    /// La fenêtre a été redimensionnée.
    WindowResized(iced::Size),
    /// Navigation vers un écran.
    Navigate(Route),
    /// Recharge les données depuis SQLite.
    Reload,
    /// Modifie la recherche globale de la page courante.
    SearchChanged(String),
    /// Bascule le rendu candidatures.
    CandidateViewChanged(CandidateView),
    /// Affiche le mois précédent du calendrier.
    PreviousMonth,
    /// Affiche le mois suivant du calendrier.
    NextMonth,
    /// Revient au mois actuel.
    CurrentMonth,
    /// Bascule entre thème clair et sombre.
    ToggleTheme,
    /// Battement de l'horloge pour les durées et opérations progressives.
    Tick,
    /// Lance une vérification de mise à jour.
    CheckUpdate,
    /// Résultat de la vérification de mise à jour.
    UpdateChecked(Result<Option<crate::core::updater::UpdateInfo>, String>),
    /// Efface la notification courante.
    ClearNotification,
    /// Ouvre un formulaire métier.
    OpenDialog(Dialog),
    /// Ferme le formulaire en cours.
    CloseDialog,
    /// Modifie le nom d'entreprise.
    EntrepriseNomChanged(String),
    /// Modifie le secteur d'entreprise.
    EntrepriseSecteurChanged(String),
    /// Modifie le type d'entreprise.
    EntrepriseTypeChanged(String),
    /// Modifie le site d'entreprise.
    EntrepriseSiteChanged(String),
    /// Modifie la ville d'entreprise.
    EntrepriseVilleChanged(String),
    /// Modifie l'adresse d'entreprise.
    EntrepriseAdresseChanged(String),
    /// Modifie les notes d'entreprise.
    EntrepriseNotesChanged(String),
    /// Soumet une entreprise.
    SubmitEntreprise,
    /// Modifie le prénom du contact.
    ContactPrenomChanged(String),
    /// Modifie le nom du contact.
    ContactNomChanged(String),
    /// Modifie le poste du contact.
    ContactPosteChanged(String),
    /// Modifie l'e-mail du contact.
    ContactEmailChanged(String),
    /// Modifie le téléphone du contact.
    ContactTelephoneChanged(String),
    /// Modifie le profil LinkedIn du contact.
    ContactLinkedinChanged(String),
    /// Modifie les notes du contact.
    ContactNotesChanged(String),
    /// Lie le contact à une entreprise.
    ContactEntrepriseChanged(Option<uuid::Uuid>),
    /// Soumet un contact.
    SubmitContact,
    /// Modifie le poste d'une candidature.
    CandidaturePosteChanged(String),
    /// Lie une candidature à une entreprise.
    CandidatureEntrepriseChanged(uuid::Uuid),
    /// Modifie le type de contrat.
    CandidatureContratChanged(TypeContrat),
    /// Modifie le statut du formulaire candidature.
    CandidatureStatutChanged(StatutCandidature),
    /// Modifie la date d'envoi.
    CandidatureDateChanged(String),
    /// Modifie le lien d'offre.
    CandidatureLienChanged(String),
    /// Modifie les notes de candidature.
    CandidatureNotesChanged(String),
    /// Soumet une candidature.
    SubmitCandidature,
    /// Persiste le statut d'une candidature existante.
    MoveCandidature(uuid::Uuid, StatutCandidature),
    /// Lie un entretien à une candidature.
    EntretienCandidatureChanged(uuid::Uuid),
    /// Lie un entretien à un contact.
    EntretienContactChanged(Option<uuid::Uuid>),
    /// Modifie la date d'entretien.
    EntretienDateChanged(String),
    /// Modifie le type d'entretien.
    EntretienTypeChanged(TypeEntretien),
    /// Modifie le lieu d'entretien.
    EntretienLieuChanged(String),
    /// Modifie les notes d'entretien.
    EntretienNotesChanged(String),
    /// Modifie le compte rendu.
    EntretienCompteRenduChanged(String),
    /// Soumet un entretien.
    SubmitEntretien,
    /// Lie une relance à une candidature.
    RelanceCandidatureChanged(uuid::Uuid),
    /// Modifie la date de relance.
    RelanceDateChanged(String),
    /// Modifie le canal de relance.
    RelanceTypeChanged(String),
    /// Modifie les notes de relance.
    RelanceNotesChanged(String),
    /// Soumet une relance.
    SubmitRelance,
    /// Demande l'export CSV filtré.
    ExportCandidatures,
    /// Résultat de l'export CSV.
    CandidaturesExported(Result<std::path::PathBuf, String>),
    /// Modifie le texte de l'offre dans l'éditeur natif.
    OfferEditorAction(iced::widget::text_editor::Action),
    /// Lance l'analyse de l'offre.
    AnalyzeOffer,
    /// Résultat de l'analyse d'offre.
    OfferAnalyzed(Result<crate::modules::ia::cv_model::OfferAnalysis, String>),
    /// Lance la génération du CV depuis l'analyse.
    GenerateCv,
    /// Résultat de la génération du CV.
    CvGenerated(Result<crate::modules::ia::cv_model::CvGeneration, String>),
    /// Annule l'opération IA active.
    CancelAi,
    /// Ouvre le sélecteur de PDF pour l'analyse externe.
    SelectImportPdf,
    /// PDF externe sélectionné.
    ImportPdfSelected(Option<std::path::PathBuf>),
    /// Modifie l'offre associée au CV externe.
    ImportOfferEditorAction(iced::widget::text_editor::Action),
    /// Lance l'analyse du CV externe.
    AnalyzeImportedCv,
    /// Résultat de l'analyse du CV externe.
    ImportedCvAnalyzed(Result<crate::modules::ia::service::ImportedCvAnalysis, String>),
    /// Modifie l'entreprise de la lettre.
    LetterCompanyChanged(String),
    /// Modifie le poste de la lettre.
    LetterJobTitleChanged(String),
    /// Modifie le contexte de la lettre.
    LetterEditorAction(iced::widget::text_editor::Action),
    /// Modifie le ton de la lettre.
    LetterToneChanged(String),
    /// Modifie la longueur de la lettre.
    LetterLengthChanged(String),
    /// Lance la génération progressive de lettre.
    GenerateLetter,
    /// Fragment ou résultat terminal du streaming.
    LetterStream(LetterStreamEvent),
    /// Modifie le fournisseur IA.
    SettingsProviderChanged(crate::shared::llm::ProviderKind),
    /// Modifie le modèle IA.
    SettingsModelChanged(String),
    /// Modifie l'endpoint IA.
    SettingsEndpointChanged(String),
    /// Modifie la clé API transitoire.
    SettingsApiKeyChanged(String),
    /// Modifie la température IA.
    SettingsTemperatureChanged(f32),
    /// Modifie le mode d'analyse.
    SettingsModeChanged(crate::shared::llm::AnalysisMode),
    /// Modifie la préférence de thème.
    SettingsThemeChanged(crate::modules::settings::model::ThemePref),
    /// Persiste les paramètres et le secret.
    SaveSettings,
    /// Résultat de la sauvegarde des paramètres.
    SettingsSaved(Result<crate::modules::settings::model::AppSettings, String>),
    /// Teste le provider saisi.
    TestLlmConnection,
    /// Résultat du test provider.
    LlmConnectionTested(Result<(), String>),
    /// Exporte une sauvegarde SQLite cohérente.
    ExportBackup,
    /// Résultat de la sauvegarde SQLite.
    BackupExported(Result<std::path::PathBuf, String>),
    /// Commence le glisser-déposer d'une candidature.
    CandidateDragStarted(uuid::Uuid),
    /// Met à jour la colonne survolée.
    CandidateDragHovered(StatutCandidature),
    /// Dépose la candidature dans une colonne.
    CandidateDropped(StatutCandidature),
    /// Change la granularité du calendrier.
    CalendarViewChanged(CalendarView),
    /// Sélectionne une date dans le calendrier.
    CalendarDateSelected(chrono::NaiveDate),
    /// Modifie le prénom du profil.
    ProfileFirstNameChanged(String),
    /// Modifie le nom du profil.
    ProfileLastNameChanged(String),
    /// Modifie l'e-mail du profil.
    ProfileEmailChanged(String),
    /// Modifie le téléphone du profil.
    ProfilePhoneChanged(String),
    /// Modifie la ville du profil.
    ProfileCityChanged(String),
    /// Modifie le titre professionnel.
    ProfileHeadlineChanged(String),
    /// Modifie le résumé du profil.
    ProfileSummaryChanged(String),
    /// Modifie la liste de compétences.
    ProfileSkillsChanged(String),
    /// Enregistre le profil.
    SubmitProfile,
    /// Télécharge et vérifie la mise à jour disponible.
    DownloadUpdate,
    /// Progression ou résultat du téléchargement.
    UpdateDownload(UpdateDownloadEvent),
    /// Filtre les candidatures par statut.
    CandidateFilterStatus(Option<StatutCandidature>),
    /// Filtre les candidatures par contrat.
    CandidateFilterContract(Option<TypeContrat>),
    /// Filtre les candidatures par entreprise.
    CandidateFilterCompany(Option<uuid::Uuid>),
    /// Filtre les candidatures par ville de l'entreprise.
    CandidateFilterCity(String),
    /// Filtre les candidatures par intitulé de poste.
    CandidateFilterPosition(String),
    /// Définit la borne de date minimale.
    CandidateFilterDateFrom(String),
    /// Définit la borne de date maximale.
    CandidateFilterDateTo(String),
    /// Réinitialise les filtres sans vider la recherche globale.
    ResetCandidateFilters,
    /// Confirme la suppression affichée.
    ConfirmDelete,
    /// Ouvre l'édition d'une entreprise.
    EditEntreprise(uuid::Uuid),
    /// Ouvre l'édition d'un contact.
    EditContact(uuid::Uuid),
    /// Ouvre l'édition d'une candidature.
    EditCandidature(uuid::Uuid),
    /// Ouvre l'édition d'un entretien.
    EditEntretien(uuid::Uuid),
    /// Ouvre l'édition d'une relance.
    EditRelance(uuid::Uuid),
    /// Modifie le nom de la version CV.
    CvVersionNameChanged(String),
    /// Sauvegarde la génération courante.
    SaveGeneratedCv,
    /// Charge une version de CV.
    LoadCvVersion(uuid::Uuid),
    /// Exporte le CV généré en PDF.
    ExportGeneratedCvPdf,
    /// Résultat de l'export PDF.
    CvPdfExported(Result<std::path::PathBuf, String>),
    /// Choisit un PDF pour l'import du profil.
    SelectProfilePdf,
    /// PDF de profil sélectionné.
    ProfilePdfSelected(Option<std::path::PathBuf>),
    /// Lance l'extraction du profil.
    ExtractProfile,
    /// Résultat de l'extraction de profil.
    ProfileExtracted(Result<crate::shared::profile::Profile, String>),
    /// Persiste le profil extrait après validation explicite.
    ApplyExtractedProfile,
    /// Analyse le compte rendu d'un entretien.
    AnalyzeInterview(uuid::Uuid),
    /// Résultat de l'analyse d'entretien.
    InterviewAnalyzed(Result<crate::shared::types::AnalyseEntretien, String>),
    /// Sélectionne un backup SQLite à restaurer.
    SelectBackupImport,
    /// Backup sélectionné et validé par le dialogue.
    BackupImportSelected(Option<std::path::PathBuf>),
    /// Confirme la restauration du backup.
    ConfirmBackupImport,
    /// Confirme la réinitialisation de toutes les données.
    ConfirmDatabaseReset,
    /// Confirme la purge du cache IA local.
    ConfirmAiCacheReset,
    /// Accepte et applique une recommandation ATS.
    AcceptRecommendation(usize),
    /// Refuse une recommandation ATS.
    RejectRecommendation(usize),
    /// Sélectionne une candidature dans le pipeline ou la liste.
    SelectCandidate(Option<uuid::Uuid>),
    /// Sélectionne une entreprise dans le répertoire.
    SelectCompany(Option<uuid::Uuid>),
    /// Sélectionne un contact dans le réseau.
    SelectContact(Option<uuid::Uuid>),
    /// Sélectionne une version de CV dans la bibliothèque.
    SelectCvVersion(Option<uuid::Uuid>),
    /// Déplie ou replie la feuille de filtres.
    ToggleFilters,
    /// Trie la vue Liste sur la colonne d'index donné.
    SortCandidates(usize),
    /// Change la section active des paramètres.
    SettingsSectionChanged(SettingsSection),
    /// Change la section active du profil.
    ProfileSectionChanged(ProfileSection),
    /// Change l'onglet actif des statistiques.
    StatisticsTabChanged(StatisticsTab),
    /// Page précédente de l'historique des scores ATS.
    AtsPagePrev,
    /// Page suivante de l'historique des scores ATS.
    AtsPageNext,
    /// Page précédente de l'historique des appels IA.
    LlmPagePrev,
    /// Page suivante de l'historique des appels IA.
    LlmPageNext,
    /// Modifie la largeur de page des plans de travail de document.
    DocumentWidthChanged(f32),
    /// Donne le focus à la recherche de l'écran courant.
    FocusSearch,
}
