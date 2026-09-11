//! Runner local du baseline d'import CV.
//! Appelle AiService::import_profile sur une base SQLite jetable.
//! N'applique jamais l'import et ne journalise aucune donnee personnelle.
//! Voir `cv_import_baseline.md` a cote de ce fichier.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use candilog_lib::core::database::{open_pool, run_local_migrations};
use candilog_lib::core::errors::AppError;
use candilog_lib::features::ai::application::{
    AiService, ManagedOllamaPaths, ManagedOllamaService,
};
use candilog_lib::features::ai::domain::{ProfileImportRequest, ProviderKind};
use candilog_lib::features::ai::infrastructure::{extract_pdf, load_config};
use candilog_lib::features::profile::domain::ImportProfilePreview;
use candilog_lib::features::settings::domain::{AppSettings, SettingsRepository};
use candilog_lib::features::settings::infrastructure::SqliteSettingsRepository;

const DEFAULT_MODEL: &str = "LiquidAI/lfm2.5-1.2b-instruct:latest";
const DEFAULT_ENDPOINT: &str = "http://localhost:11434";
const DEFAULT_TEMPERATURE: f32 = 0.7;
const ANALYSIS_CHARS: usize = 12_000;

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
}

fn temperature_from_env() -> f32 {
    let raw = std::env::var("CANDILOG_CV_TEMPERATURE").unwrap_or_default();
    if raw.trim().is_empty() {
        return DEFAULT_TEMPERATURE;
    }
    raw.trim().parse().unwrap_or_else(|_| {
        eprintln!("CANDILOG_CV_TEMPERATURE invalide: {raw}");
        std::process::exit(1);
    })
}

fn require_dir(name: &str) -> PathBuf {
    let raw = std::env::var(name).unwrap_or_default();
    if raw.is_empty() {
        eprintln!("{name} manquant : dossier local, hors dépôt");
        std::process::exit(1);
    }
    PathBuf::from(raw)
}

