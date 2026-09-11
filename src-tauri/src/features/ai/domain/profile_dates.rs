//! Remise au format des dates d'un profil extrait d'un CV par un modèle.
//!
//! Le prompt demande `AAAA-MM` ou `AAAA`, et les modèles distants s'y tiennent. Les petits
//! modèles locaux, eux, recopient le CV : « Juil. 2019 », « 12/2023 », « 2022-03-15 »,
//! « aujourd'hui », parfois une plage entière dans `start_date` (« juin 2020 – déc. 2022 »)
//! et un fragment sans valeur (« en », « sept »). Ces réponses sont exploitables — l'écran
//! de revue affiche les dates dans un champ libre — à condition de les ramener au format
//! attendu plutôt que de rejeter toute l'analyse.

use crate::core::utils::text::search_key;
use crate::features::profile::domain::Profile;

/// Mois reconnus, en français et en anglais, forme complète et abrégée usuelle.
///
/// La comparaison porte sur des mots entiers d'une clé sans accent ni majuscule : « aout »
/// couvre « Août », « sept » couvre « Sept. », et « mai » ne peut pas capturer « mail ».
const MONTHS: [(&str, u32); 41] = [
    ("janvier", 1),
    ("janv", 1),
    ("jan", 1),
    ("january", 1),
    ("fevrier", 2),
    ("fevr", 2),
    ("fev", 2),
    ("february", 2),
    ("feb", 2),
    ("mars", 3),
    ("march", 3),
    ("mar", 3),
    ("avril", 4),
    ("avr", 4),
    ("april", 4),
    ("apr", 4),
    ("mai", 5),
    ("may", 5),
    ("juin", 6),
    ("june", 6),
    ("jun", 6),
    ("juillet", 7),
    ("juil", 7),
    ("july", 7),
    ("jul", 7),
    ("aout", 8),
    ("august", 8),
    ("aug", 8),
    ("septembre", 9),
    ("september", 9),
    ("sept", 9),
    ("sep", 9),
    ("octobre", 10),
    ("october", 10),
    ("oct", 10),
    ("novembre", 11),
    ("november", 11),
    ("nov", 11),
    ("decembre", 12),
    ("december", 12),
    ("dec", 12),
];

/// Formulations qui désignent une période encore en cours, cherchées dans la clé.
const PRESENT_MARKERS: [&str; 8] = [
    "aujourd",
    "present",
    "en cours",
    "a ce jour",
    "ce jour",
    "actuel",
    "current",
    "ongoing",
];

/// Mêmes formulations, mais trop courtes pour être cherchées à l'intérieur d'un mot.
const PRESENT_EXACT: [&str; 3] = ["now", "today", "maintenant"];

/// Séparateurs de plage courants sur un CV. Les tirets ASCII isolés (`2020-06`) ne
/// figurent pas : seuls les tirets encadrés d'espaces, les tirets typographiques et les
/// formulations « au / à / to » séparent réellement début et fin.
const RANGE_SEPARATORS: [&str; 14] = [
    " – ", " — ", " - ", " –", "– ", " —", "— ", "–", "—", " au ", " à ", " to ", " until ", " / ",
];

/// Années plausibles dans un CV : au-delà, le nombre à quatre chiffres est autre chose.
const YEARS: std::ops::RangeInclusive<u32> = 1900..=2100;

/// Ramène chaque date du profil à `AAAA-MM` ou `AAAA`, et vide ce qui reste indéchiffrable.
///
/// Une date perdue se corrige en deux secondes dans l'écran de revue ; une analyse refusée
/// coûte une nouvelle passe complète du modèle.
pub fn normalize_profile_dates(profile: &mut Profile) {
    for experience in &mut profile.experiences {
        decouper_plage_experience(experience);
        experience.start_date = normalize_date(&experience.start_date).unwrap_or_default();
        // « Juil. 2019 – aujourd'hui » décrit un poste occupé : l'information est dans le
        // CV, elle appartient à `current` et non à une date de fin qu'on jetterait.
        match experience.end_date.take() {
            Some(end) if is_present(&end) => experience.current = true,
            Some(end) => experience.end_date = normalize_date(&end),
            None => {}
        }
        if experience.current {
            experience.end_date = None;
        }
    }
    for education in &mut profile.education {
        if education.end_date.is_none() {
            if let Some((start, end)) = education.start_date.as_deref().and_then(split_date_range) {
                education.start_date = Some(start);
                education.end_date = Some(end);
            }
        }
        education.start_date = education
            .start_date
            .take()
            .and_then(|date| normalize_date(&date));
        education.end_date = education
            .end_date
            .take()
            .filter(|date| !is_present(date))
            .and_then(|date| normalize_date(&date));
    }
    for certification in &mut profile.certifications {
        certification.date = certification
            .date
            .take()
            .and_then(|date| normalize_date(&date));
    }
}

