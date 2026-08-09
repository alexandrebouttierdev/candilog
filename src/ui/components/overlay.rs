//! Modales, drawers et feuilles.
//!
//! La taille suit l'usage : une confirmation ne s'affiche pas dans le même
//! cadre qu'un formulaire dense. Le drawer d'inspecteur laisse le contexte
//! lisible plutôt que de le masquer sous un voile opaque.

use super::button as controls;
use super::icon::Icon;
use super::surface;
use super::typo;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use iced::widget::{column, container, mouse_area, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme, Vector};

/// Gabarit d'une modale, choisi selon le contenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    /// Confirmation courte.
    Confirm,
    /// Formulaire courant.
    Form,
    /// Formulaire dense ou contenu tabulaire.
    Wide,
}

impl Size {
    const fn width(self) -> f32 {
        match self {
            Self::Confirm => size::DIALOG_CONFIRM,
            Self::Form => size::DIALOG_FORM,
            Self::Wide => size::DIALOG_WIDE,
        }
    }
}

/// Modale centrée, contenue dans la fenêtre, fermable par le voile.
pub fn modal<'a, Message: Clone + 'a>(
    title: &'a str,
    body: impl Into<Element<'a, Message>>,
    footer: impl Into<Element<'a, Message>>,
    kind: Size,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let panel = container(
        column![
            row![
                typo::title(title),
                Space::with_width(Length::Fill),
                controls::icon_action(Icon::Close, "Fermer", on_dismiss.clone()),
            ]
            .align_y(Alignment::Center),
            surface::divider(),
            surface::scroll(container(body.into()).padding([space::XL, 0.0]))
                .height(Length::Shrink),
            footer.into(),
        ]
        .spacing(space::LG),
    )
    .width(kind.width())
    .max_height(660)
    .padding(space::XXL)
    .style(dialog_surface);

    mouse_area(
        container(mouse_area(panel))
            .center(Length::Fill)
            .padding(space::MAX)
            .style(styles::scrim),
    )
    .on_press(on_dismiss)
    .into()
}

/// Drawer d'inspecteur ancré à droite.
pub fn drawer<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let panel = container(content.into())
        .width(size::DRAWER)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.panel)),
                text_color: Some(palette.text),
                border: Border {
                    color: palette.border_strong,
                    width: stroke::HAIRLINE,
                    radius: 0.0.into(),
                },
                shadow: Shadow {
                    color: palette.shadow,
                    offset: Vector::new(-14.0, 0.0),
                    blur_radius: 40.0,
                },
            }
        });

    row![
        mouse_area(
            container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(alpha(
                        tokens(theme).scrim,
                        tokens(theme).scrim.a * 0.55,
                    ))),
                    ..container::Style::default()
                }),
        )
        .on_press(on_dismiss),
        mouse_area(panel),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Feuille flottante ancrée sous une toolbar (filtres, menu secondaire).
pub fn sheet<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    offset_x: f32,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let panel = container(content.into())
        .padding(space::XL)
        .style(dialog_surface);

    mouse_area(
        container(
            column![
                Space::with_height(size::TOOLBAR + 4.0),
                row![
                    Space::with_width(Length::Fixed(offset_x)),
                    mouse_area(panel),
                    Space::with_width(Length::Fill),
                ],
                Space::with_height(Length::Fill),
            ]
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .on_press(on_dismiss)
    .into()
}

/// Pied de dialogue : action secondaire puis action principale, à droite.
pub fn footer<'a, Message: 'a>(
    actions: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![Space::with_width(Length::Fill)]
        .spacing(space::MD)
        .align_y(Alignment::Center);
    for action in actions {
        line = line.push(action);
    }
    line.into()
}

fn dialog_surface(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.raised)),
        text_color: Some(palette.text),
        border: Border {
            color: palette.border_strong,
            width: stroke::HAIRLINE,
            radius: radius::DIALOG.into(),
        },
        shadow: Shadow {
            color: palette.shadow,
            offset: Vector::new(0.0, 18.0),
            blur_radius: 44.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::Size;

    #[test]
    fn les_gabarits_suivent_l_usage() {
        assert!(Size::Confirm.width() < Size::Form.width());
        assert!(Size::Form.width() < Size::Wide.width());
    }

    #[test]
    fn une_confirmation_reste_etroite() {
        // Densité Confort : le plafond suit la nouvelle largeur de DIALOG_CONFIRM.
        assert!(Size::Confirm.width() <= 420.0);
    }
}