#[tokio::main]
async fn main() {
    let cv_dir = require_dir("CANDILOG_CV_DIR");
    let bench = std::env::var("CANDILOG_CV_BENCH")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| cv_dir.join(".benchmark"));
    let out_name = std::env::args().nth(1).unwrap_or_else(|| "run".to_owned());
    if out_name.is_empty()
        || out_name.contains('/')
        || out_name.contains("..")
        || out_name.starts_with('.')
    {
        eprintln!("nom de sortie invalide: {out_name}");
        std::process::exit(1);
    }
    let out_dir = bench.join(&out_name);
    let raw_dir = out_dir.join("raw");
    if let Err(error) = std::fs::create_dir_all(&raw_dir) {
        eprintln!("output directory: {error}");
        std::process::exit(1);
    }
    let _ = std::fs::set_permissions(
        &raw_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    );
    let iteration = 0_u32;
    let label = out_name.clone();

    let mapping = match load_mapping(&bench.join("mapping.json")) {
        Ok(mapping) => mapping,
        Err(error) => {
            eprintln!("mapping: {error}");
            std::process::exit(1);
        }
    };

    let db_path = bench.join("bench.sqlite");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(bench.join("bench.sqlite-wal"));
    let _ = std::fs::remove_file(bench.join("bench.sqlite-shm"));
    let models_dir = bench.join("local-ai-empty");
    let pool = match open_pool(Some(&db_path)) {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("open bench db: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = run_local_migrations(&pool) {
        eprintln!("migrations: {error}");
        std::process::exit(1);
    }
    let model = env_or("CANDILOG_CV_MODEL", DEFAULT_MODEL);
    let endpoint = env_or("CANDILOG_OLLAMA_URL", DEFAULT_ENDPOINT);
    let requested_temperature = temperature_from_env();
    let mut settings = AppSettings::default();
    settings.llm.provider = ProviderKind::Ollama;
    settings.llm.model = model.clone();
    settings.llm.endpoint = Some(endpoint.clone());
    settings.llm.api_key = None;
    settings.llm.temperature = requested_temperature;
    if let Err(error) = SqliteSettingsRepository::new(pool.clone()).upsert(&settings) {
        eprintln!("settings upsert: {error}");
        std::process::exit(1);
    }
    let loaded = match load_config(&pool) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("load_config: {error}");
            std::process::exit(1);
        }
    };
    if loaded.provider != ProviderKind::Ollama
        || loaded.model != model
        || loaded.endpoint_effectif() != endpoint
    {
        eprintln!("bench settings do not match the requested Ollama model");
        std::process::exit(1);
    }
    let temperature = loaded.temperature;
    if (temperature - requested_temperature).abs() > 0.001 {
        eprintln!(
            "refusing run: loaded temperature is {temperature}, expected {requested_temperature}"
        );
        std::process::exit(1);
    }
    let managed = Arc::new(ManagedOllamaService::new(
        pool.clone(),
        ManagedOllamaPaths {
            runtime_root: models_dir.join("managed/runtime"),
            models_dir: models_dir.join("managed/models"),
            downloads_dir: models_dir.join("managed/downloads"),
        },
    ));
    let service = AiService::new(pool, managed);

    let results_path = out_dir.join("results.json");
    let journal_path = out_dir.join("journal.md");
    let progress_path = out_dir.join("progress.log");
    let mut items = load_existing_items(&results_path);
    let done: std::collections::BTreeSet<String> = items
        .iter()
        .filter_map(|item| {
            item.get("cv_id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
        })
        .collect();

    let state = Arc::new(Mutex::new(RunState {
        mapping_len: mapping.len(),
        temperature,
        model: model.clone(),
        endpoint: endpoint.clone(),
        items: items.clone(),
        current: None,
        finished: false,
    }));
    spawn_heartbeat(Arc::clone(&state), journal_path.clone());
    if let Ok(guard) = state.lock() {
        rewrite_journal(&journal_path, &guard);
    }

    eprintln!(
        "{label} start n={} model={} temperature={} provider=ollama",
        mapping.len(),
        model,
        temperature
    );

    for (cv_id, filename) in &mapping {
        if done.contains(cv_id) {
            eprintln!("{cv_id} skip already recorded");
            continue;
        }
        if let Ok(mut guard) = state.lock() {
            guard.current = Some(Current {
                cv_id: cv_id.clone(),
                started: Instant::now(),
            });
            rewrite_journal(&journal_path, &guard);
        }
        eprintln!("{cv_id} start");

        let path = cv_dir.join(filename);
        let wall = Instant::now();
        let progress = Arc::new(Mutex::new(Progress::default()));
        let progress_cb = Arc::clone(&progress);
        let result = service
            .import_profile(
                ProfileImportRequest {
                    generation_id: format!("{out_name}-{cv_id}"),
                    method: candilog_lib::features::ai::domain::CvAnalysisMethod::Text,
                },
                path.clone(),
                move |event| {
                    let Ok(mut slot) = progress_cb.lock() else {
                        return;
                    };
                    if let Some(tokens) = event.tokens_used {
                        slot.tokens = Some(tokens);
                    }
                    if let Some(call) = parse_llm_call(&event.message) {
                        slot.llm_calls.push(call);
                    }
                    if let Some(label) = safe_progress_label(&event.message) {
                        slot.last_label = Some(label);
                    } else if let Some(step) = event.step.as_deref() {
                        if let Some(label) = safe_progress_label(step) {
                            slot.last_label = Some(label);
                        }
                    }
                },
            )
            .await;
        let duration_ms = wall.elapsed().as_millis();
        let progress = progress.lock().map(|slot| slot.clone()).unwrap_or_default();

        let item = match result {
            Ok(execution) => {
                let heuristic = match extract_pdf(path).await {
                    Ok(text) => Some(score_heuristic(&text, &execution.output.preview)),
                    Err(_) => None,
                };
                if let Err(error) = write_raw(&raw_dir, cv_id, &execution.output.preview) {
                    eprintln!("{cv_id} raw write failed: {error}");
                }
                item_ok(
                    cv_id,
                    duration_ms,
                    execution.elapsed_ms,
                    execution.tokens_used.or(progress.tokens),
                    &execution.output.preview,
                    heuristic,
                    &progress.llm_calls,
                )
            }
            Err(error) => item_err(
                cv_id,
                duration_ms,
                progress.tokens,
                &error,
                progress.last_label,
                &progress.llm_calls,
            ),
        };

        let status = if item.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            "ok"
        } else {
            "fail"
        };
        let category = item
            .get("error_category")
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        eprintln!("{cv_id} {status} duration_ms={duration_ms} category={category}");
        append_progress(&progress_path, &item);

        items.push(item);
        if let Err(error) = save_results(
            &results_path,
            &model,
            &endpoint,
            temperature,
            iteration,
            &label,
            &items,
        ) {
            eprintln!("results write: {error}");
            std::process::exit(1);
        }
        if let Ok(mut guard) = state.lock() {
            guard.items = items.clone();
            guard.current = None;
            rewrite_journal(&journal_path, &guard);
        }
    }

    if let Ok(mut guard) = state.lock() {
        guard.current = None;
        guard.finished = true;
        rewrite_journal(&journal_path, &guard);
    }
    eprintln!("{label} finished n={}", items.len());
}

