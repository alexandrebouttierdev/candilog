//! Scoring déterministe du benchmark utilisateur CV_BENCHMARK.pdf.

use super::managed_ollama::{
    benchmark_quality_label, UserBenchmarkCategoryScore, UserBenchmarkMetrics, UserBenchmarkResult,
    USER_BENCHMARK_VERSION,
};
use crate::features::profile::domain::Profile;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// PDF de référence compilé dans le binaire — `CARGO_MANIFEST_DIR` n'existe pas
/// sur la machine de l'utilisateur après installation.
const BENCHMARK_PDF_BYTES: &[u8] = include_bytes!("../../../../resources/CV_BENCHMARK.pdf");
const BENCHMARK_EXPECTED_JSON: &str =
    include_str!("../../../../resources/CV_BENCHMARK.expected.json");

#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkGroundTruth {
    pub benchmark_version: u32,
    pub profile: Profile,
}

/// Charge la ground truth embarquée dans le binaire.
pub fn load_ground_truth() -> Result<BenchmarkGroundTruth, String> {
    serde_json::from_str(BENCHMARK_EXPECTED_JSON)
        .map_err(|error| format!("ground truth invalide: {error}"))
}

/// Chemin source dans l'arbre de développement (outils, baselining).
#[must_use]
pub fn benchmark_ground_truth_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/CV_BENCHMARK.expected.json")
}

/// Matérialise le PDF de benchmark dans un fichier temporaire stable.
///
/// Les extracteurs (`pdftotext`, `pdftoppm`, `pdf-extract`) travaillent sur des chemins.
/// Le contenu vient du binaire : après installation, `resources/` du dépôt n'est plus là
/// et n'est pas non plus dans le bundle Tauri.
pub fn benchmark_pdf_path() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("candilog-user-benchmark");
    std::fs::create_dir_all(&dir).map_err(|error| {
        format!(
            "impossible de préparer le dossier du CV de benchmark ({}): {error}",
            dir.display()
        )
    })?;
    let path = dir.join(format!("CV_BENCHMARK-v{USER_BENCHMARK_VERSION}.pdf"));
    let needs_write = match std::fs::metadata(&path) {
        Ok(meta) => meta.len() as usize != BENCHMARK_PDF_BYTES.len(),
        Err(_) => true,
    };
    if needs_write {
        let partial = path.with_extension("pdf.partial");
        std::fs::write(&partial, BENCHMARK_PDF_BYTES).map_err(|error| {
            format!(
                "impossible d'écrire le CV de benchmark ({}): {error}",
                partial.display()
            )
        })?;
        std::fs::rename(&partial, &path).map_err(|error| {
            format!(
                "impossible de publier le CV de benchmark ({}): {error}",
                path.display()
            )
        })?;
    }
    Ok(path)
}

pub struct BenchmarkScore {
    pub total: u32,
    pub categories: Vec<UserBenchmarkCategoryScore>,
    pub hallucination_count: u32,
}

/// Compare le profil extrait à la ground truth avec pénalités fortes sur les hallucinations.
#[must_use]
pub fn score_extracted_profile(expected: &Profile, actual: &Profile) -> BenchmarkScore {
    let identity = score_identity(&expected.identity, &actual.identity);
    let experiences = score_experiences(&expected.experiences, &actual.experiences);
    let education = score_education(&expected.education, &actual.education);
    let skills = score_skills(&expected.skills, &actual.skills);
    let languages = score_languages(&expected.languages, &actual.languages);
    let hallucinations = count_hallucinations(expected, actual);

    let hallucination_penalty = hallucinations.saturating_mul(8);
    let raw = identity.score + experiences.score + education.score + skills.score + languages.score;
    let total = raw.saturating_sub(hallucination_penalty).min(100);

    BenchmarkScore {
        total,
        categories: vec![identity, experiences, education, skills, languages],
        hallucination_count: hallucinations,
    }
}