/// Sépare une plage collée dans `start_date` quand `end_date` est encore vide.
fn decouper_plage_experience(experience: &mut crate::features::profile::domain::Experience) {
    if experience.end_date.is_some() {
        return;
    }
    let Some((start, end)) = split_date_range(&experience.start_date) else {
        return;
    };
    experience.start_date = start;
    experience.end_date = Some(end);
}

/// Coupe une plage « début – fin » en deux morceaux bruts, sans encore les normaliser.
fn split_date_range(value: &str) -> Option<(String, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    for separator in RANGE_SEPARATORS {
        if let Some((left, right)) = trimmed.split_once(separator) {
            let left = left.trim();
            let right = right.trim();
            if left.is_empty() || right.is_empty() {
                continue;
            }
            // Les deux côtés doivent porter une année ou un marqueur « en cours » :
            // sinon on a coupé un mot composé ou une date ISO.
            if looks_like_date_side(left) && looks_like_date_side(right) {
                return Some((left.to_owned(), right.to_owned()));
            }
        }
    }
    None
}

fn looks_like_date_side(value: &str) -> bool {
    is_present(value)
        || digit_groups(&search_key(value))
            .iter()
            .any(|group| group.len() == 4 && group.parse().is_ok_and(|year| YEARS.contains(&year)))
}

/// Interprète une date écrite librement, `None` si aucune année n'y figure.
fn normalize_date(value: &str) -> Option<String> {
    let key = search_key(value);
    let groups = digit_groups(&key);
    let year_index = groups.iter().position(|group| {
        group.len() == 4 && group.parse().is_ok_and(|year| YEARS.contains(&year))
    })?;
    let year: u32 = groups[year_index].parse().ok()?;
    match month_from_name(&key).or_else(|| month_from_digits(&groups, year_index)) {
        Some(month) => Some(format!("{year:04}-{month:02}")),
        None => Some(format!("{year:04}")),
    }
}

/// Suites de chiffres de la clé, dans l'ordre : « 15/03/2022 » donne `["15", "03", "2022"]`.
fn digit_groups(key: &str) -> Vec<&str> {
    key.split(|character: char| !character.is_ascii_digit())
        .filter(|group| !group.is_empty())
        .collect()
}

/// Mois nommé dans la clé, comparé mot à mot.
fn month_from_name(key: &str) -> Option<u32> {
    key.split(|character: char| !character.is_ascii_alphabetic())
        .filter(|token| !token.is_empty())
        .find_map(|token| {
            MONTHS
                .iter()
                .find(|(name, _)| *name == token)
                .map(|(_, month)| *month)
        })
}

/// Mois déduit de la position des nombres autour de l'année.
///
/// Le nombre qui suit l'année la précise (`2022-03`, `2022-03-15`). Sinon c'est celui qui
/// la précède (`03/2022`, `15/03/2022`), sauf s'il dépasse douze : la date est alors écrite
/// à l'américaine (`03/15/2022`) et le mois est le nombre d'encore avant.
fn month_from_digits(groups: &[&str], year_index: usize) -> Option<u32> {
    if let Some(month) = groups
        .get(year_index + 1)
        .and_then(|group| month_value(group))
    {
        return Some(month);
    }
    let before = year_index.checked_sub(1)?;
    match month_value(groups[before]) {
        Some(month) => Some(month),
        None => month_value(groups[before.checked_sub(1)?]),
    }
}

/// Nombre à un ou deux chiffres désignant un mois valide.
fn month_value(group: &str) -> Option<u32> {
    (group.len() <= 2)
        .then(|| group.parse().ok())
        .flatten()
        .filter(|month| (1..=12).contains(month))
}