#[derive(Clone)]
struct Current {
    cv_id: String,
    started: Instant,
}

struct RunState {
    mapping_len: usize,
    temperature: f32,
    model: String,
    endpoint: String,
    items: Vec<serde_json::Value>,
    current: Option<Current>,
    finished: bool,
}

#[derive(Clone, Default)]
struct Progress {
    tokens: Option<u32>,
    last_label: Option<String>,
    llm_calls: Vec<serde_json::Value>,
}

fn load_mapping(path: &Path) -> Result<Vec<(String, String)>, String> {
    let raw = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    let object = value
        .get("mapping")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "mapping object missing".to_owned())?;
    let mut pairs: Vec<(String, String)> = object
        .iter()
        .filter_map(|(id, file)| {
            let name = file.as_str()?.to_owned();
            if name.contains('/') || name.contains('\\') || !name.ends_with(".pdf") {
                return None;
            }
            Some((id.clone(), name))
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    if pairs.is_empty() {
        return Err("no mapped pdfs".into());
    }
    Ok(pairs)
}

fn load_existing_items(path: &Path) -> Vec<serde_json::Value> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    value
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn save_results(
    path: &Path,
    model: &str,
    endpoint: &str,
    temperature: f32,
    iteration: u32,
    label: &str,
    items: &[serde_json::Value],
) -> std::io::Result<()> {
    let stats = summarize(items);
    let doc = serde_json::json!({
        "iteration": iteration,
        "label": label,
        "pipeline": "AiService::import_profile",
        "provider": "ollama",
        "model": model,
        "endpoint": endpoint,
        "temperature": temperature,
        "temperature_note": "App default written on the throwaway settings row and sent as Ollama options.temperature. Model card default 0.1 is overridden by that request option.",
        "http_timeout_s": 1800,
        "analysis_char_limit": ANALYSIS_CHARS,
        "score_kind": "heuristic_baseline_not_human_ground_truth",
        "pdf_count_recorded": items.len(),
        "summary": {
            "n": stats.n,
            "ok": stats.ok,
            "fail": stats.fail,
            "mean_duration_ms": stats.mean_ms,
            "median_duration_ms": stats.median_ms,
            "mean_heuristic_score": stats.mean_score,
            "median_heuristic_score": stats.median_score,
            "error_categories": stats.categories,
        },
        "items": items,
    });
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&doc).map_err(io_err)?)?;
    std::fs::rename(tmp, path)
}

fn parse_llm_call(message: &str) -> Option<serde_json::Value> {
    let mut parts = message.split_whitespace();
    if parts.next()? != "llm_call" {
        return None;
    }
    let kind = parts.next()?;
    let ms = parts.next()?.parse::<u64>().ok()?;
    let status = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    if !matches!(kind, "extraction" | "experiences") || !matches!(status, "ok" | "fail") {
        return None;
    }
    Some(serde_json::json!({
        "kind": kind,
        "ms": ms,
        "status": status,
    }))
}

