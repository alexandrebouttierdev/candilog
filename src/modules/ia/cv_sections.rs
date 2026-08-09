//! Pré-découpage heuristique du texte d'un CV par section, **en Rust** (aucun `LLM`).
//!
//! Objectif : n'envoyer à chaque appel `LLM` spécialisé que le fragment de CV qui le
//! concerne, au lieu du texte complet ×4. Sur un petit modèle local, le *prefill*
//! (relecture du prompt) domine la latence : cibler les fragments divise le travail.
//!
//! L'heuristique repère les lignes d'en-tête de section (courtes, mots-clés connus,
//! accents ignorés). **Repli systématique** : dès qu'une section attendue n'est pas
//! détectée — ou si le CV semble sans structure — l'appel concerné reçoit le texte
//! complet. On peut donc perdre le gain de vitesse, jamais du contenu.

/// Longueur maximale (caractères) d'une ligne considérée comme un titre de section.
const MAX_HEADING_CHARS: usize = 48;

/// Catégorie d'une section détectée dans le CV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bucket {
    /// Profil / à propos / contact — rejoint le fragment identité.
    Identity,
    /// Expériences professionnelles.
    Experience,
    /// Formations / diplômes.
    Education,
    /// Compétences techniques.
    Skills,
    /// Langues parlées.
    Languages,
    /// Projets / réalisations.
    Projects,
    /// Certifications / habilitations.
    Certifications,
}

/// Fragments de CV destinés aux 4 appels `LLM` spécialisés.
///
/// Chaque champ contient soit le fragment ciblé, soit le texte complet si la ou les
/// sections attendues n'ont pas été détectées (repli sans perte de contenu).
#[derive(Debug)]
pub struct SectionedCv {
    /// Fragment pour l'extraction d'identité (en-tête + profil/contact).
    pub identity: String,
    /// Fragment pour l'extraction du parcours (expériences + formations).
    pub history: String,
    /// Fragment pour l'extraction des compétences et langues.
    pub skills: String,
    /// Fragment pour l'extraction des projets et certifications.
    pub portfolio: String,
}

/// Translittère les voyelles accentuées françaises et met en minuscules.
fn normalize(line: &str) -> String {
    line.to_lowercase()
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

/// Mots-clés d'en-tête par catégorie, testés dans l'ordre : les catégories les plus
/// spécifiques d'abord (« langues » avant « compétences » car « langages de
/// programmation » relève des compétences, mais les deux alimentent le même fragment —
/// une confusion est sans effet).
const HEADING_RULES: [(&[&str], Bucket); 7] = [
    (
        &["certif", "accreditation", "habilitation"],
        Bucket::Certifications,
    ),
    (&["langue", "language"], Bucket::Languages),
    (
        &["projet", "project", "realisation", "portfolio"],
        Bucket::Projects,
    ),
    (
        &[
            "formation",
            "education",
            "diplome",
            "etude",
            "studies",
            "academic",
            "scolarite",
        ],
        Bucket::Education,
    ),
    (
        &[
            "experience",
            "parcours",
            "emploi",
            "work history",
            "employment",
            "career",
        ],
        Bucket::Experience,
    ),
    (
        &[
            "competence",
            "skill",
            "technolog",
            "stack",
            "outil",
            "expertise",
            "savoir-faire",
        ],
        Bucket::Skills,
    ),
    (
        &[
            "profil",
            "profile",
            "about",
            "a propos",
            "resume",
            "summary",
            "contact",
            "coordonnee",
            "personal",
        ],
        Bucket::Identity,
    ),
];

/// Reconnaît une ligne d'en-tête de section et sa catégorie.
///
/// Une ligne est un en-tête si elle est courte (≤ [`MAX_HEADING_CHARS`]) et contient un
/// mot-clé connu. Les mots-clés ambigus sont volontairement exclus (ex. « licence »,
/// à la fois diplôme français et *license* de certification).
fn detect_heading(line: &str) -> Option<Bucket> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_HEADING_CHARS {
        return None;
    }
    let norm = normalize(trimmed);
    HEADING_RULES
        .iter()
        .find(|(keywords, _)| keywords.iter().any(|k| norm.contains(k)))
        .map(|(_, bucket)| *bucket)
}

/// Découpe le texte nettoyé d'un CV en fragments ciblés pour les appels spécialisés.
///
/// Voir la doc du module pour l'heuristique et la politique de repli.
#[must_use]
pub fn split_cv(text: &str) -> SectionedCv {
    // En-tête implicite : tout ce qui précède le premier titre détecté (nom, contact…).
    let mut header = String::new();
    let mut chunks: Vec<(Bucket, String)> = Vec::new();
    let mut current: Option<usize> = None;

    for line in text.lines() {
        if let Some(bucket) = detect_heading(line) {
            chunks.push((bucket, String::new()));
            current = Some(chunks.len() - 1);
        }
        if let Some(index) = current {
            chunks[index].1.push_str(line);
            chunks[index].1.push('\n');
        } else {
            header.push_str(line);
            header.push('\n');
        }
    }

    // CV sans structure détectable : texte complet partout (aucune perte possible).
    if chunks.len() < 2 {
        return SectionedCv {
            identity: text.to_string(),
            history: text.to_string(),
            skills: text.to_string(),
            portfolio: text.to_string(),
        };
    }

    assemble_sectioned_cv(&header, &chunks, text)
}

/// Assemble les 4 fragments (`identity`, `history`, `skills`, `portfolio`) à partir
/// des chunks détectés et du texte de repli. Applique la politique documentée dans
/// [`split_cv`] : chaque section reçoit le texte complet si les chunks attendus sont
/// absents, sinon seuls les fragments ciblés sont concaténés.
fn assemble_sectioned_cv(
    header: &str,
    chunks: &[(Bucket, String)],
    full_text: &str,
) -> SectionedCv {
    let collect = |buckets: &[Bucket]| -> String {
        chunks
            .iter()
            .filter(|(bucket, _)| buckets.contains(bucket))
            .map(|(_, chunk)| chunk.as_str())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    };
    let has = |bucket: Bucket| chunks.iter().any(|(b, _)| *b == bucket);

    // Identité : en-tête + sections profil/contact. L'en-tête suffit s'il est non vide.
    let identity_extra = collect(&[Bucket::Identity]);
    let identity = format!("{}\n{identity_extra}", header.trim());
    let identity = if identity.trim().is_empty() {
        full_text.to_string()
    } else {
        identity.trim().to_string()
    };

    // Parcours : exige expériences ET formations détectées, sinon texte complet
    // (une formation sous un titre non reconnu serait perdue par un fragment partiel).
    let history = if has(Bucket::Experience) && has(Bucket::Education) {
        collect(&[Bucket::Experience, Bucket::Education])
    } else {
        full_text.to_string()
    };

    // Compétences : la section langues, souvent imbriquée, est jointe si détectée.
    let skills = if has(Bucket::Skills) {
        collect(&[Bucket::Skills, Bucket::Languages])
    } else {
        full_text.to_string()
    };

    // Projets/certifications : au moins l'une des deux sections détectée, sinon texte
    // complet (elles peuvent exister sans titre dédié).
    let portfolio = if has(Bucket::Projects) || has(Bucket::Certifications) {
        collect(&[Bucket::Projects, Bucket::Certifications])
    } else {
        full_text.to_string()
    };

    SectionedCv {
        identity,
        history,
        skills,
        portfolio,
    }
}

#[cfg(test)]
#[path = "tests/cv_sections/mod.rs"]
mod tests;
