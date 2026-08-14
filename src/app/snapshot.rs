//! Construction paginée des instantanés de données affichés par l'application.

use crate::modules::candidatures::model::StatutCandidature;
use crate::modules::candidatures::repository::CandidaturePageQuery;
use crate::modules::metriques::components::PipelineCounts;
use crate::modules::metriques::repository::MetriquesRepository;
use crate::navigation::Route;
use crate::shared::error::AppError;
use crate::shared::state::AppState as BackendState;

use super::state::{CandidateFilters, CandidateSort, DataSnapshot};

/// Taille des pages métier. Suffisamment dense pour le desktop, mais toujours bornée en SQL.
pub const BUSINESS_PAGE_SIZE: u64 = 24;
/// Taille des pages des sélecteurs relationnels recherchables.
pub const RELATION_PAGE_SIZE: u64 = 40;

/// Paramètres immuables d'un rechargement paginé.
#[derive(Debug, Clone)]
pub struct SnapshotRequest {
    pub sequence: u64,
    pub route: Route,
    pub search: String,
    pub company_type_filter: Option<String>,
    pub candidate_filters: CandidateFilters,
    pub candidate_sort: CandidateSort,
    pub candidate_sort_descending: bool,
    pub candidate_page: u64,
    pub company_page: u64,
    pub contact_page: u64,
    pub company_option_search: String,
    pub candidate_option_search: String,
    pub contact_option_search: String,
    pub company_option_page: u64,
    pub candidate_option_page: u64,
    pub contact_option_page: u64,
    pub selected_company_option: Option<uuid::Uuid>,
    pub selected_candidate_option: Option<uuid::Uuid>,
    pub selected_contact_option: Option<uuid::Uuid>,
    pub calendar_year: i32,
    pub calendar_month: u32,
    pub llm_page: u64,
    pub ats_page: u64,
}

impl SnapshotRequest {
    /// Traduit l'état d'écran en critères SQL, également réutilisés par l'export complet.
    #[must_use]
    pub fn candidate_query(&self) -> CandidaturePageQuery {
        CandidaturePageQuery {
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

/// Charge l'instantané complet et la liste des jeux qui n'ont pas pu être lus.
///
/// Cette fonction ne touche pas à `App`, ce qui lui permet d'être exécutée sur un fil de
/// travail (`spawn_blocking`) plutôt que sur le fil de rendu.
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
    let company_type = (request.route == Route::Entreprises)
        .then_some(request.company_type_filter.as_deref())
        .flatten();
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
        backend.entreprises.lister_page(
            request.company_page,
            BUSINESS_PAGE_SIZE,
            company_search,
            company_type,
        ),
    );
    let contact_page = charger(
        "contacts",
        &mut echecs,
        backend
            .contacts
            .lister_page(request.contact_page, BUSINESS_PAGE_SIZE, contact_search),
    );
    // La valeur sélectionnée est ajoutée à la page afin que l'édition reste fidèle.
    let mut company_options = charger(
        "options entreprises",
        &mut echecs,
        backend.entreprises.lister_page(
            request.company_option_page,
            RELATION_PAGE_SIZE,
            &request.company_option_search,
            None,
        ),
    );
    if let Some(id) = request.selected_company_option {
        if !company_options.items.iter().any(|item| item.id == id) {
            if let Ok(item) = backend.entreprises.obtenir(id) {
                company_options.items.push(item);
            }
        }
    }
    let option_query = CandidaturePageQuery {
        search: request.candidate_option_search.clone(),
        descending: true,
        ..CandidaturePageQuery::default()
    };
    let mut candidate_options = charger(
        "options candidatures",
        &mut echecs,
        backend.candidatures.lister_page(
            request.candidate_option_page,
            RELATION_PAGE_SIZE,
            &option_query,
        ),
    );
    if let Some(id) = request.selected_candidate_option {
        if !candidate_options.items.iter().any(|item| item.id == id) {
            if let Ok(item) = backend.candidatures.obtenir(id) {
                candidate_options.items.push(item);
            }
        }
    }
    let mut contact_options = charger(
        "options contacts",
        &mut echecs,
        backend.contacts.lister_page(
            request.contact_option_page,
            RELATION_PAGE_SIZE,
            &request.contact_option_search,
        ),
    );
    if let Some(id) = request.selected_contact_option {
        if !contact_options.items.iter().any(|item| item.id == id) {
            if let Ok(item) = backend.contacts.obtenir(id) {
                contact_options.items.push(item);
            }
        }
    }
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
        company_types: charger(
            "types d'entreprise",
            &mut echecs,
            backend.entreprises.lister_types(),
        ),
        contacts: contact_page.items,
        contacts_total: contact_page.total,
        contacts_total_pages: contact_page.total_pages,
        company_options: company_options.items,
        company_options_total: company_options.total,
        company_options_total_pages: company_options.total_pages,
        candidate_options: candidate_options.items,
        candidate_options_total: candidate_options.total,
        candidate_options_total_pages: candidate_options.total_pages,
        contact_options: contact_options.items,
        contact_options_total: contact_options.total,
        contact_options_total_pages: contact_options.total_pages,
        entretiens: charger(
            "entretiens",
            &mut echecs,
            backend.entretiens.lister_entre(&from, &to),
        ),
        relances: charger(
            "relances",
            &mut echecs,
            backend.relances.lister_entre(&from, &to),
        ),
        cv_versions: charger("CV", &mut echecs, backend.cv.list()),
        letters: charger("lettres de motivation", &mut echecs, backend.lettres.list()),
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
    query: &CandidaturePageQuery,
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
