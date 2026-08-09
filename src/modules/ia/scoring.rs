//! Calcul local du score `ATS` (profil × offre), déterministe et sans `LLM`.

use crate::modules::ia::cv_model::{GeneratedCv, MatchScore, ParsedOffer};
use crate::shared::profile::Profile;
use chrono::Datelike;

/// Calcule le score `ATS` du profil contre l'offre (local, déterministe).
#[must_use]
pub fn score(profile: &Profile, offer: &ParsedOffer) -> MatchScore {
    let (skills, matched, missing) = score_skills(profile, offer);
    let ats = score_keywords(profile, offer);
    let experience = score_experience(profile, offer);
    let sum = usize::from(skills) * 40 + usize::from(experience) * 40 + usize::from(ats) * 20;
    let total = pct_u8((sum + 50) / 100);
    MatchScore {
        total,
        skills,
        experience,
        ats,
        matched,
        missing,
    }
}

/// Calcule le score `ATS` d'un CV importé contre l'offre (local, déterministe).
///
/// L'expérience n'est pas évaluable (le CV parsé n'a pas de dates) : `experience`
/// vaut `0` et le total est recalculé sur les compétences (2/3) et les mots-clés (1/3).
#[must_use]
pub fn score_imported(cv: &GeneratedCv, offer: &ParsedOffer) -> MatchScore {
    let cv_skills: Vec<String> = cv.skills.iter().map(|s| s.to_lowercase()).collect();
    let mut matched = Vec::new();
    let mut missing = Vec::new();
    for skill in &offer.skills {
        if cv_skills.contains(&skill.to_lowercase()) {
            matched.push(skill.clone());
        } else {
            missing.push(skill.clone());
        }
    }
    // Listes vides = critère indéfini → neutre (100), pour ne pas plomber le total
    // (cohérent avec l'expérience sans exigence dans `score`).
    let skills = if offer.skills.is_empty() {
        100
    } else {
        pct_u8(matched.len() * 100 / offer.skills.len())
    };
    let text = cv_text(cv).to_lowercase();
    let present = offer
        .keywords
        .iter()
        .filter(|k| text.contains(&k.to_lowercase()))
        .count();
    let ats = if offer.keywords.is_empty() {
        100
    } else {
        pct_u8(present * 100 / offer.keywords.len())
    };
    let total = pct_u8((usize::from(skills) * 2 + usize::from(ats)) / 3);
    MatchScore {
        total,
        skills,
        experience: 0,
        ats,
        matched,
        missing,
    }
}

/// Concatène le texte pertinent d'un CV importé (résumé, expériences, compétences).
fn cv_text(cv: &GeneratedCv) -> String {
    let mut parts = vec![cv.summary.clone()];
    for e in &cv.experiences {
        parts.push(e.title.clone());
        parts.push(e.description.clone());
    }
    parts.extend(cv.skills.iter().cloned());
    parts.join(" ")
}

/// Sous-score compétences (40 %) + listes `matched`/`missing`.
fn score_skills(profile: &Profile, offer: &ParsedOffer) -> (u8, Vec<String>, Vec<String>) {
    let profile_skills: Vec<String> = profile
        .skills
        .iter()
        .map(|s| s.name.to_lowercase())
        .collect();
    let mut matched = Vec::new();
    let mut missing = Vec::new();
    for skill in &offer.skills {
        if profile_skills.contains(&skill.to_lowercase()) {
            matched.push(skill.clone());
        } else {
            missing.push(skill.clone());
        }
    }
    let pct = pct_u8(matched.len() * 100 / offer.skills.len().max(1));
    (pct, matched, missing)
}

/// Sous-score densité de mots-clés (20 %).
fn score_keywords(profile: &Profile, offer: &ParsedOffer) -> u8 {
    let text = profile_text(profile).to_lowercase();
    let present = offer
        .keywords
        .iter()
        .filter(|k| text.contains(&k.to_lowercase()))
        .count();
    pct_u8(present * 100 / offer.keywords.len().max(1))
}

/// Sous-score expérience (40 %) : années du profil vs années requises.
fn score_experience(profile: &Profile, offer: &ParsedOffer) -> u8 {
    let required = offer.experience.as_deref().map_or(0, first_int);
    if required == 0 {
        return 100;
    }
    let years = estimate_profile_years(profile);
    pct_u8(years * 100 / required)
}

/// Concatène le texte pertinent du profil (accroche, résumé, expériences, compétences).
fn profile_text(profile: &Profile) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(h) = &profile.personal.headline {
        parts.push(h.clone());
    }
    if let Some(s) = &profile.personal.summary {
        parts.push(s.clone());
    }
    for e in &profile.experiences {
        parts.push(e.title.clone());
        if let Some(d) = &e.description {
            parts.push(d.clone());
        }
    }
    for s in &profile.skills {
        parts.push(s.name.clone());
    }
    parts.join(" ")
}

/// Estime le nombre d'années d'expérience à partir des dates du profil.
fn estimate_profile_years(profile: &Profile) -> usize {
    let now = chrono::Utc::now().year();
    let mut total: i64 = 0;
    for exp in &profile.experiences {
        let Some(start) = extract_year(&exp.start_date) else {
            continue;
        };
        let end = match exp.end_date.as_deref().and_then(extract_year) {
            Some(y) => y,
            None => now,
        };
        total += i64::from((end - start).max(0));
    }
    usize::try_from(total).unwrap_or_default()
}

/// Premier entier trouvé dans une chaîne (0 si aucun).
fn first_int(s: &str) -> usize {
    let mut digits = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if !digits.is_empty() {
            break;
        }
    }
    digits.parse::<usize>().unwrap_or_default()
}

/// Première année à 4 chiffres trouvée dans une chaîne.
fn extract_year(s: &str) -> Option<i32> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes[i..i + 4].iter().all(u8::is_ascii_digit) {
            return s.get(i..i + 4).and_then(|y| y.parse().ok());
        }
        i += 1;
    }
    None
}

/// Convertit un pourcentage en `u8`, borné à 100.
fn pct_u8(value: usize) -> u8 {
    match u8::try_from(value) {
        Ok(n) if n <= 100 => n,
        Ok(_) | Err(_) => 100,
    }
}

#[cfg(test)]
#[path = "tests/scoring/mod.rs"]
mod tests;
