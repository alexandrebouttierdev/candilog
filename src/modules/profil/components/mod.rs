//! Rendu des collections du profil professionnel.

use crate::shared::profile::{Certification, Education, Experience, Language, Project};
use crate::ui::components::{list, typo};
use crate::ui::format;
use iced::Element;

/// Période lisible d'une expérience.
#[must_use]
pub fn experience_period(experience: &Experience) -> String {
    let end = if experience.current {
        "aujourd'hui".to_owned()
    } else {
        format::or_else(experience.end_date.as_deref(), "—")
    };
    format!("{} → {end}", experience.start_date)
}

/// Ligne d'une expérience professionnelle.
pub fn experience_row<'a, Message: 'a>(experience: &Experience) -> Element<'a, Message> {
    list::row_static(
        iced::widget::Space::with_width(0),
        iced::widget::column![
            typo::item(format!("{} — {}", experience.title, experience.company)),
            typo::caption(format::or_dash(experience.description.as_deref())),
        ]
        .spacing(0),
        typo::caption(experience_period(experience)),
    )
}

/// Ligne d'une formation.
pub fn education_row<'a, Message: 'a>(education: &Education) -> Element<'a, Message> {
    list::row_static(
        iced::widget::Space::with_width(0),
        iced::widget::column![
            typo::item(education.degree.clone()),
            typo::caption(education.school.clone()),
        ]
        .spacing(0),
        typo::caption(format::or_dash(education.end_date.as_deref())),
    )
}

/// Ligne d'une langue.
pub fn language_row<'a, Message: 'a>(language: &Language) -> Element<'a, Message> {
    list::row_static(
        iced::widget::Space::with_width(0),
        typo::body(language.name.clone()),
        typo::caption(language.level.clone()),
    )
}

/// Ligne d'un projet.
pub fn project_row<'a, Message: 'a>(project: &Project) -> Element<'a, Message> {
    list::row_static(
        iced::widget::Space::with_width(0),
        iced::widget::column![
            typo::item(project.name.clone()),
            typo::caption(format::or_dash(project.description.as_deref())),
        ]
        .spacing(0),
        typo::caption(format::or_dash(project.technologies.as_deref())),
    )
}

/// Ligne d'une certification.
pub fn certification_row<'a, Message: 'a>(certification: &Certification) -> Element<'a, Message> {
    list::row_static(
        iced::widget::Space::with_width(0),
        iced::widget::column![
            typo::item(certification.name.clone()),
            typo::caption(format::or_dash(certification.issuer.as_deref())),
        ]
        .spacing(0),
        typo::caption(format::or_dash(certification.date.as_deref())),
    )
}

#[cfg(test)]
mod tests {
    use super::experience_period;
    use crate::shared::profile::Experience;

    #[test]
    fn un_poste_en_cours_est_signale_explicitement() {
        let experience = Experience {
            start_date: "2023-06".into(),
            current: true,
            ..Experience::default()
        };
        assert_eq!(experience_period(&experience), "2023-06 → aujourd'hui");
    }

    #[test]
    fn une_periode_close_affiche_sa_date_de_fin() {
        let experience = Experience {
            start_date: "2020-01".into(),
            end_date: Some("2022-12".into()),
            current: false,
            ..Experience::default()
        };
        assert_eq!(experience_period(&experience), "2020-01 → 2022-12");
    }

    #[test]
    fn une_periode_incomplete_reste_lisible() {
        let experience = Experience {
            start_date: "2019".into(),
            end_date: None,
            current: false,
            ..Experience::default()
        };
        assert_eq!(experience_period(&experience), "2019 → —");
    }
}
