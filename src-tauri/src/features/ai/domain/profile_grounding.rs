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
use crate::features::profile::domain::{Education, Profile};

/// Efface d'un profil importé tout texte absent du CV analysé.
///
/// Les dates ne sont pas concernées : `normalize_profile_dates` les a déjà ramenées au
/// format `AAAA-MM`, que le CV n'écrit pas tel quel (« Oct. 2025 »). Les entrées vidées de
/// leur libellé sont retirées ensuite par le nettoyage par vacuité.
pub fn ground_imported_profile(source: &str, profile: &mut Profile) {
    let identity = &mut profile.identity;
    retenir(source, &mut identity.first_name);
    retenir(source, &mut identity.name);
    retenir(source, &mut identity.email);
    retenir_option(source, &mut identity.phone);
    retenir_option(source, &mut identity.address);
    retenir_option(source, &mut identity.city);
    retenir_option(source, &mut identity.title);
    retenir_option(source, &mut identity.resume);
    retenir_option(source, &mut identity.linkedin);
    retenir_option(source, &mut identity.github);
    retenir_option(source, &mut identity.website);

    for experience in &mut profile.experiences {
        retenir(source, &mut experience.title);
        retenir(source, &mut experience.company);
        retenir_option(source, &mut experience.location);
        retenir_option(source, &mut experience.description);
    }
    for skill in &mut profile.skills {
        retenir(source, &mut skill.name);
    }
    for education in &mut profile.education {
        retenir(source, &mut education.degree);
        retenir(source, &mut education.school);
        retenir_option(source, &mut education.location);
        retenir_option(source, &mut education.description);
    }
    for language in &mut profile.languages {
        retenir(source, &mut language.name);
        retenir(source, &mut language.level);
    }
    for project in &mut profile.projects {
        retenir(source, &mut project.name);
        retenir_option(source, &mut project.description);
        retenir_option(source, &mut project.url);
        retenir_option(source, &mut project.technologies);
    }
    for certification in &mut profile.certifications {
        retenir(source, &mut certification.name);
        retenir_option(source, &mut certification.issuer);
        retenir_option(source, &mut certification.url);
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

/// Une valeur est recopiée si elle porte du texte et apparaît telle quelle dans le CV.
///
/// La recherche exige des frontières alphanumériques : c'est elle qui écarte « .com », que
/// le CV n'écrit qu'accolé à un domaine (`gmail.com`), sans rejeter une valeur légitime.
/// Le garde sur les caractères alphanumériques écarte en amont les valeurs sans contenu
/// (« . »), qu'une ponctuation quelconque du CV suffirait sinon à valider.
fn est_recopie(source: &str, value: &str) -> bool {
    value.chars().any(char::is_alphanumeric) && contains_search_term(source, value)
}

/// Complète l'email et le téléphone vides à partir du texte déjà soumis au modèle.
///
/// Le recadrage n'ajoute rien : un petit modèle laisse souvent ces deux champs vides
/// alors que le texte les contient. On n'écrit qu'un extrait exact, et seulement si
/// le champ est vide, pour ne pas écraser une valeur déjà recopiée.
pub fn completer_contacts_vides(source: &str, profile: &mut Profile) {
    if profile.identity.email.trim().is_empty() {
        if let Some(email) = premier_email(source) {
            profile.identity.email = email.to_owned();
        }
    }
    let telephone_vide = match profile.identity.phone.as_deref() {
        None => true,
        Some(valeur) => valeur.trim().is_empty(),
    };
    if telephone_vide {
        if let Some(telephone) = premier_telephone(source) {
            profile.identity.phone = Some(telephone.to_owned());
        }
    }
}

/// Premier e-mail plausible, extrait tel quel du texte.
///
/// On prend d'abord le premier jeton séparé par des blancs qui est déjà une adresse :
/// c'est un extrait exact, même collé à d'autres chiffres. Sinon on recule autour du
/// premier `@` jusqu'à un extrait encore plausible.
fn premier_email(source: &str) -> Option<&str> {
    if let Some(email) = premier_email_jeton(source) {
        return Some(email);
    }
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'@' {
            if let Some((debut, fin)) = email_autour(source, index) {
                return Some(&source[debut..fin]);
            }
        }
        index += 1;
    }
    None
}

fn premier_email_jeton(source: &str) -> Option<&str> {
    let mut curseur = 0usize;
    for jeton in source.split_whitespace() {
        let relatif = source[curseur..].find(jeton)?;
        let debut = curseur + relatif;
        curseur = debut + jeton.len();
        let epure = jeton.trim_matches(|caractere: char| {
            matches!(caractere, '<' | '>' | '(' | ')' | ',' | ';' | '"' | '\'')
        });
        if epure.is_empty() || !jeton_email_plausible(epure) {
            continue;
        }
        let inner = jeton.find(epure)?;
        return Some(&source[debut + inner..debut + inner + epure.len()]);
    }
    None
}

fn jeton_email_plausible(value: &str) -> bool {
    let mut parts = value.split('@');
    let local = parts.next().unwrap_or("");
    let domaine = parts.next().unwrap_or("");
    parts.next().is_none()
        && !local.is_empty()
        && local
            .chars()
            .any(|caractere| caractere.is_ascii_alphabetic())
        && domaine.contains('.')
        && domaine.chars().all(|caractere| {
            caractere.is_ascii_alphanumeric() || matches!(caractere, '.' | '-' | '_')
        })
        && local.chars().all(|caractere| {
            caractere.is_ascii_alphanumeric() || matches!(caractere, '.' | '_' | '%' | '+' | '-')
        })
        && !domaine.rsplit('.').next().is_some_and(|tld| {
            matches!(
                tld.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf" | "bmp"
            )
        })
}

fn email_autour(source: &str, arobase: usize) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    if arobase == 0 || arobase + 1 >= bytes.len() {
        return None;
    }
    let mut debut = arobase;
    while debut > 0 && est_local(bytes[debut - 1]) {
        debut -= 1;
    }
    let mut fin = arobase + 1;
    while fin < bytes.len() && est_domaine(bytes[fin]) {
        if bytes[fin] == b'.' && (fin + 1 >= bytes.len() || !est_label(bytes[fin + 1])) {
            break;
        }
        fin += 1;
    }
    // Un téléphone ou un suffixe collé au domaine (`nom@example.fr0612`) allonge
    // l'expansion. On recule jusqu'au plus long extrait exact encore plausible.
    while fin > arobase + 1 {
        while fin > arobase + 1 && matches!(bytes[fin - 1], b'.' | b'-') {
            fin -= 1;
        }
        let local = &source[debut..arobase];
        let domaine = &source[arobase + 1..fin];
        if local_ok(local) && domaine_ok(domaine) {
            return Some((debut, fin));
        }
        if fin == arobase + 1 {
            break;
        }
        fin -= 1;
    }
    None
}

