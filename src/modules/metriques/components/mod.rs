//! Préparation des données d'indicateurs, indépendante du rendu.

use crate::modules::candidatures::model::{Candidature, StatutCandidature};
use crate::modules::metriques::model::AppelLlm;

/// Effectifs du pipeline, prêts à être affichés.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PipelineCounts {
    /// Total suivi.
    pub total: usize,
    /// Candidatures en attente de réponse.
    pub pending: usize,
    /// Candidatures relancées.
    pub followed_up: usize,
    /// Candidatures ayant abouti à un entretien.
    pub interviews: usize,
    /// Candidatures refusées.
    pub rejected: usize,
}

impl PipelineCounts {
    /// Construit les effectifs depuis les agrégats globaux calculés par SQLite.
    #[must_use]
    pub fn from_stats(stats: &crate::modules::candidatures::repository::CandidatureStats) -> Self {
        Self {
            total: usize::try_from(stats.total).unwrap_or(usize::MAX),
            pending: usize::try_from(stats.pending).unwrap_or(usize::MAX),
            followed_up: usize::try_from(stats.followed_up).unwrap_or(usize::MAX),
            interviews: usize::try_from(stats.interviews).unwrap_or(usize::MAX),
            rejected: usize::try_from(stats.rejected).unwrap_or(usize::MAX),
        }
    }
    /// Agrège les statuts d'une collection de candidatures.
    #[must_use]
    pub fn from_candidates<'a>(candidates: impl IntoIterator<Item = &'a Candidature>) -> Self {
        let mut counts = Self::default();
        for candidate in candidates {
            counts.total += 1;
            match candidate.statut {
                StatutCandidature::EnAttente => counts.pending += 1,
                StatutCandidature::Relancee => counts.followed_up += 1,
                StatutCandidature::Entretien => counts.interviews += 1,
                StatutCandidature::Refus => counts.rejected += 1,
            }
        }
        counts
    }

    /// Candidatures encore ouvertes, hors refus.
    #[must_use]
    pub const fn active(self) -> usize {
        self.total - self.rejected
    }

    /// Part des candidatures ayant reçu une réponse, quelle qu'elle soit.
    #[must_use]
    pub fn response_rate(self) -> usize {
        crate::ui::format::percent(self.interviews + self.rejected, self.total)
    }

    /// Part des candidatures ayant abouti à un entretien.
    #[must_use]
    pub fn conversion_rate(self) -> usize {
        crate::ui::format::percent(self.interviews, self.total)
    }
}

/// Synthèse de l'activité des fournisseurs IA.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LlmSummary {
    /// Nombre total d'appels enregistrés.
    pub total: usize,
    /// Appels terminés avec succès.
    pub successes: usize,
    /// Latence moyenne, en millisecondes.
    pub average_latency_ms: u64,
    /// Répartition par fournisseur, triée par nom.
    pub by_provider: Vec<(String, usize)>,
}

impl LlmSummary {
    /// Agrège l'historique des appels IA.
    #[must_use]
    pub fn from_calls(calls: &[AppelLlm]) -> Self {
        let total = calls.len();
        let successes = calls.iter().filter(|call| call.succes).count();
        let average_latency_ms = calls
            .iter()
            .map(|call| call.latence_ms)
            .sum::<u64>()
            .checked_div(u64::try_from(total).unwrap_or_default())
            .unwrap_or_default();
        let mut providers = std::collections::BTreeMap::<String, usize>::new();
        for call in calls {
            *providers.entry(call.provider.clone()).or_default() += 1;
        }
        Self {
            total,
            successes,
            average_latency_ms,
            by_provider: providers.into_iter().collect(),
        }
    }

    /// Taux de succès des appels IA.
    #[must_use]
    pub fn success_rate(&self) -> usize {
        crate::ui::format::percent(self.successes, self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::{LlmSummary, PipelineCounts};
    use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
    use crate::modules::metriques::model::{AppelLlm, OperationLlm};

    fn candidature(statut: StatutCandidature) -> Candidature {
        Candidature {
            id: uuid::Uuid::new_v4(),
            poste: "Développeur".into(),
            entreprise_id: uuid::Uuid::new_v4(),
            entreprise_nom: Some("Agrial".into()),
            contact_id: None,
            type_contrat: TypeContrat::Cdi,
            statut,
            date_envoi: "2026-08-01".into(),
            lien_offre: None,
            notes: None,
            created_at: "2026-08-01".into(),
            updated_at: "2026-08-01".into(),
        }
    }

    fn appel(provider: &str, succes: bool, latence_ms: u64) -> AppelLlm {
        AppelLlm {
            operation: OperationLlm::ParseOffer,
            provider: provider.into(),
            modele: "test".into(),
            latence_ms,
            succes,
            cree_le: "2026-08-01".into(),
        }
    }

    #[test]
    fn le_pipeline_agrege_chaque_statut() {
        let candidates = vec![
            candidature(StatutCandidature::EnAttente),
            candidature(StatutCandidature::EnAttente),
            candidature(StatutCandidature::Relancee),
            candidature(StatutCandidature::Entretien),
            candidature(StatutCandidature::Refus),
        ];
        let counts = PipelineCounts::from_candidates(&candidates);
        assert_eq!(counts.total, 5);
        assert_eq!(counts.pending, 2);
        assert_eq!(counts.followed_up, 1);
        assert_eq!(counts.interviews, 1);
        assert_eq!(counts.rejected, 1);
        assert_eq!(counts.active(), 4);
    }

    #[test]
    fn les_taux_sont_calcules_sans_division_par_zero() {
        let empty = PipelineCounts::from_candidates(std::iter::empty());
        assert_eq!(empty.response_rate(), 0);
        assert_eq!(empty.conversion_rate(), 0);
        assert_eq!(empty.active(), 0);
    }

    #[test]
    fn le_taux_de_reponse_compte_entretiens_et_refus() {
        let candidates = vec![
            candidature(StatutCandidature::EnAttente),
            candidature(StatutCandidature::Entretien),
            candidature(StatutCandidature::Refus),
            candidature(StatutCandidature::Refus),
        ];
        let counts = PipelineCounts::from_candidates(&candidates);
        assert_eq!(counts.response_rate(), 75);
        assert_eq!(counts.conversion_rate(), 25);
    }

    #[test]
    fn la_synthese_ia_agrege_latence_et_fournisseurs() {
        let calls = vec![
            appel("ollama", true, 100),
            appel("ollama", false, 300),
            appel("claude", true, 200),
        ];
        let summary = LlmSummary::from_calls(&calls);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.successes, 2);
        assert_eq!(summary.average_latency_ms, 200);
        assert_eq!(summary.success_rate(), 66);
        assert_eq!(
            summary.by_provider,
            vec![("claude".to_owned(), 1), ("ollama".to_owned(), 2)]
        );
    }

    #[test]
    fn la_synthese_ia_vide_reste_neutre() {
        let summary = LlmSummary::from_calls(&[]);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.average_latency_ms, 0);
        assert_eq!(summary.success_rate(), 0);
        assert!(summary.by_provider.is_empty());
    }
}