/// Méthode d'analyse réellement exécutée lors d'un benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkAnalysisOutcome {
    pub method_used: crate::features::ai::domain::CvAnalysisMethodUsed,
    pub fallback_used: bool,
}

#[must_use]
pub fn build_benchmark_result(
    expected_version: u32,
    score: BenchmarkScore,
    metrics: UserBenchmarkMetrics,
    provider_label: String,
    model_label: String,
    remote_warning: bool,
    analysis: BenchmarkAnalysisOutcome,
) -> UserBenchmarkResult {
    UserBenchmarkResult {
        score: score.total,
        quality: benchmark_quality_label(score.total),
        metrics,
        categories: score.categories,
        hallucination_count: score.hallucination_count,
        benchmark_version: expected_version,
        provider_label,
        model_label,
        remote_warning,
        method_used: analysis.method_used,
        fallback_used: analysis.fallback_used,
    }
}

fn score_identity(
    expected: &crate::features::profile::domain::Identity,
    actual: &crate::features::profile::domain::Identity,
) -> UserBenchmarkCategoryScore {
    let mut points = 0_u32;
    let max = 10_u32;
    if field_match(&expected.first_name, &actual.first_name) {
        points += 2;
    }
    if field_match(&expected.name, &actual.name) {
        points += 2;
    }
    if field_match(&expected.email, &actual.email) {
        points += 2;
    }
    if phone_match(expected.phone.as_deref(), actual.phone.as_deref()) {
        points += 2;
    }
    if optional_field_match(expected.address.as_deref(), actual.address.as_deref()) {
        points += 2;
    }
    UserBenchmarkCategoryScore {
        label: "Identité".into(),
        score: points.min(max),
        max_score: max,
    }
}

fn score_experiences(
    expected: &[crate::features::profile::domain::Experience],
    actual: &[crate::features::profile::domain::Experience],
) -> UserBenchmarkCategoryScore {
    let max = 30_u32;
    let score = list_overlap_score(expected, actual, |item| {
        format!(
            "{}|{}|{}",
            normalize(&item.title),
            normalize(&item.company),
            normalize_date_key(&item.start_date)
        )
    });
    UserBenchmarkCategoryScore {
        label: "Expériences".into(),
        score: ((score * max as f64).round() as u32).min(max),
        max_score: max,
    }
}

fn score_education(
    expected: &[crate::features::profile::domain::Education],
    actual: &[crate::features::profile::domain::Education],
) -> UserBenchmarkCategoryScore {
    let max = 20_u32;
    let score = list_overlap_score(expected, actual, |item| {
        format!("{}|{}", normalize(&item.degree), normalize(&item.school))
    });
    UserBenchmarkCategoryScore {
        label: "Formations".into(),
        score: ((score * max as f64).round() as u32).min(max),
        max_score: max,
    }
}

fn score_skills(
    expected: &[crate::features::profile::domain::Skill],
    actual: &[crate::features::profile::domain::Skill],
) -> UserBenchmarkCategoryScore {
    let max = 15_u32;
    let score = list_overlap_score(expected, actual, |item| normalize(&item.name));
    UserBenchmarkCategoryScore {
        label: "Compétences".into(),
        score: ((score * max as f64).round() as u32).min(max),
        max_score: max,
    }
}

fn score_languages(
    expected: &[crate::features::profile::domain::Language],
    actual: &[crate::features::profile::domain::Language],
) -> UserBenchmarkCategoryScore {
    let max = 10_u32;
    let score = list_overlap_score(expected, actual, |item| normalize(&item.name));
    UserBenchmarkCategoryScore {
        label: "Langues".into(),
        score: ((score * max as f64).round() as u32).min(max),
        max_score: max,
    }
}

