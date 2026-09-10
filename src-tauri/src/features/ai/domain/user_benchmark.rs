//! Scoring déterministe du benchmark utilisateur CV_BENCHMARK.pdf.

use super::managed_ollama::{
    benchmark_quality_label, UserBenchmarkCategoryScore, UserBenchmarkMetrics, UserBenchmarkResult,
};
use crate::features::profile::domain::Profile;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkGroundTruth {
    pub benchmark_version: u32,
    pub profile: Profile,
}

/// Charge la ground truth embarquée dans les ressources de l'application.
pub fn load_ground_truth() -> Result<BenchmarkGroundTruth, String> {
    let path = benchmark_ground_truth_path();
    let raw = std::fs::read_to_string(&path)
        .map_err(|error| format!("ground truth introuvable ({}): {error}", path.display()))?;
    serde_json::from_str(&raw).map_err(|error| format!("ground truth invalide: {error}"))
}

#[must_use]
pub fn benchmark_ground_truth_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/CV_BENCHMARK.expected.json")
}

#[must_use]
pub fn benchmark_pdf_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/CV_BENCHMARK.pdf")
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

#[must_use]
pub fn build_benchmark_result(
    expected_version: u32,
    score: BenchmarkScore,
    metrics: UserBenchmarkMetrics,
    provider_label: String,
    model_label: String,
    remote_warning: bool,
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
        points += 3;
    }
    if phone_match(expected.phone.as_deref(), actual.phone.as_deref()) {
        points += 3;
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
            normalize(&item.start_date)
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
        assert_eq!(truth.unwrap().benchmark_version, 1);
    }
}
