//! Inférence réelle du runtime Mistral Local, sous grammaire JSON.
//!
//! Ce scénario est le seul capable d'attraper les défauts d'usage de llama.cpp : ceux-ci se
//! manifestent par un `GGML_ASSERT` qui appelle `abort()`, jamais par une `Err`. Aucun test
//! unitaire ne peut les observer — le processus meurt avant toute assertion Rust.
//!
//! | Variable | Rôle | Défaut |
//! | --- | --- | --- |
//! | `CANDILOG_E2E_LOCAL_AI` | active le scénario | absent → ignoré |
//! | `CANDILOG_E2E_LOCAL_AI_MODEL` | chemin du `.gguf` | modèle du dossier de développement |
//!
//! ```bash
//! CANDILOG_E2E_LOCAL_AI=1 cargo test --manifest-path src-tauri/Cargo.toml --test e2e_local_ai -- --nocapture
//! ```

use candilog_lib::features::ai::domain::{LocalAiBackend, LocalModelId, ModelRegistry};
use candilog_lib::features::ai::infrastructure::{MistralLocalRuntime, RuntimeRequest};
use std::path::PathBuf;

fn modele() -> Option<(PathBuf, String)> {
    if std::env::var_os("CANDILOG_E2E_LOCAL_AI").is_none() {
        eprintln!("CANDILOG_E2E_LOCAL_AI absent : scénario ignoré.");
        return None;
    }
    let definition = ModelRegistry::get(LocalModelId::Ministral3Light)?;
    let chemin = std::env::var_os("CANDILOG_E2E_LOCAL_AI_MODEL").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".candilog-dev/ai/models")
                .join(&definition.local_filename)
        },
        PathBuf::from,
    );
    if !chemin.is_file() {
        eprintln!("modèle absent ({}) : scénario ignoré.", chemin.display());
        return None;
    }
    Some((chemin, definition.sha256))
}

/// Une génération sous grammaire JSON doit rendre un objet, sans abandonner le processus.
///
/// `llama_sampler_sample` accepte déjà le jeton qu'il choisit. L'accepter une seconde fois
/// faisait avancer la grammaire deux fois par jeton : ses piles d'analyse se vidaient, et
/// l'appel suivant butait sur `GGML_ASSERT(!stacks.empty())`.
#[test]
fn une_generation_json_aboutit_sans_abandonner_le_processus() {
    let Some((chemin, sha256)) = modele() else {
        return;
    };
    let runtime = MistralLocalRuntime::new();

    let sortie = runtime
        .generate(RuntimeRequest {
            model_id: LocalModelId::Ministral3Light,
            path: &chemin,
            expected_sha256: &sha256,
            backend: LocalAiBackend::Cpu,
            system: "Tu réponds uniquement par un objet JSON.",
            prompt: "Donne un objet JSON avec les clés \"ville\" et \"pays\" pour Paris.",
            temperature: 0.1,
            json: true,
            max_output_tokens: Some(64),
        })
        .expect("la génération sous grammaire JSON doit aboutir");

    eprintln!("texte généré : {}", sortie.text);
    let valeur: serde_json::Value =
        serde_json::from_str(sortie.text.trim()).expect("la grammaire impose un JSON valide");
    assert!(valeur.is_object() || valeur.is_array(), "JSON attendu");
    assert!(sortie.generated_tokens > 0, "aucun jeton généré");
}

/// La génération en texte libre emprunte la même boucle d'échantillonnage : retirer le
/// second `accept` ne doit pas l'avoir privée de ses jetons.
#[test]
fn une_generation_en_texte_libre_produit_du_contenu() {
    let Some((chemin, sha256)) = modele() else {
        return;
    };
    let runtime = MistralLocalRuntime::new();

    let sortie = runtime
        .generate(RuntimeRequest {
            model_id: LocalModelId::Ministral3Light,
            path: &chemin,
            expected_sha256: &sha256,
            backend: LocalAiBackend::Cpu,
            system: "Tu réponds en français, en une phrase.",
            prompt: "Cite une qualité utile en recherche d'emploi.",
            temperature: 0.2,
            json: false,
            max_output_tokens: Some(48),
        })
        .expect("la génération en texte libre doit aboutir");

    eprintln!("texte généré : {}", sortie.text);
    assert!(!sortie.text.trim().is_empty(), "réponse vide");
    assert!(sortie.generated_tokens > 0, "aucun jeton généré");
    assert!(sortie.prompt_tokens > 0, "invite non décodée");
}

/// Annuler doit réellement arrêter les cœurs, pas seulement rendre la main à l'interface.
///
/// La génération s'exécute dans un `spawn_blocking` que Tokio ne sait pas interrompre :
/// seul le jeton lu par la boucle l'arrête. Sans lui, ce scénario mettrait plusieurs
/// minutes — le temps que les 3 072 jetons du plafond soient produits.
#[test]
fn une_inference_locale_s_interrompt_a_la_demande() {
    let Some((chemin, sha256)) = modele() else {
        return;
    };
    let runtime = std::sync::Arc::new(MistralLocalRuntime::new());

    // Amorçage : le modèle reste chargé, la génération suivante démarre sans délai.
    let requete = |max: Option<usize>| RuntimeRequest {
        model_id: LocalModelId::Ministral3Light,
        path: &chemin,
        expected_sha256: &sha256,
        backend: LocalAiBackend::Cpu,
        system: "Tu réponds en français.",
        prompt: "Raconte en détail une longue journée de recherche d'emploi.",
        temperature: 0.2,
        json: false,
        max_output_tokens: max,
    };
    runtime
        .generate(requete(Some(1)))
        .expect("amorçage du modèle");

    let annuleur = std::sync::Arc::clone(&runtime);
    let main = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        annuleur.cancel_inference();
    });

    let debut = std::time::Instant::now();
    let resultat = runtime.generate(requete(None));
    let ecoule = debut.elapsed();
    main.join().expect("fil d'annulation");

    assert!(
        matches!(
            resultat,
            Err(candilog_lib::core::errors::AppError::Cancelled)
        ),
        "l'inférence devait être annulée, obtenu : {resultat:?}"
    );
    assert!(
        ecoule < std::time::Duration::from_secs(60),
        "l'annulation a mis {ecoule:?} : la boucle ne la consulte pas"
    );
}