fn count_hallucinations(expected: &Profile, actual: &Profile) -> u32 {
    let mut count = 0_u32;
    for experience in &actual.experiences {
        if experience.company.trim().is_empty() {
            continue;
        }
        if !expected.experiences.iter().any(|item| {
            similar(&item.company, &experience.company) || similar(&item.title, &experience.title)
        }) {
            count += 1;
        }
    }
    for education in &actual.education {
        if education.degree.trim().is_empty() {
            continue;
        }
        if !expected.education.iter().any(|item| {
            similar(&item.degree, &education.degree) || similar(&item.school, &education.school)
        }) {
            count += 1;
        }
    }
    count
}

fn list_overlap_score<T, F>(expected: &[T], actual: &[T], key: F) -> f64
where
    F: Fn(&T) -> String,
{
    if expected.is_empty() {
        return if actual.is_empty() { 1.0 } else { 0.5 };
    }
    let mut expected_keys = Vec::with_capacity(expected.len());
    for item in expected {
        expected_keys.push(key(item));
    }
    let mut actual_keys = Vec::with_capacity(actual.len());
    for item in actual {
        actual_keys.push(key(item));
    }
    let matched = expected_keys
        .iter()
        .filter(|value| actual_keys.iter().any(|actual| actual == *value))
        .count();
    matched as f64 / expected_keys.len() as f64
}

fn field_match(expected: &str, actual: &str) -> bool {
    let expected = normalize(expected);
    let actual = normalize(actual);
    !expected.is_empty()
        && (expected == actual || actual.contains(&expected) || expected.contains(&actual))
}

fn optional_field_match(expected: Option<&str>, actual: Option<&str>) -> bool {
    match (expected, actual) {
        (None, _) | (Some(""), _) => true,
        (Some(expected), Some(actual)) => field_match(expected, actual),
        (Some(_), None) => false,
    }
}

fn phone_match(expected: Option<&str>, actual: Option<&str>) -> bool {
    let expected_digits = expected.map(digits_only).unwrap_or_default();
    let actual_digits = actual.map(digits_only).unwrap_or_default();
    !expected_digits.is_empty()
        && (expected_digits == actual_digits
            || expected_digits.ends_with(&actual_digits)
            || actual_digits.ends_with(&expected_digits))
}

fn similar(left: &str, right: &str) -> bool {
    let left = normalize(left);
    let right = normalize(right);
    left == right || left.contains(&right) || right.contains(&left)
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace(['é', 'è', 'ê'], "e")
        .replace('à', "a")
        .replace('ù', "u")
        .replace('ô', "o")
        .replace('ç', "c")
}

/// Aligné sur `normalize_profile_dates` : « 01/2023 » et « 2023-01 » doivent scorer pareil.
fn normalize_date_key(value: &str) -> String {
    let key = normalize(value);
    let groups: Vec<&str> = key
        .split(|character: char| !character.is_ascii_digit())
        .filter(|group| !group.is_empty())
        .collect();
    let year_index = groups.iter().position(|group| {
        group.len() == 4
            && group
                .parse::<u32>()
                .is_ok_and(|year| (1900..=2100).contains(&year))
    });
    let Some(year_index) = year_index else {
        return key;
    };
    let year = groups[year_index];
    let month = groups
        .get(year_index + 1)
        .and_then(|group| month_value(group))
        .or_else(|| {
            year_index
                .checked_sub(1)
                .and_then(|before| month_value(groups[before]))
        });
    match month {
        Some(month) => format!("{year}-{month:02}"),
        None => year.to_owned(),
    }
}

fn month_value(group: &str) -> Option<u32> {
    (group.len() <= 2)
        .then(|| group.parse().ok())
        .flatten()
        .filter(|month| (1..=12).contains(month))
}

