//! État global de l'application native.

use crate::core::config::AppPaths;
use crate::modules::candidatures::model::Candidature;
use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
use crate::modules::contacts::model::Contact;
use crate::modules::cv::model::CvVersionSummary;
use crate::modules::entreprises::model::Entreprise;
use crate::modules::entretiens::model::{Entretien, TypeEntretien};
use crate::modules::ia::cv_model::{CvGeneration, OfferAnalysis};
use crate::modules::metriques::components::PipelineCounts;
use crate::modules::metriques::model::{AppelLlm, Page, ResumeScoresAts, ScoreAts};
use crate::modules::metriques::repository::MetriquesRepository;
use crate::modules::relances::model::Relance;
use crate::modules::settings::model::AppSettings;
use crate::navigation::Route;
use crate::shared::error::AppError;
use crate::shared::profile::Profile;
use crate::shared::state::AppState as BackendState;
pub use crate::ui::components::notification::Kind as NotificationKind;
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

/// Champ auquel le calendrier flottant doit appliquer la date choisie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePickerTarget {
    Candidature,
    Entretien,
    Relance,
    FiltreDebut,
    FiltreFin,
}

#[derive(Debug, Clone, Copy)]
pub struct DatePickerState {
    pub target: DatePickerTarget,
    pub year: i32,
    pub month: u32,
}

/// Collections structurées éditables dans le profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileCollection {
    Experience,
    Formation,
    Langue,
    Projet,
    Certification,
}

/// État du formulaire entreprise.
#[derive(Debug, Default)]
pub struct EntrepriseForm {
    pub nom: String,
    pub secteur: String,
    pub type_: String,
    pub site_web: String,
    pub ville: String,
    pub adresse: String,
    pub notes: iced::widget::text_editor::Content,
}

/// État du formulaire contact.
#[derive(Debug, Default)]
pub struct ContactForm {
    pub entreprise_id: Option<uuid::Uuid>,
    pub prenom: String,
    pub nom: String,
    pub poste: String,
    pub email: String,
    pub telephone: String,
    pub linkedin: String,
    pub notes: iced::widget::text_editor::Content,
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
            date_envoi: chrono::Local::now().format("%d-%m-%Y").to_string(),
            lien_offre: String::new(),
            notes: String::new(),
        }
    }
}

/// État du formulaire entretien.
#[derive(Debug)]
pub struct EntretienForm {
    pub candidature_id: Option<uuid::Uuid>,
    pub contact_id: Option<uuid::Uuid>,
    pub date_entretien: String,
    pub type_entretien: TypeEntretien,
    pub lieu: String,
    pub notes: iced::widget::text_editor::Content,
    pub compte_rendu: iced::widget::text_editor::Content,
}

