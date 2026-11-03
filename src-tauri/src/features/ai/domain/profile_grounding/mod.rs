//! Recadrage sur le CV d'un profil extrait par un modèle.
//!
//! L'import de profil demande au modèle de recopier le CV sans rien inventer. Les petits
//! modèles locaux ne s'y tiennent pas : ils renvoient des fragments de domaine (« .com »,
//! « .fr », « .org »), des morceaux de mots (« .franc ») et jusqu'à une certification
//! absente du document. Le nettoyage par vacuité ne rejetait que le vide, si bien que ces
//! valeurs arrivaient telles quelles dans l'écran de revue.
//!
//! On ne conserve donc qu'un texte réellement présent dans le CV analysé, comme
//! `ground_imported_resume` le fait déjà pour l'analyse de CV.

use super::normalization::contains_search_term;
use crate::features::profile::domain::{Education, Identity, Profile};

mod contacts;
mod education;

pub use contacts::completer_contacts_vides;
pub use education::completer_formations_manquantes;

/// Options du recadrage : en Vision, le texte PDF n'est qu'un complément partiel — les
/// descriptions libres et le prénom / nom lus sur les images ne doivent pas être effacés
/// faute d'y figurer mot pour mot.
#[derive(Debug, Clone, Copy)]
pub struct GroundingOptions {
    /// Recadrer aussi résumé et descriptions (expériences, formations, projets, etc.).
    pub free_text: bool,
    /// Recadrer prénom et nom : un PDF complémentaire tronqué les effaçait sinon.
    pub person_names: bool,
}

impl Default for GroundingOptions {
    fn default() -> Self {
        Self {
            free_text: true,
            person_names: true,
        }
    }
}

/// Efface d'un profil importé tout texte absent du CV analysé.
///
/// Les dates ne sont pas concernées : `normalize_profile_dates` les a déjà ramenées au
/// format `AAAA-MM`, que le CV n'écrit pas tel quel (« Oct. 2025 »). Les entrées vidées de
/// leur libellé sont retirées ensuite par le nettoyage par vacuité.
pub fn ground_imported_profile(source: &str, profile: &mut Profile) {
    ground_imported_profile_with(source, profile, GroundingOptions::default());
}

/// Recadrage pour le mode Vision : conserve prénom, nom et descriptions libres des images.
pub fn ground_imported_profile_keep_free_text(source: &str, profile: &mut Profile) {
    ground_imported_profile_with(
        source,
        profile,
        GroundingOptions {
            free_text: false,
            person_names: false,
        },
    );
}

fn ground_imported_profile_with(source: &str, profile: &mut Profile, options: GroundingOptions) {
    let identity = &mut profile.identity;
    if options.person_names {
        retenir(source, &mut identity.first_name);
        retenir(source, &mut identity.name);
    }
    retenir(source, &mut identity.email);
    retenir_option(source, &mut identity.phone);
    retenir_option(source, &mut identity.address);
    retenir_option(source, &mut identity.city);
    retenir_option(source, &mut identity.title);
    if options.free_text {
        retenir_description(source, &mut identity.resume);
    }
    retenir_option(source, &mut identity.birth_date);
    retenir_option(source, &mut identity.availability);
    retenir_option(source, &mut identity.desired_contracts);
    retenir_option(source, &mut identity.linkedin);
    retenir_option(source, &mut identity.github);
    retenir_option(source, &mut identity.website);
    // L'âge est un nombre dérivé du CV (« 34 ans ») : on le garde si la date ou
    // le chiffre apparaît dans le texte, sinon on l'efface.
    if let Some(age) = identity.age {
        let age_text = age.to_string();
        if !est_recopie(source, &age_text) {
            identity.age = None;
        }
    }

    for experience in &mut profile.experiences {
        retenir(source, &mut experience.title);
        retenir(source, &mut experience.company);
        retenir_option(source, &mut experience.location);
        if options.free_text {
            retenir_description(source, &mut experience.description);
        }
    }
    for skill in &mut profile.skills {
        retenir(source, &mut skill.name);
        if options.free_text {
            retenir_description(source, &mut skill.description);
        }
    }
    for education in &mut profile.education {
        retenir(source, &mut education.degree);
        retenir(source, &mut education.school);
        retenir_option(source, &mut education.location);
        if options.free_text {
            retenir_description(source, &mut education.description);
        }
    }
    for language in &mut profile.languages {
        retenir(source, &mut language.name);
        retenir(source, &mut language.level);
    }
    for project in &mut profile.projects {
        retenir(source, &mut project.name);
        if options.free_text {
            retenir_description(source, &mut project.description);
        }
        retenir_option(source, &mut project.url);
        retenir_option(source, &mut project.technologies);
    }
    for certification in &mut profile.certifications {
        retenir(source, &mut certification.name);
        retenir_option(source, &mut certification.issuer);
        retenir_option(source, &mut certification.url);
        if options.free_text {
            retenir_description(source, &mut certification.description);
        }
    }
    for interest in &mut profile.interests {
        retenir(source, &mut interest.name);
    }
}