fn est_local(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || matches!(octet, b'.' | b'_' | b'%' | b'+' | b'-')
}

fn est_domaine(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || matches!(octet, b'.' | b'-')
}

fn est_label(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || octet == b'-'
}

fn local_ok(local: &str) -> bool {
    !local.is_empty()
        && local
            .chars()
            .any(|caractere| caractere.is_ascii_alphabetic())
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
}

fn domaine_ok(domaine: &str) -> bool {
    let labels: Vec<&str> = domaine.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    if labels.iter().any(|label| {
        label.is_empty()
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .chars()
                .all(|caractere| caractere.is_ascii_alphanumeric() || caractere == '-')
    }) {
        return false;
    }
    let tld = labels[labels.len() - 1];
    tld.len() >= 2
        && tld.chars().all(|caractere| caractere.is_ascii_alphabetic())
        && !matches!(
            tld.to_ascii_lowercase().as_str(),
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "svg"
                | "pdf"
                | "bmp"
                | "tif"
                | "tiff"
                | "css"
                | "js"
                | "html"
                | "htm"
        )
}

/// Premier téléphone français ou international plausible, extrait tel quel.
///
/// On parcourt le texte de gauche à droite. Un numéro national `0X` de dix chiffres
/// et un numéro précédé de `+` ou `00` sont acceptés ; le premier qui l'est gagne.
fn premier_telephone(source: &str) -> Option<&str> {
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let mut index = 0;
    while index < chars.len() {
        if let Some(fin) = essai_telephone(&chars, index) {
            let debut = chars[index].0;
            let fin_octet = if fin < chars.len() {
                chars[fin].0
            } else {
                source.len()
            };
            if !dans_un_email(source, debut, fin_octet) {
                return Some(&source[debut..fin_octet]);
            }
        }
        index += 1;
    }
    None
}

fn essai_telephone(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if precede_chiffre(chars, index) {
        return None;
    }
    match chars.get(index).map(|item| item.1) {
        Some('+') => essai_plus_33(chars, index).or_else(|| essai_plus_generique(chars, index)),
        Some('(') => {
            let suivant = index + 1;
            if chars.get(suivant).map(|item| item.1) == Some('0') {
                essai_national_fr(chars, suivant)
            } else {
                None
            }
        }
        Some('0') => essai_0033(chars, index).or_else(|| essai_national_fr(chars, index)),
        _ => None,
    }
}

fn precede_chiffre(chars: &[(usize, char)], index: usize) -> bool {
    index > 0 && chars[index - 1].1.is_ascii_digit()
}

fn est_sep_telephone(caractere: char) -> bool {
    matches!(
        caractere,
        ' ' | '.' | '-' | '/' | '(' | ')' | '\u{00a0}' | '\u{202f}' | '\u{2009}'
    )
}

fn essai_plus_33(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if chars.get(index).map(|item| item.1) != Some('+') {
        return None;
    }
    let curseur = sauter_seps(chars, index + 1);
    let curseur = lire_literal(chars, curseur, "33")?;
    apres_indicatif_fr(chars, curseur)
}

fn essai_0033(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if chars.get(index).map(|item| item.1) != Some('0') {
        return None;
    }
    let curseur = lire_literal(chars, index, "0033")?;
    apres_indicatif_fr(chars, curseur)
}

fn apres_indicatif_fr(chars: &[(usize, char)], curseur: usize) -> Option<usize> {
    let curseur = sauter_seps(chars, curseur);
    if let Some(apres_tronc) = lire_tronc_parenthese(chars, curseur) {
        let curseur = sauter_seps(chars, apres_tronc);
        let fin = lire_groupes(chars, curseur, 9, true)?;
        return borne_fermee(chars, fin).then_some(fin);
    }
    if chars.get(curseur).map(|item| item.1) == Some('0') {
        return essai_national_fr(chars, curseur);
    }
    let fin = lire_groupes(chars, curseur, 9, true)?;
    borne_fermee(chars, fin).then_some(fin)
}

