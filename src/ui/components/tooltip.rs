//! Infobulle d'accompagnement.
//!
//! Elle ne porte jamais d'information neuve : elle restitue un intitulé que la
//! densité a masqué, comme le libellé d'une tuile de rail replié.

use crate::ui::components::typo;
use crate::ui::theme::metrics::{radius, space, stroke};
use crate::ui::theme::tokens::tokens;
use iced::widget::{container, tooltip};
use iced::{Border, Element, Theme};

/// Côté sur lequel l'infobulle se déploie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// À droite : le cas du rail de navigation.
    Right,
    /// En dessous : le cas d'un bouton de toolbar.
    Bottom,
}

impl Side {
    const fn position(self) -> tooltip::Position {
        match self {
            Self::Right => tooltip::Position::Right,
            Self::Bottom => tooltip::Position::Bottom,
        }
    }
}

/// Enveloppe un élément d'une infobulle portant son intitulé complet.
pub fn tip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    side: Side,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(typo::caption(label))
            .padding([space::XS, space::SM])
            .style(|theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(iced::Background::Color(palette.raised)),
                    text_color: Some(palette.text),
                    border: Border {
                        color: palette.border_strong,
                        width: stroke::HAIRLINE,
                        radius: radius::CONTROL.into(),
                    },
                    ..container::Style::default()
                }
            }),
        side.position(),
    )
    .gap(space::XS)
    .into()
}

#[cfg(test)]
mod tests {
    use super::Side;

    #[test]
    fn les_deux_cotes_sont_distincts() {
        assert_ne!(Side::Right, Side::Bottom);
    }
}
