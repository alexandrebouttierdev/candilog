//! Formatage d'affichage partagé par les écrans.
//!
//! Fonctions pures, sans dépendance à l'état ni au thème : elles transforment
//! des valeurs déjà chargées en chaînes lisibles dans une interface dense.

/// Transforme une date ISO en valeur courte adaptée aux listes desktop.
#[must_use]
pub fn compact_date(value: &str) -> String {
    let date = value.get(..10).unwrap_or(value);
    let mut parts = date.split('-');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(year), Some(month), Some(day), None)
            if year.len() == 4 && month.len() == 2 && day.len() == 2 =>
        {
            format!("{day}/{month}/{year}")
        }
        _ => value.to_owned(),
    }
}

/// Extrait l'heure `HH:MM` d'un horodatage ISO.
#[must_use]
pub fn time_part(value: &str) -> String {
    value
        .split_once('T')
        .map_or_else(String::new, |(_, time)| time.chars().take(5).collect())
}

/// Transforme un horodatage ISO en date et heure compactes.
#[must_use]
pub fn compact_datetime(value: &str) -> String {
    let time = time_part(value);
    if time.is_empty() {
        compact_date(value)
    } else {
        format!("{} · {time}", compact_date(value))
    }
}

/// Nom français d'un mois, de 1 à 12.
#[must_use]
pub const fn month_name(month: u32) -> &'static str {
    match month {
        1 => "Janvier",
        2 => "Février",
        3 => "Mars",
        4 => "Avril",
        5 => "Mai",
        6 => "Juin",
        7 => "Juillet",
        8 => "Août",
        9 => "Septembre",
        10 => "Octobre",
        11 => "Novembre",
        12 => "Décembre",
        _ => "Mois",
    }
}

/// Jour de la semaine en toutes lettres, du lundi au dimanche.
#[must_use]
pub const fn weekday_name(index: u32) -> &'static str {
    match index {
        0 => "Lundi",
        1 => "Mardi",
        2 => "Mercredi",
        3 => "Jeudi",
        4 => "Vendredi",
        5 => "Samedi",
        _ => "Dimanche",
    }
}

/// Date longue en français : `chrono` ne formate qu'en locale C.
#[must_use]
pub fn long_date(date: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    format!(
        "{} {} {} {}",
        weekday_name(date.weekday().num_days_from_monday()),
        date.day(),
        month_name(date.month()).to_lowercase(),
        date.year()
    )
}

/// Abréviation française d'un jour de la semaine, du lundi au dimanche.
#[must_use]
pub const fn weekday_abbrev(index: u32) -> &'static str {
    match index {
        0 => "Lun",
        1 => "Mar",
        2 => "Mer",
        3 => "Jeu",
        4 => "Ven",
        5 => "Sam",
        _ => "Dim",
    }
}

/// Tronque une valeur trop longue pour une colonne dense.
#[must_use]
pub fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }
    let kept: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// Valeur affichable, ou un tiret cadratin quand la donnée est absente.
#[must_use]
pub fn or_dash(value: Option<&str>) -> String {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value.to_owned(),
        None => "—".to_owned(),
    }
}

/// Valeur affichable, ou un libellé explicite quand la donnée est absente.
#[must_use]
pub fn or_else(value: Option<&str>, fallback: &str) -> String {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value.to_owned(),
        None => fallback.to_owned(),
    }
}

/// Pourcentage entier, sûr même quand le dénominateur est nul.
#[must_use]
pub fn percent(part: usize, total: usize) -> usize {
    part.saturating_mul(100).checked_div(total).unwrap_or(0)
}

/// Accorde un libellé au pluriel sans dupliquer la logique dans les écrans.
#[must_use]
pub fn plural(count: usize, singular: &str, plural: &str) -> String {
    if count > 1 {
        format!("{count} {plural}")
    } else {
        format!("{count} {singular}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compact_date, compact_datetime, long_date, month_name, or_dash, or_else, percent, plural,
        time_part, truncate, weekday_abbrev, weekday_name,
    };

    #[test]
    fn date_iso_est_formatee_pour_une_liste_desktop() {
        assert_eq!(compact_date("2026-08-09T10:15:00+02:00"), "09/08/2026");
        assert_eq!(compact_date("2026-08-09"), "09/08/2026");
    }

    #[test]
    fn valeur_non_iso_est_conservee() {
        assert_eq!(compact_date("à définir"), "à définir");
    }

    #[test]
    fn horodatage_iso_est_formate_pour_un_inspecteur() {
        assert_eq!(
            compact_datetime("2026-08-09T10:15:00+02:00"),
            "09/08/2026 · 10:15"
        );
        assert_eq!(compact_datetime("2026-08-09"), "09/08/2026");
    }

    #[test]
    fn heure_est_extraite_sans_les_secondes() {
        assert_eq!(time_part("2026-08-09T10:15:00+02:00"), "10:15");
        assert_eq!(time_part("2026-08-09"), "");
    }

    #[test]
    fn mois_et_jours_couvrent_toute_l_annee_et_la_semaine() {
        for month in 1..=12 {
            assert_ne!(month_name(month), "Mois");
        }
        assert_eq!(month_name(0), "Mois");
        assert_eq!(month_name(13), "Mois");
        assert_eq!(weekday_abbrev(0), "Lun");
        assert_eq!(weekday_abbrev(6), "Dim");
        assert_eq!(weekday_name(0), "Lundi");
        assert_eq!(weekday_name(6), "Dimanche");
    }

    #[test]
    fn la_date_longue_est_ecrite_en_francais() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 8, 9).expect("date valide");
        assert_eq!(long_date(date), "Dimanche 9 août 2026");
    }

    #[test]
    fn troncature_preserve_la_lisibilite() {
        assert_eq!(truncate("Développeur", 20), "Développeur");
        assert_eq!(truncate("Développeur Rust senior", 12), "Développeur…");
        assert_eq!(truncate("abc", 3), "abc");
    }

    #[test]
    fn valeur_absente_devient_un_tiret() {
        assert_eq!(or_dash(None), "—");
        assert_eq!(or_dash(Some("   ")), "—");
        assert_eq!(or_dash(Some("Rennes")), "Rennes");
        assert_eq!(or_else(None, "Non renseignée"), "Non renseignée");
        assert_eq!(or_else(Some("Nantes"), "Non renseignée"), "Nantes");
    }

    #[test]
    fn pourcentage_ne_divise_jamais_par_zero() {
        assert_eq!(percent(3, 0), 0);
        assert_eq!(percent(1, 4), 25);
        assert_eq!(percent(140, 140), 100);
    }

    #[test]
    fn accord_au_pluriel_est_correct() {
        assert_eq!(plural(0, "candidature", "candidatures"), "0 candidature");
        assert_eq!(plural(1, "candidature", "candidatures"), "1 candidature");
        assert_eq!(plural(2, "candidature", "candidatures"), "2 candidatures");
    }
}
