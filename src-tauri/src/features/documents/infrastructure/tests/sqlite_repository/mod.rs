use super::*;
use crate::core::database::{open_pool, run_local_migrations};

fn pool() -> SqlitePool {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    pool
}

#[test]
fn cv_est_restitue_avec_son_json_et_son_resume() {
    let repo = SqliteCvRepository::new(pool());
    let saved = repo
        .enregistrer(&NouveauCv {
            nom: "CV Produit".into(),
            contenu: serde_json::json!({"cv":{"summary":"Bonjour"}}),
        })
        .unwrap();
    assert_eq!(repo.lister().unwrap()[0].nom, "CV Produit");
    assert_eq!(
        repo.obtenir(saved.id).unwrap().contenu["cv"]["summary"],
        "Bonjour"
    );
}

#[test]
fn lettre_est_enregistree_et_supprimee() {
    let repo = SqliteLettreRepository::new(pool());
    let saved = repo
        .enregistrer(&NouvelleLettre {
            nom: "Lettre Nova".into(),
            entreprise: Some("Nova".into()),
            poste: Some("Designer".into()),
            ton: "formal".into(),
            longueur: "medium".into(),
            contenu: "Madame, Monsieur…".into(),
        })
        .unwrap();
    assert_eq!(repo.lister().unwrap(), vec![saved.clone()]);
    repo.supprimer(saved.id).unwrap();
    assert!(repo.lister().unwrap().is_empty());
}
