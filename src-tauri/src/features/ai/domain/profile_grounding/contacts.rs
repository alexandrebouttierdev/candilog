//! Courriel et téléphone retrouvés dans le CV quand le modèle les a omis.

use super::*;

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
pub(super) fn premier_email(source: &str) -> Option<&str> {
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

pub(super) fn premier_email_jeton(source: &str) -> Option<&str> {
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

pub(super) fn jeton_email_plausible(value: &str) -> bool {
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

pub(super) fn email_autour(source: &str, arobase: usize) -> Option<(usize, usize)> {
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

pub(super) fn est_local(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || matches!(octet, b'.' | b'_' | b'%' | b'+' | b'-')
}

pub(super) fn est_domaine(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || matches!(octet, b'.' | b'-')
}

pub(super) fn est_label(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || octet == b'-'
}

pub(super) fn local_ok(local: &str) -> bool {
    !local.is_empty()
        && local
            .chars()
            .any(|caractere| caractere.is_ascii_alphabetic())
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
}

pub(super) fn domaine_ok(domaine: &str) -> bool {
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
pub(super) fn premier_telephone(source: &str) -> Option<&str> {
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

pub(super) fn essai_telephone(chars: &[(usize, char)], index: usize) -> Option<usize> {
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

pub(super) fn precede_chiffre(chars: &[(usize, char)], index: usize) -> bool {
    index > 0 && chars[index - 1].1.is_ascii_digit()
}

pub(super) fn est_sep_telephone(caractere: char) -> bool {
    matches!(
        caractere,
        ' ' | '.' | '-' | '/' | '(' | ')' | '\u{00a0}' | '\u{202f}' | '\u{2009}'
    )
}

pub(super) fn essai_plus_33(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if chars.get(index).map(|item| item.1) != Some('+') {
        return None;
    }
    let curseur = sauter_seps(chars, index + 1);
    let curseur = lire_literal(chars, curseur, "33")?;
    apres_indicatif_fr(chars, curseur)
}

pub(super) fn essai_0033(chars: &[(usize, char)], index: usize) -> Option<usize> {
    if chars.get(index).map(|item| item.1) != Some('0') {
        return None;
    }
    let curseur = lire_literal(chars, index, "0033")?;
    apres_indicatif_fr(chars, curseur)
}

pub(super) fn apres_indicatif_fr(chars: &[(usize, char)], curseur: usize) -> Option<usize> {
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

pub(super) fn essai_plus_generique(chars: &[(usize, char)], index: usize) -> Option<usize> {
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

pub(super) fn essai_national_fr(chars: &[(usize, char)], index: usize) -> Option<usize> {
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

pub(super) fn deuxieme_chiffre_national(chars: &[(usize, char)], index: usize) -> bool {
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

pub(super) fn borne_fermee(chars: &[(usize, char)], fin: usize) -> bool {
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

pub(super) fn sauter_seps(chars: &[(usize, char)], mut curseur: usize) -> usize {
    while curseur < chars.len() && est_sep_telephone(chars[curseur].1) {
        curseur += 1;
    }
    curseur
}

pub(super) fn lire_literal(
    chars: &[(usize, char)],
    mut curseur: usize,
    attendu: &str,
) -> Option<usize> {
    for caractere in attendu.chars() {
        if chars.get(curseur).map(|item| item.1) != Some(caractere) {
            return None;
        }
        curseur += 1;
    }
    Some(curseur)
}

pub(super) fn lire_tronc_parenthese(chars: &[(usize, char)], mut curseur: usize) -> Option<usize> {
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

pub(super) fn lire_groupes(
    chars: &[(usize, char)],
    curseur: usize,
    attendu: usize,
    premier_non_zero: bool,
) -> Option<usize> {
    lire_groupes_bornes(chars, curseur, attendu, attendu, premier_non_zero)
}

pub(super) fn lire_groupes_bornes(
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

pub(super) fn dans_un_email(source: &str, debut: usize, fin: usize) -> bool {
    source
        .get(debut..fin)
        .is_some_and(|extrait| extrait.contains('@'))
}
