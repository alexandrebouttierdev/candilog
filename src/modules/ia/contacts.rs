//! Extraction **déterministe** des coordonnées d'un CV, en Rust (aucun `LLM`).
//!
//! E-mail, téléphone et URLs (`LinkedIn`, `GitHub`, site) suivent des formats réguliers :
//! une regex les extrait de façon fiable et reproductible. Les petits modèles, eux, les
//! **déforment ou inventent** régulièrement (un caractère d'e-mail changé, un domaine
//! plausible mais faux). On retire donc ces champs du travail du `LLM` : ils sont extraits
//! ici depuis le texte source (autoritaire), le `LLM` ne servant que de repli.

use regex::Regex;
use std::sync::OnceLock;

/// Compile un motif **constant** connu valide (jamais une entrée utilisateur).
///
/// `expect` est justifié ici : le motif est une constante du code source ; un échec de
/// compilation serait un bug détecté au premier test, pas une erreur d'exécution possible.
#[allow(clippy::expect_used)]
fn compiled(pattern: &'static str) -> Regex {
    Regex::new(pattern).expect("motif regex constant valide")
}

/// Coordonnées repérées dans le texte brut d'un CV.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Contacts {
    /// Adresse e-mail (première trouvée).
    pub email: Option<String>,
    /// Numéro de téléphone normalisé (première séquence de 9 à 15 chiffres plausible).
    pub phone: Option<String>,
    /// URL/handle `LinkedIn`.
    pub linkedin: Option<String>,
    /// URL/handle `GitHub`.
    pub github: Option<String>,
    /// Autre site web (hors `LinkedIn`/`GitHub`).
    pub website: Option<String>,
}

/// Compile (une seule fois) et renvoie la regex d'e-mail.
fn email_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| compiled(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}"))
}

/// Compile (une seule fois) la regex de candidat téléphone (validée ensuite par le nombre de chiffres).
fn phone_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| compiled(r"\+?\d[\d ()\.\-]{7,}\d"))
}

/// Compile (une seule fois) la regex `LinkedIn`.
fn linkedin_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        compiled(r"(?i)(?:https?://)?(?:[a-z0-9]+\.)?linkedin\.com/(?:in|pub)/[A-Za-z0-9._%\-/]+")
    })
}

/// Compile (une seule fois) la regex `GitHub`.
fn github_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| compiled(r"(?i)(?:https?://)?(?:www\.)?github\.com/[A-Za-z0-9._\-]+"))
}

/// Compile (une seule fois) la regex d'URL générique (`http(s)://…`).
fn url_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| compiled(r"(?i)https?://[^\s)>\]]+"))
}

/// Retire la ponctuation de fin souvent collée à une URL extraite (`.`, `,`, `)`…).
fn trim_url(raw: &str) -> String {
    raw.trim_end_matches(['.', ',', ';', ')', ']', '>', '"', '\''])
        .to_string()
}

/// Valide un candidat téléphone : conserve la sous-chaîne si elle contient 9 à 15 chiffres.
fn valid_phone(candidate: &str) -> Option<String> {
    let digits = candidate.chars().filter(char::is_ascii_digit).count();
    (9..=15)
        .contains(&digits)
        .then(|| candidate.trim().to_string())
}

/// Extrait les coordonnées présentes dans le texte brut d'un CV.
#[must_use]
pub fn extract_contacts(text: &str) -> Contacts {
    let email = email_re().find(text).map(|m| m.as_str().to_string());
    let linkedin = linkedin_re().find(text).map(|m| trim_url(m.as_str()));
    let github = github_re().find(text).map(|m| trim_url(m.as_str()));
    // Site web : première URL http(s) qui n'est ni LinkedIn ni GitHub.
    let website = url_re()
        .find_iter(text)
        .map(|m| trim_url(m.as_str()))
        .find(|u| {
            let low = u.to_lowercase();
            !low.contains("linkedin.com") && !low.contains("github.com")
        });
    // Téléphone : première séquence plausible. On exclut les candidats contenant un « @ »
    // (fragments d'e-mail) via la recherche sur des zones sans arobase.
    let phone = phone_re()
        .find_iter(text)
        .filter(|m| !m.as_str().contains('@'))
        .find_map(|m| valid_phone(m.as_str()));

    Contacts {
        email,
        phone,
        linkedin,
        github,
        website,
    }
}

#[cfg(test)]
#[path = "tests/contacts/mod.rs"]
mod tests;