fn essai_plus_generique(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if chars.get(index).map(|item| item.1) != Some('+') {
        return None;
    }
    let curseur = sauter_seps(chars, index + 1);
    if chars.get(curseur).map(|item| item.1) == Some('3')
        && chars.get(curseur + 1).map(|item| item.1) == Some('3')
    {
        return None;
    }
    let fin = lire_groupes_bornes(chars, curseur, 8, 15, true)?;
    borne_fermee(chars, fin).then_some(fin)
}

fn essai_national_fr(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if chars.get(index).map(|item| item.1) != Some('0') || precede_chiffre(chars, index) {
        return None;
    }
    if chars.get(index + 1).map(|item| item.1) == Some('0') {
        return None;
    }
    let fin = lire_groupes(chars, index, 10, false)?;
    // Un chiffre collé sans séparateur appartient à un identifiant plus long.
    // Un groupe séparé qui suit n'est pas absorbé : l'extrait des dix chiffres reste exact.
    if deuxieme_chiffre_national(chars, index)
        && chars.get(fin).map(|item| item.1.is_ascii_digit()) != Some(true)
    {
        return Some(fin);
    }
    None
}

fn deuxieme_chiffre_national(chars: &[(usize, char)], index: usize) -> bool {
    let mut vus = 0usize;
    let mut curseur = index;
    while curseur < chars.len() && vus < 2 {
        let caractere = chars[curseur].1;
        if caractere.is_ascii_digit() {
            vus += 1;
            if vus == 2 {
                return ('1'..='9').contains(&caractere);
            }
        }
        curseur += 1;
    }
    false
}

fn borne_fermee(chars: &[(usize, char)], fin: usize) -> bool {
    if fin >= chars.len() {
        return true;
    }
    if chars[fin].1.is_ascii_digit() {
        return false;
    }
    if !est_sep_telephone(chars[fin].1) || chars[fin].1 == '/' {
        return true;
    }
    let suivant = fin + 1;
    if suivant < chars.len() && chars[suivant].1.is_ascii_digit() {
        return false;
    }
    true
}

fn sauter_seps(chars: &[(usize, char)], mut curseur: usize) -> usize {
    while curseur < chars.len() && est_sep_telephone(chars[curseur].1) {
        curseur += 1;
    }
    curseur
}

fn lire_literal(chars: &[(usize, char)], mut curseur: usize, attendu: &str) -> Option<usize> {
    for caractere in attendu.chars() {
        if chars.get(curseur).map(|item| item.1) != Some(caractere) {
            return None;
        }
        curseur += 1;
    }
    Some(curseur)
}

fn lire_tronc_parenthese(chars: &[(usize, char)], mut curseur: usize) -> Option<usize> {
    if chars.get(curseur).map(|item| item.1) != Some('(') {
        return None;
    }
    curseur += 1;
    if chars.get(curseur).map(|item| item.1) == Some(' ') {
        curseur += 1;
    }
    if chars.get(curseur).map(|item| item.1) != Some('0') {
        return None;
    }
    curseur += 1;
    if chars.get(curseur).map(|item| item.1) == Some(' ') {
        curseur += 1;
    }
    if chars.get(curseur).map(|item| item.1) != Some(')') {
        return None;
    }
    Some(curseur + 1)
}

fn lire_groupes(
    chars: &[(usize, char)],
    curseur: usize,
    attendu: usize,
    premier_non_zero: bool,
) -> Option<usize> {
    lire_groupes_bornes(chars, curseur, attendu, attendu, premier_non_zero)
}

fn lire_groupes_bornes(
    chars: &[(usize, char)],
    mut curseur: usize,
    min_chiffres: usize,
    max_chiffres: usize,
    premier_non_zero: bool,
) -> Option<usize> {
    let mut compte = 0usize;
    let mut fin = None;
    while curseur < chars.len() && compte < max_chiffres {
        let caractere = chars[curseur].1;
        if caractere.is_ascii_digit() {
            if compte == 0 && premier_non_zero && caractere == '0' {
                return None;
            }
            compte += 1;
            curseur += 1;
            fin = Some(curseur);
            continue;
        }
        if compte == 0 || !est_sep_telephone(caractere) {
            break;
        }
        let mut avance = curseur;
        let mut seps = 0usize;
        while avance < chars.len() && est_sep_telephone(chars[avance].1) && seps < 2 {
            seps += 1;
            avance += 1;
        }
        if avance < chars.len() && chars[avance].1.is_ascii_digit() && compte < max_chiffres {
            curseur = avance;
            continue;
        }
        break;
    }
    if compte < min_chiffres {
        return None;
    }
    fin
}

fn dans_un_email(source: &str, debut: usize, fin: usize) -> bool {
    source
        .get(debut..fin)
        .is_some_and(|extrait| extrait.contains('@'))
}

