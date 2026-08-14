//! Écran Paramètres : fournisseurs, configuration, sauvegarde — page unique
//! dans l'esprit candilog-desktop.

use crate::app::{App, Message};
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{layout, surface, typo};
use crate::ui::theme::metrics::{radius, size, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length, Theme};

pub mod maintenance;
pub mod provider;

pub use maintenance::{about_view, backup_view, updates_view};
use provider::provider_card;

/// Largeur maximale du corps de la page (`max-w-[980px]`).
const BODY_MAX_WIDTH: f32 = 1120.0;

/// Rend l'écran des paramètres.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::route_header(
            Icon::Settings,
            "Intelligence artificielle",
            Route::Parametres,
            Message::Navigate,
            controls::primary("Enregistrer", Some(Icon::Save))
                .on_press(Message::SaveSettings)
                .into(),
        ),
        layout::workspace(
            column![surface::scroll(
                container(main_column(app))
                    .max_width(BODY_MAX_WIDTH)
                    .center_x(Length::Fill),
            )
            .height(Length::Fill),]
            .spacing(space::MD)
            .height(Length::Fill),
        ),
    )
}

/// Colonne principale dédiée à la configuration IA.
fn main_column(app: &App) -> Element<'_, Message> {
    column![provider_card(app)]
        .spacing(space::LG)
        .width(Length::Fill)
        .into()
}

/// Carte d'une section des paramètres : en-tête (icône, titre) et contenu.
fn section_card<'a>(
    glyph: Icon,
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = content.into();
    container(
        column![
            container(
                row![
                    header_tile(glyph),
                    text(title).size(font::LABEL).font(font::SEMIBOLD),
                    layout::spacer(),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .height(size::SECTION_HEADER)
            .align_y(Alignment::Center),
            surface::divider(),
            content,
        ]
        .width(Length::Fill),
    )
    .padding([space::MD, space::LG])
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Pastille d'icône d'un en-tête de section (`bg-secondary/70 rounded-xl`).
fn header_tile<'a, Message: 'a>(glyph: Icon) -> Element<'a, Message> {
    container(icon::icon(glyph, icon::SM, Ink::Muted))
        .width(28.0)
        .height(28.0)
        .center(Length::Fixed(28.0))
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

/// Bandeau éditorial commun aux écrans secondaires des réglages.
fn settings_hero<'a>(
    glyph: Icon,
    eyebrow: &'a str,
    title: &'a str,
    description: &'a str,
) -> Element<'a, Message> {
    container(
        row![
            container(icon::icon(glyph, 28.0, Ink::Accent))
                .width(56.0)
                .height(56.0)
                .center(Length::Fixed(56.0))
                .style(styles::toned(Tone::Accent)),
            column![
                typo::meta_toned(eyebrow, Tone::Accent),
                typo::title(title),
                typo::caption(description),
            ]
            .spacing(space::XS),
        ]
        .spacing(space::LG)
        .align_y(Alignment::Center),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Carte d'action autonome : une intention par surface, plus lisible qu'une pile de boutons.
fn action_card<'a>(
    glyph: Icon,
    title: &'a str,
    description: &'a str,
    action: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        column![
            header_tile(glyph),
            typo::label(title),
            typo::caption(description),
            container(action).padding([space::MD, 0.0]),
        ]
        .spacing(space::SM)
        .width(Length::Fill),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .height(190.0)
    .style(styles::glass_card)
    .into()
}
