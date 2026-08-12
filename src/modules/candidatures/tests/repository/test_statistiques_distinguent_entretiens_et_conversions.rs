use super::*;

#[test]
fn test_statistiques_distinguent_entretiens_et_conversions() {
    let repo = repo();
    let company = entreprise(&repo, "ACME");
    let candidature = repo.create(&entree(company, "Développeur Rust")).unwrap();

    repo.update_statut(candidature.id, StatutCandidature::Entretien)
        .unwrap();
    repo.update_statut(candidature.id, StatutCandidature::Refus)
        .unwrap();

    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    for jour in ["2026-02-10T09:00:00Z", "2026-02-17T14:00:00Z"] {
        conn.execute(
            "INSERT INTO entretiens (
                id, candidature_id, date_entretien, type, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 'Visio', ?3, ?3)",
            rusqlite::params![
                uuid::Uuid::new_v4().to_string(),
                candidature.id.to_string(),
                jour,
            ],
        )
        .unwrap();
    }

    let stats = repo.stats().unwrap();
    assert_eq!(stats.interviews, 0, "le statut final est Refus");
    assert_eq!(stats.interviews_total, 2, "chaque entretien est compté");
    assert_eq!(
        stats.converted_candidates, 1,
        "une candidature convertie ne doit être comptée qu'une fois"
    );
}
