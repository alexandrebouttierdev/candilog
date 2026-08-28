use super::*;
use crate::core::database::{open_pool, run_local_migrations};

fn pool() -> SqlitePool {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    pool
}

#[test]
fn cv_est_restitue_avec_son_json_et_son_resume() {
    let repo = SqliteResumeRepository::new(pool());
    let saved = repo
        .save(&NewResume {
            name: "CV Produit".into(),
            content: serde_json::json!({"cv":{"summary":"Bonjour"}}),
        })
        .unwrap();
    assert_eq!(repo.list().unwrap()[0].name, "CV Produit");
    assert_eq!(
        repo.get(saved.id).unwrap().content["cv"]["summary"],
        "Bonjour"
    );
}

#[test]
fn lettre_est_enregistree_et_supprimee() {
    let repo = SqliteCoverLetterRepository::new(pool());
    let saved = repo
        .save(&NewCoverLetter {
            name: "Lettre Nova".into(),
            company: Some("Nova".into()),
            job_title: Some("Designer".into()),
            tone: "formal".into(),
            length: "medium".into(),
            content: "Madame, Monsieur…".into(),
        })
        .unwrap();
    assert_eq!(repo.list().unwrap(), vec![saved.clone()]);
    repo.delete(saved.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}