/// Vide une valeur que le CV ne contient pas.
fn retenir(source: &str, value: &mut String) {
    if !est_recopie(source, value) {
        value.clear();
    }
}

/// Retire une valeur optionnelle que le CV ne contient pas.
fn retenir_option(source: &str, value: &mut Option<String>) {
    if !value
        .as_deref()
        .is_some_and(|texte| est_recopie(source, texte))
    {
        *value = None;
    }
}

/// Conserve les fragments d'une description présents dans le CV.
///
/// Une description multi-puces jointe par des sauts de ligne n'apparaît presque jamais
/// telle quelle dans le texte source (puces `•`, colonnes, reformulations légères). On
/// retient chaque fragment recopié ; si aucun ne l'est, le champ est vidé.
fn retenir_description(source: &str, value: &mut Option<String>) {
    let Some(texte) = value.as_deref() else {
        return;
    };
    if texte.trim().is_empty() {
        *value = None;
        return;
    }
    if est_recopie(source, texte) {
        return;
    }
    let kept: Vec<String> = decouper_description(texte)
        .into_iter()
        .filter(|fragment| est_recopie(source, fragment))
        .collect();
    if kept.is_empty() {
        *value = None;
    } else {
        *value = Some(kept.join("\n"));
    }
}

fn decouper_description(texte: &str) -> Vec<String> {
    texte
        .lines()
        .flat_map(|ligne| ligne.split(['•', '●', '▪', '◦', '·', ';', '|']))
        .map(|fragment| {
            fragment
                .trim()
                .trim_start_matches(['-', '–', '—', '*', '·', '•'])
                .trim()
                .to_owned()
        })
        .filter(|fragment| fragment.chars().any(char::is_alphanumeric))
        .collect()
}

/// Une valeur est recopiée si elle porte du texte et apparaît telle quelle dans le CV.
///
/// La recherche exige des frontières alphanumériques : c'est elle qui écarte « .com », que
/// le CV n'écrit qu'accolé à un domaine (`gmail.com`), sans rejeter une valeur légitime.
/// Le garde sur les caractères alphanumériques écarte en amont les valeurs sans contenu
/// (« . »), qu'une ponctuation quelconque du CV suffirait sinon à valider.
fn est_recopie(source: &str, value: &str) -> bool {
    value.chars().any(char::is_alphanumeric) && contains_search_term(source, value)
}

/// Sépare une description collée dans le libellé (compétences et projets).
///
/// Les petits modèles mettent souvent « Rust — async Tokio » ou « Candilog : app desktop »
/// dans un seul champ `nom`. Sans cette découpe, le grounding échoue sur le libellé
/// complet (absent tel quel du CV) et la description n'existe jamais en propre.
pub fn completer_descriptions_depuis_libelles(profile: &mut Profile) {
    for skill in &mut profile.skills {
        separer_description_du_libelle(&mut skill.name, &mut skill.description);
    }
    for project in &mut profile.projects {
        separer_description_du_libelle(&mut project.name, &mut project.description);
    }
}

fn separer_description_du_libelle(name: &mut String, description: &mut Option<String>) {
    if description
        .as_deref()
        .is_some_and(|texte| !texte.trim().is_empty())
    {
        return;
    }
    let Some((libelle, detail)) = couper_libelle_et_description(name) else {
        return;
    };
    *name = libelle;
    *description = Some(detail);
}