impl Default for EntretienForm {
    fn default() -> Self {
        Self {
            candidature_id: None,
            contact_id: None,
            date_entretien: chrono::Local::now().format("%d-%m-%Y %H:%M").to_string(),
            type_entretien: TypeEntretien::Presentiel,
            lieu: String::new(),
            notes: iced::widget::text_editor::Content::new(),
            compte_rendu: iced::widget::text_editor::Content::new(),
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

/// Formulaire de l'écran Paramètres, **distinct** de l'instantané de données.
///
/// Les huit messages d'édition écrivaient directement dans `app.data.settings`, c'est-à-dire
/// dans la copie censée refléter le contenu de la base, alors que les cinq autres formulaires
/// du projet disposent tous d'une structure d'édition séparée. Trois conséquences : un échec
/// d'enregistrement laissait `app.data.settings` divergé de la base sans qu'aucun rechargement
/// ne vienne le corriger ; quitter l'écran sans enregistrer conservait les modifications en
/// mémoire ; et le bandeau de titre annonçait aussitôt un fournisseur qui n'était pas celui
/// réellement persisté.
///
/// La clé API n'y transite que le temps de la saisie — le champ de l'instantané porte la
/// mention « sans secret en clair », qu'y déposer la clé contredisait.
#[derive(Debug, Clone, Default)]
pub struct SettingsForm {
    /// Valeurs en cours d'édition.
    pub draft: AppSettings,
}

impl SettingsForm {
    /// Initialise le formulaire à l'ouverture de l'écran, depuis l'état persisté.
    #[must_use]
    pub fn from_settings(settings: &AppSettings) -> Self {
        Self {
            draft: settings.clone(),
        }
    }
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

/// Paramètres immuables d'un rechargement paginé.
#[derive(Debug, Clone)]
pub struct SnapshotRequest {
    pub sequence: u64,
    pub route: Route,
    pub search: String,
    pub candidate_filters: CandidateFilters,
    pub candidate_sort: CandidateSort,
    pub candidate_sort_descending: bool,
    pub candidate_page: u64,
    pub company_page: u64,
    pub contact_page: u64,
    pub calendar_year: i32,
    pub calendar_month: u32,
    pub llm_page: u64,
    pub ats_page: u64,
}

impl SnapshotRequest {
    /// Traduit l'état d'écran en critères SQL, également réutilisés par l'export complet.
    #[must_use]
    pub fn candidate_query(
        &self,
    ) -> crate::modules::candidatures::repository::CandidaturePageQuery {
        crate::modules::candidatures::repository::CandidaturePageQuery {
            search: if self.route == Route::Candidatures {
                self.search.clone()
            } else {
                String::new()
            },
            status: self.candidate_filters.status,
            contract: self.candidate_filters.contract,
            company_id: self.candidate_filters.company_id,
            city: self.candidate_filters.city.clone(),
            position: self.candidate_filters.position.clone(),
            date_from: crate::ui::format::date_to_storage(&self.candidate_filters.date_from).ok(),
            date_to: crate::ui::format::date_to_storage(&self.candidate_filters.date_to).ok(),
            sort: match self.candidate_sort {
                CandidateSort::Poste => "poste",
                CandidateSort::Entreprise => "entreprise",
                CandidateSort::Statut => "statut",
                CandidateSort::Date => "date",
            }
            .into(),
            descending: self.candidate_sort_descending,
        }
    }
}

/// Taille des pages métier. Suffisamment dense pour le desktop, mais toujours bornée en SQL.
pub const BUSINESS_PAGE_SIZE: u64 = 24;

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
            date_relance: chrono::Local::now().format("%d-%m-%Y").to_string(),
            type_relance: "Email".into(),
            notes: String::new(),
        }
    }
}

/// Le harnais de capture visuelle est-il demandé ?
///
/// Toujours faux sans la caractéristique Cargo `capture` : le binaire distribué ne doit ni lire
/// ces variables, ni écrire de fichier au chemin qu'elles désignent.
#[must_use]
pub fn capture_demandee() -> bool {
    cfg!(feature = "capture") && std::env::var_os("CANDILOG_CAPTURE_PATH").is_some()
}

/// Charge l'instantané complet et la liste des jeux qui n'ont pas pu être lus.
///
/// Fonction **libre** et non méthode : elle ne touche pas à `App`, ce qui lui permet d'être
/// exécutée sur un fil de travail (`spawn_blocking`) plutôt que sur le fil de rendu.
#[must_use]
pub fn charger_instantane(
    backend: &BackendState,
    request: &SnapshotRequest,
) -> (DataSnapshot, Vec<&'static str>) {
    let mut echecs: Vec<&'static str> = Vec::new();
    let taille = crate::modules::metriques::views::PAGE_SIZE;
    let company_search = if request.route == Route::Entreprises {
        request.search.as_str()
    } else {
        ""
    };
    let contact_search = if request.route == Route::Reseau {
        request.search.as_str()
    } else {
        ""
    };
    let candidate_query = request.candidate_query();
    let candidate_page = charger(
        "candidatures",
        &mut echecs,
        backend.candidatures.lister_page(
            request.candidate_page,
            BUSINESS_PAGE_SIZE,
            &candidate_query,
        ),
    );
    let filtered_candidate_counts = charger(
        "compteurs filtrés du pipeline",
        &mut echecs,
        compter_pipeline_filtre(backend, &candidate_query),
    );
    let company_page = charger(
        "entreprises",
        &mut echecs,
        backend
            .entreprises
            .lister_page(request.company_page, BUSINESS_PAGE_SIZE, company_search),
    );
    let contact_page = charger(
        "contacts",
        &mut echecs,
        backend
            .contacts
            .lister_page(request.contact_page, BUSINESS_PAGE_SIZE, contact_search),
    );
    // Catalogues de sélecteurs explicitement bornés : ils préservent les formulaires liés sans
    // réintroduire un `SELECT *` non limité dans l'instantané global.
    let company_options = charger(
        "options entreprises",
        &mut echecs,
        backend.entreprises.lister_page(1, 200, ""),
    );
    let candidate_options = charger(
        "options candidatures",
        &mut echecs,
        backend.candidatures.lister_page(
            1,
            200,
            &crate::modules::candidatures::repository::CandidaturePageQuery::default(),
        ),
    );
    let contact_options = charger(
        "options contacts",
        &mut echecs,
        backend.contacts.lister_page(1, 200, ""),
    );
    let month_start = chrono::NaiveDate::from_ymd_opt(
        request.calendar_year,
        request.calendar_month.clamp(1, 12),
        1,
    )
    .unwrap_or_else(|| chrono::Local::now().date_naive());
    let from = (month_start - chrono::Duration::days(8))
        .format("%Y-%m-%d")
        .to_string();
    let to = (month_start + chrono::Duration::days(40))
        .format("%Y-%m-%dT23:59")
        .to_string();
    let follow_up_before = (chrono::Local::now().date_naive() - chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let data = DataSnapshot {
        candidatures: candidate_page.items,
        candidatures_total: candidate_page.total,
        candidatures_total_pages: candidate_page.total_pages,
        filtered_candidate_counts,
        candidature_stats: charger(
            "statistiques candidatures",
            &mut echecs,
            backend.candidatures.statistiques(),
        ),
        follow_up_candidates: charger(
            "candidatures à relancer",
            &mut echecs,
            backend.candidatures.a_relancer(&follow_up_before, 6),
        ),
        entreprises: company_page.items,
        entreprises_total: company_page.total,
        entreprises_total_pages: company_page.total_pages,
        contacts: contact_page.items,
        contacts_total: contact_page.total,
        contacts_total_pages: contact_page.total_pages,
        company_options: company_options.items,
        candidate_options: candidate_options.items,
        contact_options: contact_options.items,
        entretiens: charger(
            "entretiens",
            &mut echecs,
            backend.entretiens.lister_entre(&from, &to),
        ),
        relances: charger("relances", &mut echecs, backend.relances.lister()),
        cv_versions: charger("CV", &mut echecs, backend.cv.list()),
        profile: charger("profil", &mut echecs, backend.profil.get()),
        settings: charger("paramètres", &mut echecs, backend.settings.get()),
        llm_calls: charger(
            "historique IA",
            &mut echecs,
            backend
                .metriques
                .lister_appels_page(request.llm_page, taille),
        ),
        ats_scores: charger(
            "scores ATS",
            &mut echecs,
            backend
                .metriques
                .lister_scores_page(request.ats_page, taille),
        ),
        ats_summary: charger(
            "synthèse ATS",
            &mut echecs,
            backend.metriques.resumer_scores().map(Some),
        ),
    };
    (data, echecs)
}

fn compter_pipeline_filtre(
    backend: &BackendState,
    query: &crate::modules::candidatures::repository::CandidaturePageQuery,
) -> Result<PipelineCounts, AppError> {
    let mut counts = PipelineCounts::default();
    for status in crate::modules::candidatures::components::PIPELINE {
        if query.status.is_some_and(|selected| selected != status) {
            continue;
        }
        let mut status_query = query.clone();
        status_query.status = Some(status);
        let total = backend.candidatures.lister_page(1, 1, &status_query)?.total;
        let total = usize::try_from(total).unwrap_or(usize::MAX);
        match status {
            StatutCandidature::EnAttente => counts.pending = total,
            StatutCandidature::Relancee => counts.followed_up = total,
            StatutCandidature::Entretien => counts.interviews = total,
            StatutCandidature::Refus => counts.rejected = total,
        }
        counts.total = counts.total.saturating_add(total);
    }
    Ok(counts)
}

/// Charge un jeu de données isolément : un échec est journalisé, recensé, et remplacé par la
/// valeur par défaut du type plutôt que d'interrompre le chargement des dix autres.
fn charger<T: Default>(
    nom: &'static str,
    echecs: &mut Vec<&'static str>,
    resultat: Result<T, AppError>,
) -> T {
    match resultat {
        Ok(valeur) => valeur,
        Err(error) => {
            tracing::error!(jeu = nom, erreur = %error, "jeu de données illisible");
            echecs.push(nom);
            T::default()
        }
    }
}

/// Message adressé à l'utilisateur, **avec** sa nature.
///
/// La nature accompagne le texte au lieu d'être redevinée au moment du rendu : toutes les
/// erreurs étaient converties en `String` dès `update()`, puis reclassées par recherche de
/// mots-clés, avec `Success` pour cas par défaut — un échec sur deux s'affichait en vert.
#[derive(Debug, Clone)]
pub struct Notification {
    /// Nature, qui détermine le ton et l'icône du toast.
    pub kind: NotificationKind,
    /// Texte affiché, déjà destiné à l'utilisateur.
    pub message: String,
}

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
    /// Contacts.
    pub contacts: Vec<Contact>,
    pub contacts_total: u64,
    pub contacts_total_pages: u64,
    /// Catalogues bornés pour les relations des formulaires.
    pub company_options: Vec<Entreprise>,
    pub candidate_options: Vec<Candidature>,
    pub contact_options: Vec<Contact>,
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
    /// Paquet de mise à jour téléchargé et vérifié.
    pub verified_update_path: Option<std::path::PathBuf>,
    /// Backup sélectionné avant confirmation de restauration.
    pub pending_backup_import: Option<std::path::PathBuf>,
    /// Dialogue métier ouvert.
    pub dialog: Option<Dialog>,
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
    /// Pages des trois répertoires métier, indexées à partir de 1.
    pub candidate_page: u64,
    pub company_page: u64,
    pub contact_page: u64,
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
            provider_health: crate::ui::components::runtime_status::Health::default(),
            available_models: Vec::new(),
            available_update: None,
            update_progress: None,
            verified_update_path: None,
            pending_backup_import: None,
            dialog: None,
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
            selected_candidate: None,
            selected_company: None,
            selected_contact: None,
            selected_cv: None,
            filters_open: false,
            candidate_sort: CandidateSort::default(),
            candidate_sort_descending: true,
            candidate_page: 1,
            company_page: 1,
            contact_page: 1,
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
    /// n'est pas destiné aux utilisateurs : ses onze variables d'environnement modifient le
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
        if std::env::var("CANDILOG_CAPTURE_THEME").as_deref() == Ok("light") {
            self.is_dark = false;
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
                "profil" => Some(Dialog::Profil),
                _ => None,
            };
        }
    }

    /// Réglages du harnais de capture qui supposent les données chargées.
    #[cfg(feature = "capture")]
    fn appliquer_harnais_de_capture_apres_ouverture(&mut self) {
        // `bootstrap` recharge le thème persisté ; l'override de capture doit donc être
        // réappliqué après l'ouverture de la base pour rester prioritaire.
        if std::env::var("CANDILOG_CAPTURE_THEME").as_deref() == Ok("light") {
            self.is_dark = false;
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
        if self.dialog == Some(Dialog::Profil) {
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
    /// échec sur l'un des onze jeux abandonnait les dix autres et laissait `initialized` à faux,
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
            candidate_filters: self.candidate_filters.clone(),
            candidate_sort: self.candidate_sort,
            candidate_sort_descending: self.candidate_sort_descending,
            candidate_page: self.candidate_page,
            company_page: self.company_page,
            contact_page: self.contact_page,
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

    /// Applique la recherche et les six catégories de filtres aux candidatures.
    #[must_use]
    pub fn filtered_candidates(&self) -> Vec<&Candidature> {
        let search = self.search.trim().to_lowercase();
        let position = self.candidate_filters.position.trim().to_lowercase();
        let city = self.candidate_filters.city.trim().to_lowercase();
        let date_from = crate::ui::format::date_to_storage(&self.candidate_filters.date_from).ok();
        let date_to = crate::ui::format::date_to_storage(&self.candidate_filters.date_to).ok();
        // Index des villes, construit **une seule fois** et **seulement** si le filtre par
        // ville est actif. La recherche de l'entreprise était auparavant faite pour chaque
        // candidature — un balayage linéaire complet, inconditionnel, avant même de savoir si
        // sa valeur servirait — soit un coût O(n×m) payé à chaque cycle de rendu.
        let villes: std::collections::HashMap<uuid::Uuid, String> = if city.is_empty() {
            std::collections::HashMap::new()
        } else {
            self.data
                .entreprises
                .iter()
                .map(|company| {
                    (
                        company.id,
                        company.ville.as_deref().unwrap_or_default().to_lowercase(),
                    )
                })
                .collect()
        };
        self.data
            .candidatures
            .iter()
            .filter(|candidate| {
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
                        || villes
                            .get(&candidate.entreprise_id)
                            .is_some_and(|ville| ville.contains(&city)))
                    && date_from
                        .as_deref()
                        .is_none_or(|from| candidate.date_envoi.as_str() >= from)
                    && date_to
                        .as_deref()
                        .is_none_or(|to| candidate.date_envoi.as_str() <= to)
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

#[cfg(test)]
#[path = "tests/state/mod.rs"]
mod tests;
