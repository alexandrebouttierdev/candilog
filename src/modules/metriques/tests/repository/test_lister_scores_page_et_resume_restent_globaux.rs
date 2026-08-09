//! Cas de test isolé.

use super::*;

#[test]
fn test_lister_scores_page_et_resume_restent_globaux() {
    let r = repo();
    for (index, (score, origine)) in [
        (40, OrigineScore::Genere),
        (60, OrigineScore::Genere),
        (80, OrigineScore::Importe),
        (90, OrigineScore::Importe),
    ]
    .into_iter()
    .enumerate()
    {
        r.enregistrer_score(&ScoreAts {
            score,
            origine,
            cree_le: format!("2026-07-16T10:{index:02}:00Z"),
        })
        .unwrap();
    }
    let page = r.lister_scores_page(2, 2).unwrap();
    let resume = r.resumer_scores().unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.total, 4);
    assert_eq!(page.total_pages, 2);
    assert_eq!(resume.nombre, 4);
    assert_eq!(resume.moyenne, 68);
    assert_eq!(
        (
            resume.faibles,
            resume.partiels,
            resume.bons,
            resume.excellents
        ),
        (1, 1, 1, 1)
    );
    assert_eq!((resume.generes_nombre, resume.generes_moyenne), (2, 50));
    assert_eq!((resume.importes_nombre, resume.importes_moyenne), (2, 85));
}
