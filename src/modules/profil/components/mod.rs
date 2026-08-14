//! Rendu des collections du profil professionnel.

use crate::shared::profile::{Certification, Education, Experience, Language, Project};
use crate::ui::components::{list, typo};
use crate::ui::format;
use crate::ui::theme::metrics::radius;
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use iced::widget::container;
use iced::{Background, Border, Element, Theme};

pub mod form;

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

/// Période en jeton monospace (`bg-secondary/70 rounded-md`), sur la
/// timeline des expériences.
pub fn period_badge<'a, Message: 'a>(
    period: impl iced::widget::text::IntoFragment<'a>,
) -> Element<'a, Message> {
    container(typo::text_mono(period, font::MICRO, font::MONO_MEDIUM).style(styles::secondary_text))
        .padding([3.0, 8.0])
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(alpha(palette.sunken, 0.70))),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
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
        period_badge(experience_period(experience)),
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
#[path = "tests/mod/mod.rs"]
mod tests;
