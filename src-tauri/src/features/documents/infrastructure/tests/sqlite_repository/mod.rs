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
            recipient: Some("Service recrutement".into()),
            recipient_address: Some("12 rue de la Monnaie, 35000 Rennes".into()),
            job_reference: Some("FS-2026-114".into()),
            tone: "formal".into(),
            length: "medium".into(),
            content: "Madame, Monsieur…".into(),
        })
        .unwrap();
    assert_eq!(repo.list().unwrap(), vec![saved.clone()]);
    assert_eq!(saved.recipient.as_deref(), Some("Service recrutement"));
    assert_eq!(saved.job_reference.as_deref(), Some("FS-2026-114"));
    repo.delete(saved.id).unwrap();
    assert!(repo.list().unwrap().is_empty());
}

#[test]
fn cv_pagination_filtre_avant_la_limite() {
    let repo = SqliteResumeRepository::new(pool());
    for index in 0..25 {
        repo.save(&NewResume {
            name: if index % 2 == 0 {
                format!("CV Cible {index:02}")
            } else {
                format!("CV Autre {index:02}")
            },
            content: serde_json::json!({}),
        })
        .unwrap();
    }

    let page = repo.list_page(2, 8, "cible").unwrap();

    assert_eq!(page.total, 13);
    assert_eq!(page.items.len(), 5);
    assert!(page.items.iter().all(|item| item.name.contains("Cible")));
}

#[test]
fn lettres_pagination_filtre_avant_la_limite() {
    let repo = SqliteCoverLetterRepository::new(pool());
    for index in 0..25 {
        repo.save(&NewCoverLetter {
            name: if index % 2 == 0 {
                format!("Lettre Cible {index:02}")
            } else {
                format!("Lettre Autre {index:02}")
            },
            company: None,
            job_title: None,
            recipient: None,
            recipient_address: None,
            job_reference: None,
            tone: "formal".into(),
            length: "medium".into(),
            content: "Contenu".into(),
        })
        .unwrap();
    }

    let page = repo.list_page(2, 8, "cible").unwrap();

    assert_eq!(page.total, 13);
    assert_eq!(page.items.len(), 5);
    assert!(page.items.iter().all(|item| item.name.contains("Cible")));
}

/// Les deux bibliothèques comparaient un terme normalisé en Rust à une colonne passée par
/// `lower()` de SQLite, qui laisse les majuscules accentuées intactes : un CV « CV ÉCOLE »
/// restait introuvable par « école » comme par son propre nom.
#[test]
fn la_recherche_des_bibliotheques_ignore_les_accents() {
    let cvs = SqliteResumeRepository::new(pool());
    cvs.save(&NewResume {
        name: "CV ÉCOLE".into(),
        content: serde_json::json!({}),
    })
    .unwrap();
    let lettres = SqliteCoverLetterRepository::new(pool());
    lettres
        .save(&NewCoverLetter {
            name: "Lettre ÉCOLE".into(),
            company: None,
            job_title: None,
            recipient: None,
            recipient_address: None,
            job_reference: None,
            tone: "formal".into(),
            length: "medium".into(),
            content: "Contenu".into(),
        })
        .unwrap();

    for terme in ["école", "ECOLE"] {
        assert_eq!(
            cvs.list_page(1, 8, terme).unwrap().total,
            1,
            "CV : recherche « {terme} » sans résultat"
        );
        assert_eq!(
            lettres.list_page(1, 8, terme).unwrap().total,
            1,
            "lettres : recherche « {terme} » sans résultat"
        );
    }
}