/// La valeur désigne-t-elle une période encore en cours ?
fn is_present(value: &str) -> bool {
    let key = search_key(value);
    PRESENT_MARKERS.iter().any(|marker| key.contains(marker))
        || PRESENT_EXACT.iter().any(|marker| key == *marker)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profile::domain::{Certification, Education, Experience};

    #[test]
    fn un_format_deja_conforme_est_conserve() {
        assert_eq!(normalize_date("2022-03").as_deref(), Some("2022-03"));
        assert_eq!(normalize_date("2022").as_deref(), Some("2022"));
    }

    #[test]
    fn les_ecritures_courantes_d_un_cv_sont_ramenees_au_format() {
        for (brut, attendu) in [
            ("2022-3", "2022-03"),
            ("2022-03-15", "2022-03"),
            ("2022/03/15", "2022-03"),
            ("15/03/2022", "2022-03"),
            ("03/15/2022", "2022-03"),
            ("03/2022", "2022-03"),
            ("12/2023", "2023-12"),
            ("Juil. 2019", "2019-07"),
            ("Depuis sept. 2022", "2022-09"),
            ("Oct. 2025", "2025-10"),
            ("août 2016", "2016-08"),
            ("March 2022", "2022-03"),
        ] {
            assert_eq!(
                normalize_date(brut).as_deref(),
                Some(attendu),
                "date « {brut} »"
            );
        }
    }

    /// Une plage collée dans un seul champ est découpée avant normalisation.
    #[test]
    fn une_plage_d_annees_remplit_debut_et_fin() {
        let mut profile = Profile {
            experiences: vec![Experience {
                title: "Dev".into(),
                company: "Studio".into(),
                start_date: "2016 - 2019".into(),
                end_date: None,
                ..Experience::default()
            }],
            ..Profile::default()
        };

        normalize_profile_dates(&mut profile);

        assert_eq!(profile.experiences[0].start_date, "2016");
        assert_eq!(profile.experiences[0].end_date.as_deref(), Some("2019"));
    }

    #[test]
    fn une_plage_mois_annee_remplit_la_date_de_fin() {
        let mut profile = Profile {
            experiences: vec![Experience {
                title: "Full-Stack Developer".into(),
                company: "Kivora Commerce".into(),
                start_date: "juin 2020 – décembre 2022".into(),
                end_date: None,
                ..Experience::default()
            }],
            ..Profile::default()
        };

        normalize_profile_dates(&mut profile);

        assert_eq!(profile.experiences[0].start_date, "2020-06");
        assert_eq!(profile.experiences[0].end_date.as_deref(), Some("2022-12"));
        assert!(!profile.experiences[0].current);
    }

    #[test]
    fn un_fragment_sans_annee_est_abandonne() {
        for brut in ["en", "sept", ".", "", "aujourd'hui", "n/a"] {
            assert_eq!(normalize_date(brut), None, "date « {brut} »");
        }
    }

    #[test]
    fn une_fin_en_cours_devient_un_poste_actuel() {
        let mut profile = Profile {
            experiences: vec![Experience {
                title: "Développeur".into(),
                company: "Linaïa".into(),
                start_date: "Juil. 2019".into(),
                end_date: Some("aujourd'hui".into()),
                current: false,
                ..Experience::default()
            }],
            ..Profile::default()
        };

        normalize_profile_dates(&mut profile);

        assert_eq!(profile.experiences[0].start_date, "2019-07");
        assert_eq!(profile.experiences[0].end_date, None);
        assert!(profile.experiences[0].current);
    }

    #[test]
    fn une_plage_jusqu_a_aujourd_hui_marque_le_poste_actuel() {
        let mut profile = Profile {
            experiences: vec![Experience {
                title: "Senior".into(),
                company: "AlthéaRH".into(),
                start_date: "janvier 2023 – aujourd'hui".into(),
                end_date: None,
                current: false,
                ..Experience::default()
            }],
            ..Profile::default()
        };

        normalize_profile_dates(&mut profile);

        assert_eq!(profile.experiences[0].start_date, "2023-01");
        assert_eq!(profile.experiences[0].end_date, None);
        assert!(profile.experiences[0].current);
    }

    #[test]
    fn les_formations_et_certifications_sont_normalisees() {
        let mut profile = Profile {
            education: vec![Education {
                degree: "Master".into(),
                school: "ENI".into(),
                start_date: Some("en".into()),
                end_date: Some("juin 2019".into()),
                ..Education::default()
            }],
            certifications: vec![Certification {
                name: "AWS".into(),
                date: Some("12/2023".into()),
                ..Certification::default()
            }],
            ..Profile::default()
        };

        normalize_profile_dates(&mut profile);

        assert_eq!(profile.education[0].start_date, None);
        assert_eq!(profile.education[0].end_date.as_deref(), Some("2019-06"));
        assert_eq!(profile.certifications[0].date.as_deref(), Some("2023-12"));
    }
}
