//! Harness de reproduction du flux « Améliorer le CV » (générateur de CV).
//!
//! Copie la base de dev, puis exécute la même séquence que l'interface :
//! parse_offer (cache) → generate_cv → analyze_ats, en chronométrant chaque étape.

use candilog::modules::ia::service;
use candilog::shared::state::AppState;
use std::path::Path;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

fn copy_dev_db(tmp: &Path) -> std::path::PathBuf {
    let src = Path::new(".candilog-dev/candilog.sqlite");
    let dst = tmp.join("repro.sqlite");
    std::fs::copy(src, &dst).expect("copie de la base de dev");
    dst
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn reproduire_ameliorer_le_cv() {
    let tmp = tempfile::tempdir().expect("dossier temporaire");
    let db = copy_dev_db(tmp.path());
    let state = AppState::persistent(&db).expect("ouverture de la base");

    let settings = state.secure_settings_async().await.expect("paramètres");
    println!(
        "fournisseur={:?} mode={:?} modèle={} endpoint={:?}",
        settings.llm.provider,
        settings.llm.resolved_mode(),
        settings.llm.model,
        settings.llm.endpoint
    );

    let offer = serde_json::json!({
        "title": "ingénieur système linux H/F",
        "skills": ["Installation", "configuration", "Linux", "open source", "développement", "maintenance", "sauvegarde", "restauration", "Gestion des utilisateurs", "scripts", "automatisation", "correctifs", "mises à jour de sécurité", "sécurisation", "bases de données", "SQL", "NoSQL", "serveurs Web", "Apache", "NGINX", "Haproxy", "virtualisation", "infrastructure"],
        "soft_skills": ["Compétences en communication et en présentation", "Capacité à travailler en équipe et dans un environnement collaboratif"],
        "experience": "2 ans",
        "keywords": ["ingénieur système linux", "cybersécurité", "Linux", "open source", "SQL", "NoSQL", "Apache", "NGINX", "Haproxy", "virtualisation", "sauvegarde", "restauration", "scripts", "automatisation", "sécurité", "infrastructure", "IT", "OT", "Cloud", "PASSI RGS", "LPM", "Squad"]
    })
    .to_string();

    // Étape 1 — analyse de l'offre (sert du cache si présent).
    let started = Instant::now();
    let analysis = service::analyze_offer(&state, offer)
        .await
        .expect("analyse de l'offre");
    println!("parse_offer OK en {:.1} s", started.elapsed().as_secs_f32());
    println!("  score total = {}", analysis.score.total);

    // Étape 2 — génération du CV puis analyse ATS (séquence complète de « Améliorer le CV »).
    let started = Instant::now();
    let generation = service::generate_cv(
        &state,
        analysis.parsed,
        analysis.score,
        CancellationToken::new(),
    )
    .await
    .expect("génération du CV + analyse ATS");
    println!(
        "generate_cv + analyze_ats OK en {:.1} s",
        started.elapsed().as_secs_f32()
    );
    println!("  summary = {}", &generation.cv.summary);
    println!("  expériences = {}", generation.cv.experiences.len());
    println!("  compétences = {}", generation.cv.skills.len());
    println!("  score ATS = {}", generation.analysis.score);
    println!(
        "  recommandations = {}",
        generation.analysis.recommandations.len()
    );
    for (index, rec) in generation.analysis.recommandations.iter().enumerate() {
        println!(
            "  rec[{index}] section={} impact={}",
            rec.section, rec.impact
        );
    }
}