fn item_ok(
    cv_id: &str,
    duration_ms: u128,
    service_elapsed_ms: u32,
    tokens: Option<u32>,
    preview: &ImportProfilePreview,
    heuristic: Option<serde_json::Value>,
    llm_calls: &[serde_json::Value],
) -> serde_json::Value {
    serde_json::json!({
        "cv_id": cv_id,
        "ok": true,
        "duration_ms": duration_ms,
        "service_elapsed_ms": service_elapsed_ms,
        "tokens": tokens,
        "llm_calls": llm_calls,
        "llm_call_count": llm_calls.len(),
        "counts": {
            "identity": preview.counts.identity,
            "experience": preview.experiences.len(),
            "formation": preview.education.len(),
            "skill": preview.skills.len(),
            "language": preview.languages.len(),
            "project": preview.projects.len(),
            "certification": preview.certifications.len(),
        },
        "presence": {
            "first_name": identity_present(preview, "first_name"),
            "name": identity_present(preview, "name"),
            "email": identity_present(preview, "email"),
            "phone": identity_present(preview, "phone"),
            "title": identity_present(preview, "title"),
            "city": identity_present(preview, "city"),
        },
        "heuristic": heuristic,
        "error_code": null,
        "error_category": null,
    })
}

fn item_err(
    cv_id: &str,
    duration_ms: u128,
    tokens: Option<u32>,
    error: &AppError,
    last_label: Option<String>,
    llm_calls: &[serde_json::Value],
) -> serde_json::Value {
    let (code, category) = error_meta(error);
    serde_json::json!({
        "cv_id": cv_id,
        "ok": false,
        "duration_ms": duration_ms,
        "service_elapsed_ms": null,
        "tokens": tokens,
        "llm_calls": llm_calls,
        "llm_call_count": llm_calls.len(),
        "counts": {
            "identity": 0,
            "experience": 0,
            "formation": 0,
            "skill": 0,
            "language": 0,
            "project": 0,
            "certification": 0,
        },
        "presence": {
            "first_name": false,
            "name": false,
            "email": false,
            "phone": false,
            "title": false,
            "city": false,
        },
        "heuristic": null,
        "error_code": code,
        "error_category": category,
        "last_step": last_label,
    })
}

fn error_meta(error: &AppError) -> (&'static str, &'static str) {
    let category = match error {
        AppError::Validation(_) => "validation",
        AppError::NotFound(_) => "not_found",
        AppError::Database(_) => "database",
        AppError::IncompatibleData(_) => "incompatible_data",
        AppError::Http(message) => http_category(message),
        AppError::Serialization(_) => "serialization",
        AppError::Provider(message) => {
            if message.contains("aucune information") {
                "empty_extraction"
            } else if message.contains("Configurez un fournisseur") {
                "not_configured"
            } else {
                "provider"
            }
        }
        AppError::Cancelled => "cancelled",
    };
    (error.code(), category)
}

fn http_category(message: &str) -> &'static str {
    let lower = message.to_lowercase();
    if lower.contains("timeout") || lower.contains("timed out") || lower.contains("delai") {
        "http_timeout"
    } else if lower.contains("connect") {
        "http_connect"
    } else {
        "http"
    }
}

fn identity_value(preview: &ImportProfilePreview, id: &str) -> Option<String> {
    preview
        .identity
        .iter()
        .find(|item| item.id == id)
        .map(|item| item.proposed.clone())
        .filter(|value| !value.trim().is_empty())
}

fn identity_present(preview: &ImportProfilePreview, id: &str) -> bool {
    identity_value(preview, id).is_some()
}

fn write_raw(dir: &Path, cv_id: &str, preview: &ImportProfilePreview) -> std::io::Result<()> {
    if !cv_id.starts_with("CV-") || cv_id.contains('/') || cv_id.contains('.') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bad cv id",
        ));
    }
    let path = dir.join(format!("{cv_id}.json"));
    let body = serde_json::to_vec_pretty(preview).map_err(io_err)?;
    std::fs::write(&path, body)?;
    let _ = std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600));
    Ok(())
}