fn couper_libelle_et_description(complet: &str) -> Option<(String, String)> {
    let complet = complet.trim();
    for sep in [" — ", " – ", " - ", " : ", ": "] {
        if let Some((gauche, droite)) = complet.split_once(sep) {
            let gauche = gauche.trim();
            let droite = droite.trim();
            if libelle_court_ok(gauche) && description_ok(droite) {
                return Some((gauche.to_owned(), droite.to_owned()));
            }
        }
    }
    if let Some(open) = complet.rfind('(') {
        if complet.ends_with(')') && open > 0 {
            let gauche = complet[..open].trim();
            let droite = complet[open + 1..complet.len() - 1].trim();
            if libelle_court_ok(gauche) && description_ok(droite) {
                return Some((gauche.to_owned(), droite.to_owned()));
            }
        }
    }
    None
}

fn libelle_court_ok(valeur: &str) -> bool {
    let len = valeur.chars().count();
    (1..=60).contains(&len) && valeur.chars().any(char::is_alphanumeric)
}

fn description_ok(valeur: &str) -> bool {
    let len = valeur.chars().count();
    len >= 8 && valeur.chars().any(char::is_alphanumeric)
}

/// Complète prénom et nom quand l'un des deux (ou les deux) est vide.
///
/// Un petit modèle met souvent le nom complet dans un seul champ. On découpe sur le
/// premier espace. Si les deux restent vides, on tente une ligne d'identité en tête du
/// texte source — utile après un recadrage Vision qui n'a pas touché aux noms.
pub fn completer_identite_noms(source: &str, profile: &mut Profile) {
    decouper_nom_complet(&mut profile.identity);
    if profile.identity.first_name.trim().is_empty() && profile.identity.name.trim().is_empty() {
        if let Some((prenom, nom)) = premier_nom_personne(source) {
            profile.identity.first_name = prenom;
            profile.identity.name = nom;
        }
    }
}

/// Découpe un nom complet coincé dans `first_name` ou `name`.
fn decouper_nom_complet(identity: &mut Identity) {
    let prenom_vide = identity.first_name.trim().is_empty();
    let nom_vide = identity.name.trim().is_empty();
    if prenom_vide == nom_vide {
        return;
    }
    let source = if prenom_vide {
        identity.name.trim()
    } else {
        identity.first_name.trim()
    };
    let Some((prenom, nom)) = couper_prenom_nom(source) else {
        return;
    };
    identity.first_name = prenom;
    identity.name = nom;
}

fn couper_prenom_nom(complet: &str) -> Option<(String, String)> {
    let mut parts = complet.split_whitespace();
    let prenom = parts.next()?.to_owned();
    let reste: Vec<&str> = parts.collect();
    if reste.is_empty() {
        return None;
    }
    Some((prenom, reste.join(" ")))
}

/// Première ligne courte qui ressemble à « Prénom Nom », extraite telle quelle.
fn premier_nom_personne(source: &str) -> Option<(String, String)> {
    for ligne in source.lines() {
        let ligne = ligne.trim();
        if ligne.is_empty()
            || ligne.contains('@')
            || ligne.to_ascii_lowercase().contains("http")
            || ligne.chars().any(|caractere| caractere.is_ascii_digit())
        {
            continue;
        }
        let mots: Vec<&str> = ligne.split_whitespace().collect();
        if !(2..=4).contains(&mots.len()) || ligne.chars().count() > 60 {
            continue;
        }
        if !mots.iter().all(|mot| est_jeton_nom(mot)) {
            continue;
        }
        return couper_prenom_nom(ligne);
    }
    None
}

fn est_jeton_nom(valeur: &str) -> bool {
    let epure = valeur.trim_matches(|caractere: char| {
        matches!(
            caractere,
            ',' | ';' | '.' | ':' | '!' | '?' | '"' | '\'' | '(' | ')'
        )
    });
    !epure.is_empty()
        && epure
            .chars()
            .all(|caractere| caractere.is_alphabetic() || matches!(caractere, '-' | '\'' | '’'))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