/// Ajoute une formation manquante après le recadrage, sans second appel au modèle.
///
/// Le score compte des lignes de formation face aux indices de diplôme du CV tronqué.
/// On n'ajoute une ligne qu'en queue, pour un indice non déjà couvert, et seulement si
/// la fenêtre contient exactement un diplôme ou une école recopié tel quel. Une fenêtre
/// ambiguë est ignorée. Les formations déjà émises ne sont ni réécrites ni réordonnées.
pub fn completer_formations_manquantes(source: &str, profile: &mut Profile) {
    let attendu = compter_indices_diplome(source);
    if attendu == 0 || profile.education.len() >= attendu {
        return;
    }
    let occurrences = occurrences_diplome(source);
    let mut couverts = vec![false; profile.education.len()];
    for occurrence in &occurrences {
        if profile.education.len() >= attendu {
            break;
        }
        if indice_deja_couvert(profile, &mut couverts, &occurrence.indice) {
            continue;
        }
        let Some((diplome, ecole)) = formation_unique(source, &occurrences, occurrence) else {
            continue;
        };
        if diplome.is_empty() && ecole.is_empty() {
            continue;
        }
        if formation_deja_emise(profile, &diplome, &ecole) {
            continue;
        }
        profile.education.push(Education {
            degree: diplome,
            school: ecole,
            ..Education::default()
        });
        couverts.push(true);
    }
}