fn score_heuristic(full_text: &str, preview: &ImportProfilePreview) -> serde_json::Value {
    let analysis = truncate_chars(full_text, ANALYSIS_CHARS);
    let source_email = first_email(&analysis);
    let source_phone = first_phone_digits(&analysis);
    let extracted_email = identity_value(preview, "email");
    let extracted_phone = identity_value(preview, "phone");
    let email_grounded = email_matches(
        extracted_email.as_deref(),
        source_email.as_deref(),
        &analysis,
    );
    let phone_grounded = phone_matches(
        extracted_phone.as_deref(),
        source_phone.as_deref(),
        &analysis,
    );
    let date_spans = count_year_ranges(&analysis);
    let diploma_hints = count_diploma_hints(&analysis);
    let language_hints = count_language_hints(&analysis);
    let skill_section = has_skill_section(&analysis);
    let experience = preview.experiences.len();
    let formation = preview.education.len();
    let skills = preview.skills.len();
    let languages = preview.languages.len();

    let email_points = if email_grounded { 15.0 } else { 0.0 };
    let phone_points = if phone_grounded { 10.0 } else { 0.0 };
    let experience_points = proximity_points(experience, date_spans, 30.0);
    let formation_points = proximity_points(formation, diploma_hints, 20.0);
    let skill_points = if skill_section {
        if skills > 0 {
            15.0
        } else {
            0.0
        }
    } else if skills == 0 {
        10.0
    } else {
        12.0
    };
    let language_points = proximity_points(languages, language_hints, 10.0);
    let score = (email_points
        + phone_points
        + experience_points
        + formation_points
        + skill_points
        + language_points)
        .round() as u32;

    serde_json::json!({
        "score": score,
        "label": "heuristic_baseline_not_human_ground_truth",
        "source_has_email": source_email.is_some(),
        "extracted_has_email": extracted_email.is_some(),
        "email_grounded": email_grounded,
        "source_has_phone": source_phone.is_some(),
        "extracted_has_phone": extracted_phone.is_some(),
        "phone_grounded": phone_grounded,
        "source_date_span_count": date_spans,
        "source_diploma_hint_count": diploma_hints,
        "source_language_hint_count": language_hints,
        "source_skill_section": skill_section,
        "source_chars": full_text.chars().count(),
        "analysis_chars": analysis.chars().count(),
        "truncated": full_text.chars().count() > ANALYSIS_CHARS,
        "components": {
            "email": email_points as u32,
            "phone": phone_points as u32,
            "experience": experience_points as u32,
            "formation": formation_points as u32,
            "skill": skill_points as u32,
            "language": language_points as u32,
        }
    })
}

fn proximity_points(extracted: usize, expected: usize, weight: f64) -> f64 {
    if expected == 0 && extracted == 0 {
        return weight * 0.7;
    }
    if expected == 0 {
        return weight * 0.5;
    }
    let max = extracted.max(expected) as f64;
    let min = extracted.min(expected) as f64;
    weight * (min / max)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }
    let mut truncated: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

fn normalize_email(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c: char| matches!(c, '<' | '>' | '(' | ')' | ',' | ';' | '"' | '\''))
        .to_lowercase()
}

fn first_email(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let cleaned = normalize_email(token);
        if is_email(&cleaned) {
            return Some(cleaned);
        }
    }
    None
}

fn is_email(value: &str) -> bool {
    let mut parts = value.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    parts.next().is_none()
        && !local.is_empty()
        && domain.contains('.')
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        && local
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-'))
}

fn email_matches(extracted: Option<&str>, source: Option<&str>, analysis: &str) -> bool {
    match (extracted, source) {
        (Some(found), Some(source)) => {
            let found = normalize_email(found);
            found == normalize_email(source) && analysis.to_lowercase().contains(&found)
        }
        (None, None) => true,
        (Some(found), None) => {
            let found = normalize_email(found);
            found.contains('@') && analysis.to_lowercase().contains(&found)
        }
        (None, Some(_)) => false,
    }
}

