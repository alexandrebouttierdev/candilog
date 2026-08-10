//! Modales, drawers et feuilles.
//!
//! La taille suit l'usage : une confirmation ne s'affiche pas dans le même
//! cadre qu'un formulaire dense. Le drawer d'inspecteur laisse le contexte
//! lisible plutôt que de le masquer sous un voile opaque. Sur une fenêtre
//! assez large, ce drawer cède la place à [`side_panel`] : le même contenu,
//! mais posé à côté du contenu plutôt que par-dessus.

use super::button as controls;
use super::icon::Icon;
use super::surface;
use super::typo;
use crate::ui::theme::layout::MIN_HEIGHT;
use crate::ui::theme::metrics::{elevation, radius, size, space, stroke};
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
    .max_height(MIN_HEIGHT)
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
                    radius: radius::NONE.into(),
                },
                shadow: Shadow {
                    color: palette.shadow,
                    offset: Vector::new(-elevation::DRAWER_OFFSET, 0.0),
                    blur_radius: elevation::DRAWER_BLUR,
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

/// Inspecteur posé en colonne, à côté du contenu plutôt que par-dessus.
///
/// Même contenu qu'un [`drawer`], mais sans voile et sans ombre : appelé
/// quand la fenêtre est assez large pour que l'inspecteur prenne sa place
/// dans la rangée principale au lieu de flotter dans une couche superposée.
/// Un filet vertical à gauche le détache du volet de travail qu'il jouxte.
/// La signature reste alignée sur [`drawer`] ; `on_dismiss` n'est pas
/// consommé ici, faute de voile à fermer au clic — la fermeture reste
/// portée par le bouton de fermeture du contenu lui-même.
pub fn side_panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    _on_dismiss: Message,
) -> Element<'a, Message> {
    let panel = container(content.into())
        .width(size::DRAWER)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.panel)),
                text_color: Some(palette.text),
                border: Border::default(),
                shadow: Shadow::default(),
            }
        });

    row![surface::split_rule(), panel]
        .width(Length::Shrink)
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
                Space::with_height(size::TOOLBAR + space::XS),
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
            offset: Vector::new(0.0, elevation::OVERLAY_OFFSET),
            blur_radius: elevation::OVERLAY_BLUR,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::Size;
    use crate::ui::theme::metrics::size;

    #[test]
    fn les_gabarits_suivent_l_usage() {
        assert!(Size::Confirm.width() < Size::Form.width());
        assert!(Size::Form.width() < Size::Wide.width());
    }

    #[test]
    fn une_confirmation_reste_etroite() {
        assert!(Size::Confirm.width() <= size::DIALOG_CONFIRM);
    }
}
