//! Instantané des données chargées pour les écrans.

use crate::modules::candidatures::model::Candidature;
use crate::modules::contacts::model::Contact;
use crate::modules::cv::model::CvVersionSummary;
use crate::modules::entreprises::model::Entreprise;
use crate::modules::entretiens::model::Entretien;
use crate::modules::lettres::model::LettreMotivation;
use crate::modules::metriques::components::PipelineCounts;
use crate::modules::metriques::model::{AppelLlm, Page, ResumeScoresAts, ScoreAts};
use crate::modules::relances::model::Relance;
use crate::modules::settings::model::AppSettings;
use crate::shared::profile::Profile;

/// Données chargées pour les différents écrans.
#[derive(Debug, Clone, Default)]
pub struct DataSnapshot {
    /// Candidatures.
    pub candidatures: Vec<Candidature>,
    pub candidatures_total: u64,
    pub candidatures_total_pages: u64,
    /// Compteurs globaux du pipeline après application des filtres et de la recherche.
    pub filtered_candidate_counts: PipelineCounts,
    /// Agrégats globaux indépendants de la page affichée.
    pub candidature_stats: crate::modules::candidatures::repository::CandidatureStats,
    /// Aperçu borné des candidatures nécessitant une relance.
    pub follow_up_candidates: Vec<Candidature>,
    /// Entreprises.
    pub entreprises: Vec<Entreprise>,
    pub entreprises_total: u64,
    pub entreprises_total_pages: u64,
    /// Types d'entreprise réellement présents, triés et dédupliqués.
    pub company_types: Vec<String>,
    /// Contacts.
    pub contacts: Vec<Contact>,
    pub contacts_total: u64,
    pub contacts_total_pages: u64,
    /// Pages recherchables pour les relations des formulaires.
    pub company_options: Vec<Entreprise>,
    pub company_options_total: u64,
    pub company_options_total_pages: u64,
    pub candidate_options: Vec<Candidature>,
    pub candidate_options_total: u64,
    pub candidate_options_total_pages: u64,
    pub contact_options: Vec<Contact>,
    pub contact_options_total: u64,
    pub contact_options_total_pages: u64,
    /// Entretiens.
    pub entretiens: Vec<Entretien>,
    /// Relances.
    pub relances: Vec<Relance>,
    /// Versions de CV.
    pub cv_versions: Vec<CvVersionSummary>,
    /// Lettres de motivation enregistrées.
    pub letters: Vec<LettreMotivation>,
    /// Profil structuré.
    pub profile: Profile,
    /// Paramètres applicatifs sans secret en clair.
    pub settings: AppSettings,
    /// Historique des appels IA.
    ///
    /// **Page** et non liste complète : `reload()` chargeait tout l'historique en mémoire à
    /// chaque rechargement — donc après chaque création, modification et suppression — par un
    /// `SELECT … ORDER BY cree_le DESC` sans `LIMIT`, puis paginait en mémoire à l'affichage.
    /// Le coût croissait linéairement avec l'ancienneté de l'installation et était payé même
    /// quand l'écran Statistiques n'était pas ouvert. Les méthodes paginées du dépôt
    /// existaient déjà et n'étaient appelées que par leurs propres tests.
    pub llm_calls: Page<AppelLlm>,
    /// Historique des scores ATS.
    pub ats_scores: Page<ScoreAts>,
    /// Agrégats ATS globaux.
    pub ats_summary: Option<ResumeScoresAts>,
}