fn digits_only(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn first_phone_digits(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() || chars[i] == '+' {
            let start = i;
            let mut digits = String::new();
            let mut consumed = 0usize;
            while i < chars.len() && consumed < 24 {
                let c = chars[i];
                if c.is_ascii_digit() {
                    digits.push(c);
                    consumed += 1;
                    i += 1;
                } else if matches!(c, ' ' | '.' | '-' | '/' | '(' | ')' | '\u{00a0}')
                    && !digits.is_empty()
                {
                    consumed += 1;
                    i += 1;
                } else {
                    break;
                }
            }
            if (10..=13).contains(&digits.len()) {
                return Some(if digits.len() > 10 {
                    digits[digits.len() - 10..].to_owned()
                } else {
                    digits
                });
            }
            if i == start {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    None
}

fn phone_matches(extracted: Option<&str>, source: Option<&str>, analysis: &str) -> bool {
    let source_digits = digits_only(analysis);
    match (extracted, source) {
        (Some(found), Some(source)) => {
            let digits = digits_only(found);
            (digits.len() >= 8 && source_digits.contains(&digits))
                || (source.len() >= 8 && digits.contains(source))
        }
        (None, None) => true,
        (Some(found), None) => {
            let digits = digits_only(found);
            digits.len() >= 8 && source_digits.contains(&digits)
        }
        (None, Some(_)) => false,
    }
}

fn count_year_ranges(text: &str) -> usize {
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 4 <= chars.len() {
        if parse_year(&chars[i..]).is_some() {
            let after = i + 4;
            if looks_like_range(&chars, after) {
                count += 1;
                i = after + 1;
                continue;
            }
        }
        i += 1;
    }
    count
}

fn looks_like_range(chars: &[char], mut index: usize) -> bool {
    let mut skipped = 0usize;
    while index < chars.len() && skipped < 16 {
        let c = chars[index];
        if c.is_whitespace() || matches!(c, '-' | '–' | '—' | '/' | '|' | '·' | '•' | 'à' | ':')
        {
            index += 1;
            skipped += 1;
            continue;
        }
        break;
    }
    if index + 4 <= chars.len() && parse_year(&chars[index..]).is_some() {
        return true;
    }
    let rest: String = chars[index..].iter().take(16).collect();
    const PRESENT: &[&str] = &[
        "aujourd", "présent", "present", "current", "en cours", "now", "actuel",
    ];
    PRESENT.iter().any(|word| rest.starts_with(word))
}

fn parse_year(chars: &[char]) -> Option<u32> {
    if chars.len() < 4 || !chars[..4].iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let year: u32 = chars[..4].iter().collect::<String>().parse().ok()?;
    if (1970..=2035).contains(&year) {
        Some(year)
    } else {
        None
    }
}

fn count_diploma_hints(text: &str) -> usize {
    let lower = text.to_lowercase();
    let padded = format!(
        " {} ",
        lower.replace(['\n', '\t', '/', ',', ';', ':', '(', ')', '[', ']'], " ")
    );
    let tokens: Vec<&str> = padded.split_whitespace().collect();
    const HINTS: &[&str] = &[
        "baccalauréat",
        "baccalaureat",
        "master",
        "licence",
        "bachelor",
        "doctorat",
        "phd",
        "mba",
        "dut",
        "bts",
        "deug",
        "deust",
        "ingénieur",
        "ingenieur",
        "msc",
        "maîtrise",
        "maitrise",
        "diplôme",
        "diplome",
    ];
    let seen = HINTS
        .iter()
        .filter(|hint| {
            tokens
                .iter()
                .any(|token| token.trim_matches(|c: char| !c.is_alphanumeric()) == **hint)
        })
        .count();
    let repeats = tokens
        .iter()
        .filter(|token| {
            let t = token.trim_matches(|c: char| !c.is_alphanumeric());
            matches!(
                t,
                "master"
                    | "licence"
                    | "bachelor"
                    | "bts"
                    | "dut"
                    | "mba"
                    | "doctorat"
                    | "phd"
                    | "ingénieur"
                    | "ingenieur"
                    | "msc"
            )
        })
        .count();
    repeats.max(seen.min(6))
}

fn count_language_hints(text: &str) -> usize {
    let tokens: std::collections::BTreeSet<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    const GROUPS: &[&[&str]] = &[
        &["francais", "français", "french"],
        &["anglais", "english"],
        &["espagnol", "spanish", "espanol"],
        &["allemand", "deutsch", "german"],
        &["italien", "italiano", "italian"],
        &["portugais", "portuguese"],
        &["arabe", "arabic"],
        &["chinois", "mandarin", "chinese"],
        &["japonais", "japanese"],
        &["russe", "russian"],
        &["neerlandais", "néerlandais", "dutch"],
        &["hebreu", "hébreu", "hebrew"],
        &["polonais", "polish"],
        &["turc", "turkish"],
        &["grec", "greek"],
        &["coreen", "coréen", "korean"],
        &["ukrainien", "ukrainian"],
        &["hindi"],
        &["catalan"],
    ];
    GROUPS
        .iter()
        .filter(|group| group.iter().any(|name| tokens.contains(*name)))
        .count()
}

fn has_skill_section(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "compétences",
        "competences",
        "skills",
        "savoir-faire",
        "savoir faire",
        "technologies",
        "outils",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn safe_progress_label(message: &str) -> Option<String> {
    const SAFE: &[&str] = &[
        "Lecture du fichier",
        "Lecture du fichier impossible",
        "Texte extrait",
        "Extraction du contenu impossible",
        "Analyse démarrée",
        "Analyse sans résultat, nouvel essai",
        "Analyse du CV impossible",
        "Aucune donnée exploitable",
        "Analyse terminée",
        "Complément des expériences",
        "Appel LLM",
        "Lecture du fichier…",
        "Extraction du contenu…",
        "Analyse du CV…",
        "Nouvel essai d'analyse…",
        "Préparation de la revue…",
    ];
    if SAFE.contains(&message) || message.ends_with("détectée") || message.ends_with("détectées")
    {
        Some(message.to_owned())
    } else {
        None
    }
}

fn append_progress(path: &Path, item: &serde_json::Value) {
    let id = item.get("cv_id").and_then(|v| v.as_str()).unwrap_or("CV-?");
    let ok = item.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let duration = item
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let exp = item
        .pointer("/counts/experience")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let score = item.pointer("/heuristic/score").and_then(|v| v.as_u64());
    let category = item
        .get("error_category")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let line = if ok {
        format!(
            "{} {id} ok duration_ms={duration} exp={exp} heuristic={}\n",
            now_label(),
            fmt_u64(score)
        )
    } else {
        format!(
            "{now} {id} fail duration_ms={duration} category={category}\n",
            now = now_label()
        )
    };
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        use std::io::Write;
        let _ = file.write_all(line.as_bytes());
    }
}

fn spawn_heartbeat(state: Arc<Mutex<RunState>>, journal: PathBuf) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(120));
        loop {
            ticker.tick().await;
            let Ok(guard) = state.lock() else {
                break;
            };
            if guard.finished {
                break;
            }
            if guard.current.is_some() {
                rewrite_journal(&journal, &guard);
            }
        }
    });
}

