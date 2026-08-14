//! Cas de test isolé.

use super::*;

#[test]
fn le_taux_d_entretien_est_un_pourcentage_arrondi_sans_division_par_zero() {
    let counts = PipelineCounts {
        total: 10,
        interviews: 3,
        ..PipelineCounts::default()
    };
    assert_eq!(interview_rate(counts.total, counts.interviews), 30);
    assert_eq!(interview_rate(0, 0), 0);
    let complet = PipelineCounts {
        total: 4,
        interviews: 4,
        ..PipelineCounts::default()
    };
    assert_eq!(interview_rate(complet.total, complet.interviews), 100);
    assert_eq!(interview_rate(4, 8), 100);
}
