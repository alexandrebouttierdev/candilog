//! Écran Profil : identité, complétion et sections en cartes sur une seule
//! page défilante, dans l'esprit candilog-desktop.

use crate::app::{App, Message};
use crate::modules::profil::service::completion_score;
use crate::shared::profile::PersonalInfo;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::{layout, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, progress_bar, row};

pub mod import;
pub mod sections;

use iced::{Alignment, Background, Border, Element, Length, Theme};
pub use import::import_review_drawer;
use import::import_section;
use sections::sections_grid;

/// Largeur de la colonne de complétion, à droite de la carte d'identité
/// (`w-36`).
const COMPLETION_WIDTH: f32 = 144.0;
/// Rend l'écran du profil.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::page_header(
            Icon::Profile,
            "Profil professionnel",
            "Votre identité et vos atouts",
            iced::widget::Space::with_width(0).into(),
        ),
        layout::workspace(surface::scroll(
            column![identity_card(app), import_section(app), sections_grid(app),]
                .spacing(space::LG)
                .width(Length::Fill),
        )),
    )
}

/// Carte d'identité : nom, accroche, contacts et complétion, sans emplacement photo.
fn identity_card(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile;
    let personal = &profile.personal;
    let name = format!("{} {}", personal.first_name, personal.last_name)
        .trim()
        .to_owned();
    let score = completion_score(profile);

    container(
        row![
            column![
                typo::title(if name.is_empty() {
                    "Profil à compléter".to_owned()
                } else {
                    name
                }),
                typo::toned(
                    format::or_else(
                        personal.headline.as_deref(),
                        "Titre professionnel non renseigné",
                    ),
                    Tone::Accent,
                ),
                contact_chips(personal),
            ]
            .spacing(space::SM)
            .align_x(Alignment::Start),
            layout::spacer(),
            completion_panel(score),
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center),
    )
    .padding(space::XXL)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Jetons de contact : e-mail, téléphone et ville.
fn contact_chips<'a, Message: 'a>(personal: &PersonalInfo) -> Element<'a, Message> {
    let values: Vec<String> = [
        Some(personal.email.as_str()).filter(|value| !value.is_empty()),
        personal.phone.as_deref(),
        personal.city.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(str::to_owned)
    .collect();

    let mut line = row![].spacing(space::SM);
    for value in values {
        line = line.push(chip(value));
    }
    line.into()
}

/// Jeton de contact : bordure fine sur fond ambiant translucide
/// (`rounded-full border bg-background/35`).
fn chip<'a, Message: 'a>(value: String) -> Element<'a, Message> {
    container(typo::caption(value))
        .padding([4.0, 10.0])
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(alpha(palette.canvas, 0.35))),
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: radius::PILL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Colonne de complétion : libellé, barre et pourcentage monospace.
fn completion_panel<'a, Message: 'a>(score: u8) -> Element<'a, Message> {
    column![
        typo::caption("Profil complété"),
        progress_bar(0.0..=1.0, f32::from(score) / 100.0)
            .height(6.0)
            .style(styles::progress(Tone::Accent)),
        typo::text_mono(format!("{score} %"), font::MICRO, font::MONO_SEMIBOLD),
    ]
    .spacing(space::XS)
    .width(Length::Fixed(COMPLETION_WIDTH))
    .into()
}