fn rewrite_journal(path: &Path, state: &RunState) {
    let summary = summarize(&state.items);
    let mut body = String::new();
    body.push_str("# Import CV benchmark\n\n");
    body.push_str("Change: current product pipeline. Layout extraction, grounding, contact fill and formation salvage. Model comes from CANDILOG_CV_MODEL.\n");
    body.push_str(if state.finished {
        "Status: finished\n"
    } else {
        "Status: running\n"
    });
    body.push_str("Pipeline: `AiService::import_profile` (extract, truncate 12000, Ollama, parse/repair, validate, dates, ground). `profile_apply_import` not called.\n");
    body.push_str(&format!("Model: `{}`\n", state.model));
    body.push_str("Provider: ollama\n");
    body.push_str(&format!("Endpoint: `{}`\n", state.endpoint));
    body.push_str(&format!("Temperature used: {} (settings row + Ollama options.temperature; model card default 0.1 overridden)\n", state.temperature));
    body.push_str("Database: throwaway sqlite under `.benchmark/bench.sqlite` (migrations + settings + empty profile). User daily DB untouched.\n");
    body.push_str(
        "Score: heuristic baseline vs Candilog-extracted PDF text. Not human ground truth.\n",
    );
    body.push_str(&format!("Updated: {}\n\n", now_label()));
    body.push_str("## Summary\n\n");
    body.push_str(&format!("- Mapped PDFs: {}\n", state.mapping_len));
    body.push_str(&format!("- Processed: {}\n", summary.n));
    body.push_str(&format!("- Successes: {}\n", summary.ok));
    body.push_str(&format!("- Failures: {}\n", summary.fail));
    body.push_str(&format!(
        "- Mean duration ms: {}\n",
        fmt_opt(summary.mean_ms)
    ));
    body.push_str(&format!(
        "- Median duration ms: {}\n",
        fmt_opt(summary.median_ms)
    ));
    body.push_str(&format!(
        "- Mean heuristic score /100: {}\n",
        fmt_opt(summary.mean_score)
    ));
    body.push_str(&format!(
        "- Median heuristic score /100: {}\n",
        fmt_opt(summary.median_score)
    ));
    if summary.categories.is_empty() {
        body.push_str("- Error categories: none\n");
    } else {
        body.push_str("- Error categories:\n");
        for (name, count) in &summary.categories {
            body.push_str(&format!("  - {name}: {count}\n"));
        }
    }
    if let Some(current) = &state.current {
        body.push_str(&format!(
            "\nCurrent: {} running for {} s\n",
            current.cv_id,
            current.started.elapsed().as_secs()
        ));
    }
    body.push_str("\n## Log\n\n");
    for item in &state.items {
        let id = item.get("cv_id").and_then(|v| v.as_str()).unwrap_or("CV-?");
        let ok = item.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let duration = item
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let tokens = item.get("tokens").and_then(|v| v.as_u64());
        let exp = item
            .pointer("/counts/experience")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let edu = item
            .pointer("/counts/formation")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let skill = item
            .pointer("/counts/skill")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let lang = item
            .pointer("/counts/language")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let score = item.pointer("/heuristic/score").and_then(|v| v.as_u64());
        if ok {
            let calls = item
                .get("llm_call_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            body.push_str(&format!(
                "- {id} ok duration_ms={duration} tokens={} exp={exp} formation={edu} skill={skill} language={lang} heuristic={} llm_calls={calls}\n",
                fmt_u64(tokens),
                fmt_u64(score)
            ));
        } else {
            let code = item
                .get("error_code")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let category = item
                .get("error_category")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            body.push_str(&format!(
                "- {id} fail duration_ms={duration} tokens={} error={code}/{category}\n",
                fmt_u64(tokens)
            ));
        }
    }
    if let Some(current) = &state.current {
        body.push_str(&format!(
            "- {} running {}s\n",
            current.cv_id,
            current.started.elapsed().as_secs()
        ));
    }
    let tmp = path.with_extension("md.tmp");
    if std::fs::write(&tmp, body).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