const INDICES_DIPLOME: [&str; 19] = [
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

const INDICES_REPES: [&str; 11] = [
    "master",
    "licence",
    "bachelor",
    "bts",
    "dut",
    "mba",
    "doctorat",
    "phd",
    "ingénieur",
    "ingenieur",
    "msc",
];

struct OccurrenceDiplome {
    debut: usize,
    fin: usize,
    indice: String,
}

fn compter_indices_diplome(source: &str) -> usize {
    let jetons = jetons_diplome(source);
    let vus = INDICES_DIPLOME
        .iter()
        .filter(|indice| jetons.iter().any(|jeton| jeton == *indice))
        .count();
    let repetes = jetons
        .iter()
        .filter(|jeton| INDICES_REPES.contains(&jeton.as_str()))
        .count();
    repetes.max(vus.min(6))
}

fn jetons_diplome(source: &str) -> Vec<String> {
    let minuscule = source.to_lowercase();
    let espaces = minuscule.replace(['\n', '\t', '/', ',', ';', ':', '(', ')', '[', ']'], " ");
    espaces
        .split_whitespace()
        .map(|jeton| {
            jeton
                .trim_matches(|caractere: char| !caractere.is_alphanumeric())
                .to_owned()
        })
        .filter(|jeton| !jeton.is_empty())
        .collect()
}

fn occurrences_diplome(source: &str) -> Vec<OccurrenceDiplome> {
    let mut occurrences = Vec::new();
    let mut debut_mot = None;
    for (index, caractere) in source.char_indices() {
        if caractere.is_alphanumeric() || caractere == '\'' || caractere == '’' {
            debut_mot.get_or_insert(index);
            continue;
        }
        if let Some(debut) = debut_mot.take() {
            retenir_indice(source, debut, index, &mut occurrences);
        }
    }
    if let Some(debut) = debut_mot {
        retenir_indice(source, debut, source.len(), &mut occurrences);
    }
    occurrences
}

fn retenir_indice(
    source: &str,
    debut: usize,
    fin: usize,
    occurrences: &mut Vec<OccurrenceDiplome>,
) {
    let Some(brut) = source.get(debut..fin) else {
        return;
    };
    let epure = brut_indice(brut);
    if INDICES_DIPLOME.contains(&epure.as_str()) {
        let relatif = brut.find(epure.as_str()).unwrap_or(0);
        let copie_debut = debut + relatif;
        occurrences.push(OccurrenceDiplome {
            debut: copie_debut,
            fin: copie_debut + epure.len(),
            indice: epure,
        });
    }
}

fn brut_indice(brut: &str) -> String {
    brut.trim_matches(|caractere: char| !caractere.is_alphanumeric())
        .to_lowercase()
}

fn indice_deja_couvert(profile: &Profile, couverts: &mut [bool], indice: &str) -> bool {
    for (index, formation) in profile.education.iter().enumerate() {
        if couverts.get(index).copied().unwrap_or(false) {
            continue;
        }
        if contient_indice(&formation.degree, indice) || contient_indice(&formation.school, indice)
        {
            couverts[index] = true;
            return true;
        }
    }
    false
}

fn contient_indice(valeur: &str, indice: &str) -> bool {
    jetons_diplome(valeur).iter().any(|jeton| jeton == indice)
}

fn formation_deja_emise(profile: &Profile, diplome: &str, ecole: &str) -> bool {
    let diplome = diplome.to_lowercase();
    let ecole = ecole.to_lowercase();
    profile.education.iter().any(|formation| {
        !diplome.is_empty() && formation.degree.to_lowercase() == diplome
            || !ecole.is_empty() && formation.school.to_lowercase() == ecole && diplome.is_empty()
    })
}

fn formation_unique(
    source: &str,
    occurrences: &[OccurrenceDiplome],
    occurrence: &OccurrenceDiplome,
) -> Option<(String, String)> {
    if est_intitule_de_poste(source, occurrence) {
        return None;
    }
    let (fenetre_debut, fenetre_fin) = fenetre_occurrence(source, occurrences, occurrence);
    let texte = source.get(fenetre_debut..fenetre_fin)?;
    if texte.contains('…') {
        return None;
    }
    let mut diplomes = Vec::new();
    let mut ecoles = Vec::new();
    for ligne in lignes_de(source, fenetre_debut, fenetre_fin) {
        if est_entete_formation(ligne) {
            continue;
        }
        for fragment in fragments_formation(ligne) {
            if fragment.chars().count() > 72 {
                continue;
            }
            let jetons = jetons_diplome(fragment);
            if jetons.iter().any(|jeton| jeton == &occurrence.indice) {
                if jetons
                    .iter()
                    .filter(|jeton| INDICES_DIPLOME.contains(&jeton.as_str()))
                    .count()
                    == 1
                    && fragment.split_whitespace().count() <= 8
                    && !phrase_narrative(fragment)
                {
                    pousser_unique(&mut diplomes, fragment);
                }
                continue;
            }
            if a_marqueur_ecole(fragment) && fragment.split_whitespace().count() <= 8 {
                pousser_unique(&mut ecoles, fragment);
            }
        }
    }
    if diplomes.len() > 1 || ecoles.len() > 1 {
        return None;
    }
    let diplome = diplomes.first().copied().unwrap_or("").trim();
    let ecole = ecoles.first().copied().unwrap_or("").trim();
    if diplome.is_empty() && ecole.is_empty() {
        return None;
    }
    if !diplome.is_empty() && !source.contains(diplome) {
        return None;
    }
    if !ecole.is_empty() && !source.contains(ecole) {
        return None;
    }
    Some((diplome.to_owned(), ecole.to_owned()))
}

fn est_intitule_de_poste(source: &str, occurrence: &OccurrenceDiplome) -> bool {
    if !matches!(occurrence.indice.as_str(), "ingénieur" | "ingenieur") {
        return false;
    }
    let ligne = ligne_contenant(source, occurrence.debut).unwrap_or("");
    if a_marqueur_ecole(ligne) || ligne.to_lowercase().contains("dipl") {
        return false;
    }
    let autour = contexte_lignes(source, occurrence.debut, 1);
    autour.contains("201") || autour.contains("199") || autour.contains("202")
}

fn fenetre_occurrence(
    source: &str,
    occurrences: &[OccurrenceDiplome],
    occurrence: &OccurrenceDiplome,
) -> (usize, usize) {
    let precedente = occurrences
        .iter()
        .rev()
        .find(|autre| autre.debut < occurrence.debut)
        .map(|autre| autre.fin);
    let suivante = occurrences
        .iter()
        .find(|autre| autre.debut > occurrence.debut)
        .map(|autre| autre.debut);
    let ligne = ligne_contenant(source, occurrence.debut).unwrap_or("");
    let ligne_debut = source.find(ligne).unwrap_or(occurrence.debut);
    let mut debut = ligne_debut.max(precedente.unwrap_or(0));
    let mut fin = ligne_debut + ligne.len();
    if let Some(suivante) = suivante {
        fin = fin.min(suivante);
    }
    // L'école suit souvent le diplôme. La ligne d'avant appartient au diplôme précédent.
    if precedente.is_none() {
        if let Some(avant) = ligne_precedente(source, occurrence.debut) {
            let avant_debut = source.find(avant).unwrap_or(debut);
            if !occurrences_dans(occurrences, avant_debut, occurrence.debut) {
                debut = debut.min(avant_debut);
            }
        }
    }
    if let Some(apres) = ligne_suivante(source, occurrence.debut) {
        let apres_debut = source[occurrence.fin..]
            .find(apres)
            .map(|relatif| occurrence.fin + relatif)
            .unwrap_or(fin);
        if suivante.is_none_or(|borne| apres_debut < borne)
            && !occurrences_dans(occurrences, fin, apres_debut.saturating_add(apres.len()))
        {
            fin = fin.max((apres_debut + apres.len()).min(suivante.unwrap_or(source.len())));
        }
    }
    (
        debut.min(occurrence.debut),
        fin.max(occurrence.fin).min(source.len()),
    )
}

fn occurrences_dans(occurrences: &[OccurrenceDiplome], debut: usize, fin: usize) -> bool {
    occurrences
        .iter()
        .any(|occurrence| occurrence.debut >= debut && occurrence.debut < fin)
}

fn lignes_de(source: &str, debut: usize, fin: usize) -> Vec<&str> {
    source
        .get(debut..fin)
        .unwrap_or("")
        .split('\n')
        .map(str::trim)
        .filter(|ligne| !ligne.is_empty())
        .collect()
}

fn fragments_formation(ligne: &str) -> Vec<&str> {
    let mut morceaux = vec![ligne];
    for separateur in [" | ", " - ", " – ", " — ", " · ", " • ", "|"] {
        let mut suivant = Vec::new();
        for morceau in morceaux {
            if morceau.contains(separateur) {
                suivant.extend(
                    morceau
                        .split(separateur)
                        .map(str::trim)
                        .filter(|part| !part.is_empty()),
                );
            } else {
                suivant.push(morceau);
            }
        }
        morceaux = suivant;
    }
    morceaux
}

fn pousser_unique<'a>(liste: &mut Vec<&'a str>, valeur: &'a str) {
    let epure = valeur.trim();
    if epure.is_empty() {
        return;
    }
    if !liste.iter().any(|autre| autre.eq_ignore_ascii_case(epure)) {
        liste.push(epure);
    }
}

