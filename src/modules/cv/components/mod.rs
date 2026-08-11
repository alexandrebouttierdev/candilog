//! Rendu des objets de la bibliothèque de CV.

use crate::modules::cv::model::CvVersionSummary;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::typo;
use crate::ui::format;
use crate::ui::theme::color::Tone;
use crate::ui::theme::metrics::{radius, size, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use iced::border::Radius;
use iced::widget::button::{self, Status};
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

/// Carte d'une version de CV : vignette blanche, nom, date, actions.
///
/// Le bouton englobe la vignette et le pied de carte ; les actions du pied
/// (Éditer, Supprimer) captent leur propre clic avant lui (Iced traite
/// l'enfant avant le parent).
pub fn version_card<'a, Message: Clone + 'a>(
    version: &CvVersionSummary,
    on_open: Message,
    on_edit: Message,
    on_delete: Message,
) -> Element<'a, Message> {
    iced::widget::button(
        container(column![
            vignette(),
            card_footer(version, on_edit, on_delete),
        ])
        .width(Length::Fill)
        .style(styles::glass_card),
    )
    .width(Length::Fill)
    .style(card_lift)
    .on_press(on_open)
    .into()
}

/// Miniature de page blanche : titre d'accent puis lignes de texte simulées.
fn vignette<'a, Message: 'a>() -> Element<'a, Message> {
    container(
        column![
            container(iced::widget::Space::with_width(0))
                .width(32.0)
                .height(16.0)
                .style(|theme: &Theme| {
                    let palette = tokens(theme);
                    container::Style {
                        background: Some(Background::Color(palette.accent_fill)),
                        border: Border {
                            radius: 4.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                }),
            iced::widget::Space::with_height(Length::Fill),
            simulated_line(92, false),
            simulated_line(76, true),
            simulated_line(84, false),
            simulated_line(58, true),
        ]
        .width(Length::Fill)
        .padding(space::LG)
        .spacing(space::SM),
    )
    .width(Length::Fill)
    .height(170.0)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.paper)),
            border: Border {
                radius: Radius {
                    top_left: radius::CARD,
                    top_right: radius::CARD,
                    ..Radius::default()
                },
                ..Border::default()
            },
            ..container::Style::default()
        }
    })
    .into()
}

/// Ligne de texte simulée : barre grise dont la largeur varie.
fn simulated_line<'a, Message: 'a>(fraction: u16, faint: bool) -> Element<'a, Message> {
    row![
        container(iced::widget::Space::with_width(0))
            .width(Length::FillPortion(fraction))
            .height(4.0)
            .style(move |theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(if faint {
                        Color {
                            a: 0.35,
                            ..palette.paper_ink_muted
                        }
                    } else {
                        palette.paper_rule
                    })),
                    ..container::Style::default()
                }
            }),
        iced::widget::Space::with_width(Length::FillPortion(100 - fraction)),
    ]
    .width(Length::Fill)
    .into()
}

/// Pied de carte : nom et date, puis actions Éditer et Supprimer.
fn card_footer<'a, Message: Clone + 'a>(
    version: &CvVersionSummary,
    on_edit: Message,
    on_delete: Message,
) -> Element<'a, Message> {
    container(
        column![
            row![
                text(format::truncate(&version.name, 26))
                    .size(font::BODY)
                    .font(font::SEMIBOLD),
                iced::widget::Space::with_width(Length::Fill),
                typo::caption(format::compact_datetime(&version.created_at)),
            ]
            .align_y(Alignment::Center),
            row![
                iced::widget::button(
                    row![
                        icon::icon(Icon::Edit, icon::SM, Ink::Toned(Tone::Accent)),
                        text("Éditer")
                            .size(font::BODY)
                            .style(styles::toned_text(Tone::Accent)),
                    ]
                    .spacing(space::SM)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .height(size::ACTION)
                .padding([0.0, space::MD])
                .style(accent_ghost)
                .on_press(on_edit),
                controls::icon_danger(Icon::Trash, "Supprimer", on_delete),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(space::MD),
    )
    .padding(space::LG)
    .into()
}

/// Action discrète teintée d'accent : fond accent à 10 %, texte accent.
fn accent_ghost(theme: &Theme, status: Status) -> button::Style {
    let palette = tokens(theme);
    let engaged = matches!(status, Status::Hovered | Status::Pressed);
    button::Style {
        background: Some(Background::Color(alpha(
            palette.accent,
            if engaged { 0.18 } else { 0.10 },
        ))),
        text_color: alpha(
            palette.accent,
            if matches!(status, Status::Disabled) {
                0.45
            } else {
                1.0
            },
        ),
        border: Border {
            radius: radius::CONTROL.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
    }
}

/// Carte interactive plate : la vignette interne porte déjà son filet et sa surface.
fn card_lift(_theme: &Theme, _status: Status) -> button::Style {
    button::Style {
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

/// Date de la version la plus récente, ou « — » quand la bibliothèque est vide.
#[must_use]
pub fn latest_version_date(versions: &[CvVersionSummary]) -> String {
    versions
        .iter()
        .map(|version| version.created_at.as_str())
        .max()
        .map_or_else(|| "—".to_owned(), format::compact_datetime)
}

/// Détermine si une version correspond à une recherche libre (sur le nom).
#[must_use]
pub fn matches(version: &CvVersionSummary, needle: &str) -> bool {
    needle.is_empty() || version.name.to_lowercase().contains(needle)
}

#[cfg(test)]
mod tests {
    use super::{latest_version_date, matches, version_card};
    use crate::modules::cv::model::CvVersionSummary;
    use iced::Element;
    use uuid::Uuid;

    fn version(name: &str, created_at: &str) -> CvVersionSummary {
        CvVersionSummary {
            id: Uuid::new_v4(),
            name: name.into(),
            created_at: created_at.into(),
        }
    }

    #[test]
    fn la_bibliotheque_vide_n_a_pas_de_date() {
        assert_eq!(latest_version_date(&[]), "—");
    }

    #[test]
    fn la_date_la_plus_recente_est_retournee() {
        let versions = vec![
            version("V1", "2026-08-01T09:00:00+02:00"),
            version("V2", "2026-08-09T10:15:00+02:00"),
            version("V3", "2026-08-05T18:30:00+02:00"),
        ];
        assert_eq!(latest_version_date(&versions), "09-08-2026 · 10:15");
    }

    #[test]
    fn la_recherche_est_insensible_a_la_casse() {
        assert!(matches(
            &version("CV Consultant", "2026-08-01"),
            "consultant"
        ));
        assert!(matches(&version("CV Consultant", "2026-08-01"), "cv"));
        assert!(!matches(&version("CV Consultant", "2026-08-01"), "dev"));
        assert!(matches(&version("CV Consultant", "2026-08-01"), ""));
    }

    #[test]
    fn la_carte_de_version_s_instancie_avec_ses_trois_actions() {
        let version = version("CV Consultant", "2026-08-01");
        let _: Element<'_, ()> = version_card(&version, (), (), ());
    }
}