struct Summary {
    n: usize,
    ok: usize,
    fail: usize,
    mean_ms: Option<u64>,
    median_ms: Option<u64>,
    mean_score: Option<u64>,
    median_score: Option<u64>,
    categories: BTreeMap<String, usize>,
}

fn summarize(items: &[serde_json::Value]) -> Summary {
    let n = items.len();
    let ok = items
        .iter()
        .filter(|i| i.get("ok").and_then(|v| v.as_bool()) == Some(true))
        .count();
    let mut categories = BTreeMap::new();
    for item in items {
        if item.get("ok").and_then(|v| v.as_bool()) == Some(false) {
            if let Some(category) = item.get("error_category").and_then(|v| v.as_str()) {
                *categories.entry(category.to_owned()).or_insert(0) += 1;
            }
        }
    }
    let durations: Vec<u64> = items
        .iter()
        .filter_map(|i| i.get("duration_ms").and_then(|v| v.as_u64()))
        .collect();
    let scores: Vec<u64> = items
        .iter()
        .filter_map(|i| i.pointer("/heuristic/score").and_then(|v| v.as_u64()))
        .collect();
    Summary {
        n,
        ok,
        fail: n - ok,
        mean_ms: mean(&durations),
        median_ms: median(&durations),
        mean_score: mean(&scores),
        median_score: median(&scores),
        categories,
    }
}

fn mean(values: &[u64]) -> Option<u64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<u64>() / values.len() as u64)
    }
}

fn median(values: &[u64]) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    Some(sorted[sorted.len() / 2])
}

fn fmt_opt(value: Option<u64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "n/a".into())
}

fn fmt_u64(value: Option<u64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
}

fn now_label() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let local = secs + 2 * 3600;
    let day = local / 86400;
    let rem = local % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    let (y, m, d) = civil_from_days(day as i64);
    format!("{y:04}-{m:02}-{d:02} {hour:02}:{min:02}:{sec:02} Paris")
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn io_err(error: serde_json::Error) -> std::io::Error {
    std::io::Error::other(error)
}