fn digits_only(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profile::domain::{Education, Experience, Identity, Language, Skill};

    #[test]
    fn score_parfait_quand_identique() {
        let profile = Profile {
            identity: Identity {
                first_name: "Thomas".into(),
                name: "Candilog".into(),
                email: "thomas.candilog.dev@gmail.com".into(),
                phone: Some("0618425791".into()),
                ..Identity::default()
            },
            experiences: vec![Experience {
                title: "Senior Full-Stack Developer".into(),
                company: "AlthéaRH".into(),
                start_date: "01/2023".into(),
                ..Experience::default()
            }],
            skills: vec![Skill {
                name: "React".into(),
                description: None,
            }],
            education: vec![Education {
                degree: "Master".into(),
                school: "INSA".into(),
                ..Education::default()
            }],
            languages: vec![Language {
                name: "Français".into(),
                level: "Natif".into(),
            }],
            ..Profile::default()
        };
        let score = score_extracted_profile(&profile, &profile);
        assert!(score.total >= 80);
        assert_eq!(score.hallucination_count, 0);
    }

    #[test]
    fn hallucination_penalise_le_score() {
        let expected = Profile::default();
        let mut actual = Profile::default();
        actual.experiences.push(Experience {
            title: "Inventé".into(),
            company: "Société fictive".into(),
            start_date: "2020".into(),
            ..Experience::default()
        });
        let score = score_extracted_profile(&expected, &actual);
        assert!(score.hallucination_count >= 1);
        assert!(score.total < 100);
    }

    #[test]
    fn ground_truth_chargeable() {
        let truth = load_ground_truth();
        assert!(truth.is_ok(), "{}", truth.err().unwrap_or_default());
        assert_eq!(truth.unwrap().benchmark_version, 2);
    }

    /// Régression : en installation, `CARGO_MANIFEST_DIR` pointe vers la machine de build,
    /// pas vers le disque de l'utilisateur. Le PDF doit donc être extrait du binaire.
    #[test]
    fn pdf_benchmark_est_extrait_vers_un_fichier_temporaire() {
        let path = benchmark_pdf_path().expect("matérialisation du PDF de benchmark");
        assert!(
            path.starts_with(std::env::temp_dir()),
            "attendu sous {:?}, obtenu {} — le chemin source compile-time n'existe pas après installation",
            std::env::temp_dir(),
            path.display()
        );
        let bytes = std::fs::read(&path).expect("lecture du PDF matérialisé");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "le fichier matérialisé doit être un PDF"
        );
        assert_eq!(bytes.as_slice(), BENCHMARK_PDF_BYTES);
    }

    #[test]
    fn les_dates_d_experience_scorent_malgre_les_formats_divergents() {
        let expected = Profile {
            experiences: vec![Experience {
                title: "Senior Full-Stack Developer".into(),
                company: "AlthéaRH".into(),
                start_date: "01/2023".into(),
                ..Experience::default()
            }],
            ..Profile::default()
        };
        let actual = Profile {
            experiences: vec![Experience {
                title: "Senior Full-Stack Developer".into(),
                company: "AlthéaRH".into(),
                start_date: "2023-01".into(),
                ..Experience::default()
            }],
            ..Profile::default()
        };
        let score = score_extracted_profile(&expected, &actual);
        let experiences = score
            .categories
            .iter()
            .find(|category| category.label == "Expériences")
            .expect("catégorie expériences");
        assert_eq!(experiences.score, experiences.max_score);
    }

    #[test]
    fn l_adresse_compte_dans_le_score_identite() {
        let expected = Profile {
            identity: Identity {
                first_name: "Thomas".into(),
                name: "Candilog".into(),
                email: "thomas.candilog.dev@gmail.com".into(),
                phone: Some("0618425791".into()),
                address: Some("18 rue des Tanneurs, 35000 Rennes".into()),
                ..Identity::default()
            },
            ..Profile::default()
        };
        let mut without_address = expected.clone();
        without_address.identity.address = None;
        let with = score_extracted_profile(&expected, &expected);
        let without = score_extracted_profile(&expected, &without_address);
        assert!(with.total > without.total);
    }
}
