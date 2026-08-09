//! Validation d'**ancrage** : rejet des informations factuelles absentes du texte source.
//!
//! Un prompt anti-hallucination ne suffit pas sur un petit modèle : il invente parfois un
//! employeur, une école ou une compétence plausibles. Ici, après extraction, on **vérifie
//! côté application** que chaque valeur factuelle figure réellement dans le CV source ; sinon
//! on la retire. Déterministe, sans `LLM`.
//!
//! La comparaison est **tolérante** (minuscules, accents repliés, par tokens) pour ne pas
//! rejeter une valeur légitime que le modèle a normalisée (casse, accents). Elle ne s'applique
//! qu'aux champs factuels (noms d'entreprise, d'école, de compétence, de projet, de
//! certification) — jamais aux textes rédigés (résumé, descriptions) qui sont, eux, reformulés.

use crate::shared::profile::Profile;

/// Replie les voyelles accentuées françaises et met en minuscules (comparaison insensible).
fn fold(raw: &str) -> String {
    raw.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'û' | 'ü' | 'ù' => 'u',
            'ç' => 'c',
            _ => c,
        })
        .collect()
}

/// Indique si `candidate` est ancré dans `source_folded` (déjà replié).
///
/// Ancré si : la valeur repliée est une sous-chaîne du source, **ou** si au moins la moitié de
/// ses tokens significatifs (≥ 3 caractères) y figurent. Les valeurs trop courtes (< 3
/// caractères) ou sans token significatif sont considérées ancrées (indécidables → conservées).
#[must_use]
pub fn is_grounded(source_folded: &str, candidate: &str) -> bool {
    let cand = fold(candidate.trim());
    if cand.len() < 3 {
        return true;
    }
    if source_folded.contains(&cand) {
        return true;
    }
    let tokens: Vec<&str> = cand
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .collect();
    if tokens.is_empty() {
        return true;
    }
    let present = tokens.iter().filter(|t| source_folded.contains(*t)).count();
    present * 2 >= tokens.len()
}

/// Retire du profil les entrées factuelles non ancrées dans le texte source.
///
/// Champs vérifiés : `company` (expériences), `school` (formations), `name` (compétences,
/// projets, certifications). Une entrée dont le champ clé est **présent mais introuvable** dans
/// le source est supprimée (probable hallucination). Un champ clé vide n'est pas jugé (conservé).
/// Les textes rédigés (résumé, descriptions, headline) ne sont jamais touchés.
pub fn ground_profile(profile: &mut Profile, source: &str) {
    let src = fold(source);
    let anchored = |value: &str| value.trim().is_empty() || is_grounded(&src, value);

    profile.experiences.retain(|e| anchored(&e.company));
    profile.education.retain(|e| anchored(&e.school));
    profile.skills.retain(|s| anchored(&s.name));
    profile.projects.retain(|p| anchored(&p.name));
    profile.certifications.retain(|c| anchored(&c.name));
}

#[cfg(test)]
#[path = "tests/grounding/mod.rs"]
mod tests;