fn phrase_narrative(fragment: &str) -> bool {
    let minuscule = fragment.to_lowercase();
    [" puis ", " avec ", " pour ", " dans ", " voir ", " suivi "]
        .iter()
        .any(|mot| minuscule.contains(mot))
}

fn a_marqueur_ecole(fragment: &str) -> bool {
    let jetons = jetons_diplome(fragment);
    const MARQUEURS: [&str; 12] = [
        "université",
        "universite",
        "university",
        "école",
        "ecole",
        "institut",
        "iut",
        "lycée",
        "lycee",
        "college",
        "campus",
        "school",
    ];
    jetons
        .iter()
        .any(|jeton| MARQUEURS.contains(&jeton.as_str()))
}

fn est_entete_formation(ligne: &str) -> bool {
    let jetons = jetons_diplome(ligne);
    if jetons.len() > 3 {
        return false;
    }
    const ENTETES: [&str; 6] = [
        "formation",
        "formations",
        "education",
        "etudes",
        "diplomes",
        "scolarite",
    ];
    jetons
        .iter()
        .all(|jeton| ENTETES.contains(&jeton.as_str()) || jeton == "diplôme" || jeton == "diplome")
        && jetons.iter().any(|jeton| {
            ENTETES.contains(&jeton.as_str())
                || jeton == "diplômes"
                || jeton == "diplome"
                || jeton == "diplôme"
        })
}

fn ligne_contenant(source: &str, offset: usize) -> Option<&str> {
    source.lines().find(|ligne| {
        source
            .find(ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    })
}

fn contexte_lignes(source: &str, offset: usize, rayon: usize) -> String {
    let lignes: Vec<&str> = source.lines().collect();
    let index = lignes.iter().position(|ligne| {
        source
            .find(ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    });
    let Some(index) = index else {
        return String::new();
    };
    let debut = index.saturating_sub(rayon);
    let fin = (index + rayon + 1).min(lignes.len());
    lignes[debut..fin].join("\n")
}

fn ligne_precedente(source: &str, offset: usize) -> Option<&str> {
    let lignes: Vec<&str> = source
        .lines()
        .filter(|ligne| !ligne.trim().is_empty())
        .collect();
    let index = lignes.iter().position(|ligne| {
        source
            .find(*ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    })?;
    index.checked_sub(1).map(|index| lignes[index])
}

fn ligne_suivante(source: &str, offset: usize) -> Option<&str> {
    let lignes: Vec<&str> = source
        .lines()
        .filter(|ligne| !ligne.trim().is_empty())
        .collect();
    let index = lignes.iter().position(|ligne| {
        source
            .find(*ligne)
            .is_some_and(|debut| offset >= debut && offset < debut + ligne.len())
    })?;
    lignes.get(index + 1).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profile::domain::{
        Certification, Education, Experience, Identity, Language, Project, Skill,
    };

    /// Extrait du CV réellement analysé, tel que le lecteur PDF le restitue.
    const CV: &str = "Alexandre Bouttier\n\n\
Technicien Supérieur Systèmes et Réseaux (TSSR) · Recherche contrat de professionnalisation\n\n\
Admis en formation TSSR à l'ENI de Chartres-de-Bretagne\n\n\
07 86 66 46 99 34 ans alexandrebouttier@gmail.com Saint-Jacques-de-la-Lande (35)\n\n\
linkedin.com/in/alexandrebouttier ↗ alexandrebouttier.fr ↗\n\n\
Projet professionnel · Technicien Supérieur Systèmes et Réseaux : admis en formation TSSR\n\
à l'ENI ; autoformation et mise en pratique sur un serveur VPS\n\n\
Oct. 2025 – Sept. 2026\n\n\
Projet personnel · entretienmx.fr ↗· OVH · Ubuntu Server\n\n\
Français · langue maternelle\n\n\
Anglais · lecture courante de documentation technique\n";

    /// Les fragments observés dans la réponse réelle de `maternion/lfm2.5:350m`.
    #[test]
    fn efface_les_fragments_de_domaine_inventes_par_le_modele() {
        let mut profile = Profile {
            languages: vec![Language {
                name: ".com".into(),
                level: ".com".into(),
            }],
            projects: vec![Project {
                name: "Projet personnel · entretienmx.fr".into(),
                url: Some(".fr".into()),
                ..Project::default()
            }],
            certifications: vec![Certification {
                name: ".org".into(),
                url: Some(".com".into()),
                ..Certification::default()
            }],
            ..Profile::default()
        };

        ground_imported_profile(CV, &mut profile);

        assert_eq!(profile.languages[0].name, "");
        assert_eq!(profile.languages[0].level, "");
        assert_eq!(
            profile.projects[0].name,
            "Projet personnel · entretienmx.fr"
        );
        assert_eq!(profile.projects[0].url, None);
        assert_eq!(profile.certifications[0].name, "");
        assert_eq!(profile.certifications[0].url, None);
    }

    #[test]
    fn efface_les_morceaux_de_mots_et_la_certification_absente_du_cv() {
        let mut profile = Profile {
            experiences: vec![Experience {
                title: "Projet professionnel".into(),
                company: "ENI".into(),
                location: Some(".chartres-de-bretagne".into()),
                ..Experience::default()
            }],
            languages: vec![
                Language {
                    name: ".franc".into(),
                    level: "Bac".into(),
                },
                Language {
                    name: ".anglais".into(),
                    level: "Anglais".into(),
                },
            ],
            projects: vec![Project {
                name: "Projet personnel".into(),
                url: Some(".github".into()),
                ..Project::default()
            }],
            certifications: vec![Certification {
                name: ".enf".into(),
                issuer: Some(".chartres-de-bretagne".into()),
                url: Some(".".into()),
                ..Certification::default()
            }],
            ..Profile::default()
        };

        ground_imported_profile(CV, &mut profile);

        // L'expérience reste, seul le lieu inventé disparaît.
        assert_eq!(profile.experiences[0].title, "Projet professionnel");
        assert_eq!(profile.experiences[0].company, "ENI");
        assert_eq!(profile.experiences[0].location, None);
        assert_eq!(profile.languages[0].name, "");
        assert_eq!(profile.languages[1].name, "");
        assert_eq!(profile.projects[0].url, None);
        assert_eq!(profile.certifications[0].name, "");
        assert_eq!(profile.certifications[0].issuer, None);
        assert_eq!(profile.certifications[0].url, None);
    }

    #[test]
    fn conserve_les_valeurs_recopiees_du_cv() {
        let mut profile = Profile {
            identity: Identity {
                first_name: "Alexandre".into(),
                name: "Bouttier".into(),
                email: "alexandrebouttier@gmail.com".into(),
                phone: Some("07 86 66 46 99".into()),
                city: Some("Chartres-de-Bretagne".into()),
                title: Some("Technicien Supérieur Systèmes et Réseaux".into()),
                resume: Some("Recherche contrat de professionnalisation".into()),
                linkedin: Some("linkedin.com/in/alexandrebouttier".into()),
                website: Some("alexandrebouttier.fr".into()),
                ..Identity::default()
            },
            skills: vec![Skill {
                name: "Ubuntu Server".into(),
            }],
            languages: vec![Language {
                name: "Français".into(),
                level: "langue maternelle".into(),
            }],
            ..Profile::default()
        };
        let attendu = profile.clone();

        ground_imported_profile(CV, &mut profile);

        assert_eq!(profile, attendu);
    }

    /// La comparaison ignore la casse et les accents, comme partout ailleurs.
    #[test]
    fn accepte_une_recopie_a_la_casse_ou_aux_accents_pres() {
        let mut profile = Profile {
            languages: vec![Language {
                name: "FRANCAIS".into(),
                level: "Langue Maternelle".into(),
            }],
            ..Profile::default()
        };

        ground_imported_profile(CV, &mut profile);

        assert_eq!(profile.languages[0].name, "FRANCAIS");
        assert_eq!(profile.languages[0].level, "Langue Maternelle");
    }

    /// Une date déjà reformatée ne figure pas telle quelle dans le CV : la recopier serait
    /// impossible, et le recadrage ne doit pas y toucher.
    #[test]
    fn ne_touche_pas_aux_dates_deja_normalisees() {
        let mut profile = Profile {
            experiences: vec![Experience {
                title: "Projet professionnel".into(),
                company: "ENI".into(),
                start_date: "2025-10".into(),
                end_date: Some("2026-09".into()),
                ..Experience::default()
            }],
            ..Profile::default()
        };

        ground_imported_profile(CV, &mut profile);

        assert_eq!(profile.experiences[0].start_date, "2025-10");
        assert_eq!(profile.experiences[0].end_date.as_deref(), Some("2026-09"));
    }

    fn identite_vide() -> Profile {
        Profile::default()
    }

    #[test]
    fn complete_email_et_telephone_vides_par_un_extrait_exact() {
        let source = "Camille Martin\n06 12 34 56 78\ncamille.martin@example.fr\nRennes";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.email, "camille.martin@example.fr");
        assert_eq!(profile.identity.phone.as_deref(), Some("06 12 34 56 78"));
        assert!(source.contains(&profile.identity.email));
        assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
    }

    #[test]
    fn ne_recouvre_pas_un_email_ou_telephone_deja_recopie() {
        let source = "Nadia Leroy\nnadia.leroy@example.org\n+33 6 11 22 33 44\n06 99 88 77 66";
        let mut profile = Profile {
            identity: Identity {
                email: "nadia.leroy@example.org".into(),
                phone: Some("+33 6 11 22 33 44".into()),
                ..Identity::default()
            },
            ..Profile::default()
        };

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.email, "nadia.leroy@example.org");
        assert_eq!(profile.identity.phone.as_deref(), Some("+33 6 11 22 33 44"));
    }

    #[test]
    fn n_invente_pas_un_contact_absent_du_texte() {
        let source = "Ines Bernard\nDeveloppeuse\n2019-2024\n75002 Paris";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.email, "");
        assert_eq!(profile.identity.phone, None);
    }

    #[test]
    fn prend_le_premier_email_et_le_premier_telephone_plausible() {
        let source = "Contact 01/02/2020 puis 06.12.34.56.78 puis 07-11-22-33-44\na@b et leo.martin+job@example.co.uk puis autre.personne@example.net";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.email, "leo.martin+job@example.co.uk");
        assert_eq!(profile.identity.phone.as_deref(), Some("06.12.34.56.78"));
    }

    #[test]
    fn accepte_le_format_international_francais_exact() {
        let source = "Portable +33 (0)6 12 34 56 78 et aussi 0033 7 00 11 22 33";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(
            profile.identity.phone.as_deref(),
            Some("+33 (0)6 12 34 56 78")
        );
        assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
    }

    #[test]
    fn accepte_un_indicatif_international_hors_france() {
        let source = "Bureau +44 20 7946 0958\nhello.world@example.com";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.phone.as_deref(), Some("+44 20 7946 0958"));
        assert_eq!(profile.identity.email, "hello.world@example.com");
    }

    #[test]
    fn ignore_un_numero_colle_a_une_suite_de_chiffres_et_un_email_de_fichier() {
        let source = "Ref 061234567890123 logo@2x.png\n0033 6 12 34 56 78\nmaya.rossi@example.io";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(
            profile.identity.phone.as_deref(),
            Some("0033 6 12 34 56 78")
        );
        assert_eq!(profile.identity.email, "maya.rossi@example.io");
    }

    #[test]
    fn remplit_un_telephone_vide_sans_ecraser_l_email_et_inversement() {
        let source = "Sam Ortega sam.ortega@example.net 07 45 67 89 01";
        let mut email_seul = Profile {
            identity: Identity {
                email: "sam.ortega@example.net".into(),
                phone: None,
                ..Identity::default()
            },
            ..Profile::default()
        };
        completer_contacts_vides(source, &mut email_seul);
        assert_eq!(email_seul.identity.email, "sam.ortega@example.net");
        assert_eq!(email_seul.identity.phone.as_deref(), Some("07 45 67 89 01"));

        let mut tel_seul = Profile {
            identity: Identity {
                email: "   ".into(),
                phone: Some("07 45 67 89 01".into()),
                ..Identity::default()
            },
            ..Profile::default()
        };
        completer_contacts_vides(source, &mut tel_seul);
        assert_eq!(tel_seul.identity.email, "sam.ortega@example.net");
        assert_eq!(tel_seul.identity.phone.as_deref(), Some("07 45 67 89 01"));
    }

    #[test]
    fn separe_un_email_colle_a_un_telephone() {
        let source = "Ref 12lea.martin@example.fr0612345678 et 06 12 34 56 78 90";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.email, "12lea.martin@example.fr0612345678");
        assert_eq!(profile.identity.phone.as_deref(), Some("0612345678"));
        assert!(source.contains(profile.identity.email.as_str()));
        assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
    }

    #[test]
    fn accepte_un_telephone_entre_parentheses() {
        let source = "Ligne (07) 45 67 89 01 — ines.bernard@example.com";
        let mut profile = identite_vide();

        completer_contacts_vides(source, &mut profile);

        assert_eq!(profile.identity.phone.as_deref(), Some("(07) 45 67 89 01"));
        assert!(source.contains(profile.identity.phone.as_deref().unwrap()));
    }

    #[test]
    fn ajoute_un_diplome_unique_sans_ecraser_la_formation_existante() {
        let source = "\
Formations\n\
Licence Histoire\n\
Nébula Université\n\
\n\
Master Informatique\n\
Orion Institut\n";
        let mut profile = Profile {
            education: vec![Education {
                degree: "Licence Histoire".into(),
                school: "Nébula Université".into(),
                ..Education::default()
            }],
            ..Profile::default()
        };
        let avant = profile.education[0].clone();

        completer_formations_manquantes(source, &mut profile);

        assert_eq!(profile.education.len(), 2);
        assert_eq!(profile.education[0], avant);
        assert_eq!(profile.education[1].degree, "Master Informatique");
        assert!(source.contains(profile.education[1].degree.as_str()));
        assert_eq!(profile.education[1].school, "Orion Institut");
        assert!(source.contains(profile.education[1].school.as_str()));
    }

    #[test]
    fn ignore_une_fenetre_de_formation_ambigue() {
        let source = "\
Formations\n\
J'ai suivi un master puis une autre voie, voir Orion Institut et Pétale École.\n";
        let mut profile = Profile::default();

        completer_formations_manquantes(source, &mut profile);

        assert!(profile.education.is_empty());
    }

    #[test]
    fn ne_duplique_pas_un_diplome_deja_couvert() {
        let source = "Master Informatique\nOrion Institut\n";
        let mut profile = Profile {
            education: vec![Education {
                degree: "Master Informatique".into(),
                school: "Orion Institut".into(),
                ..Education::default()
            }],
            ..Profile::default()
        };

        completer_formations_manquantes(source, &mut profile);

        assert_eq!(profile.education.len(), 1);
        assert_eq!(profile.education[0].degree, "Master Informatique");
        assert_eq!(profile.education[0].school, "Orion Institut");
    }

    #[test]
    fn ajoute_plusieurs_formations_sans_paniquer_sur_la_couverture() {
        let source = "Formations\nLicence Histoire\nMaster Informatique\n";
        let mut profile = Profile::default();

        completer_formations_manquantes(source, &mut profile);

        assert_eq!(profile.education.len(), 2);
        assert!(profile
            .education
            .iter()
            .any(|formation| formation.degree == "Licence Histoire"));
        assert!(profile
            .education
            .iter()
            .any(|formation| formation.degree == "Master Informatique"));
    }
}
