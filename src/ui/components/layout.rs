//! Compositions d'écran.
//!
//! Un écran = une toolbar + un corps `Length::Fill`. Il n'existe aucun
//! `scrollable` global : le corps répartit lui-même ses zones défilantes.

use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use iced::widget::{column, container, row, Space};
use iced::{Element, Length};

/// Assemble un écran : toolbar figée, corps occupant toute la hauteur.
pub fn screen<'a, Message: 'a>(
    toolbar: impl Into<Element<'a, Message>>,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        toolbar.into(),
        container(body.into())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::canvas),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Écran doté d'une bande de contexte entre la toolbar et le corps.
pub fn screen_with_strip<'a, Message: 'a>(
    toolbar: impl Into<Element<'a, Message>>,
    strip: impl Into<Element<'a, Message>>,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        toolbar.into(),
        strip.into(),
        container(body.into())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::canvas),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Corps composé d'un seul contenu, avec la marge de plan de travail.
pub fn workspace<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content.into())
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([space::XL, space::WORKSPACE_X])
        .into()
}

/// Deux volets côte à côte : liste à gauche, détail à droite.
pub fn split<'a, Message: 'a>(
    master: impl Into<Element<'a, Message>>,
    detail: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![
        container(master.into())
            .width(Length::Fixed(size::MASTER))
            .height(Length::Fill),
        super::surface::split_rule(),
        container(detail.into())
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Deux volets proportionnels, pour un atelier (édition / aperçu).
pub fn split_portions<'a, Message: 'a>(
    left_portion: u16,
    left: impl Into<Element<'a, Message>>,
    right_portion: u16,
    right: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![
        container(left.into())
            .width(Length::FillPortion(left_portion))
            .height(Length::Fill),
        super::surface::split_rule(),
        container(right.into())
            .width(Length::FillPortion(right_portion))
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Colonne verticale à l'espacement canonique entre blocs d'un écran.
pub fn stack<'a, Message: 'a>(
    blocks: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut body = column![].spacing(space::LG).width(Length::Fill);
    for block in blocks {
        body = body.push(block);
    }
    body.into()
}

/// Ligne horizontale à l'espacement canonique entre blocs d'un écran.
pub fn columns<'a, Message: 'a>(
    blocks: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut body = row![].spacing(space::LG).height(Length::Fill);
    for block in blocks {
        body = body.push(block);
    }
    body.into()
}

/// Rangée de panneaux à **hauteur explicite**, pour un contenu défilable.
///
/// [`columns`] réclame `Length::Fill`, ce qui n'a pas de sens dans un conteneur défilable :
/// la hauteur y est illimitée, et un enfant `Fill` s'y effondre. Une hauteur explicite garantit
/// au contraire que les panneaux gardent la place nécessaire à leur contenu, le débordement
/// étant absorbé par le défilement de la page.
pub fn columns_of_height<'a, Message: 'a>(
    height: f32,
    blocks: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut body = row![].spacing(space::LG).height(Length::Fixed(height));
    for block in blocks {
        body = body.push(block);
    }
    body.into()
}

/// Espace élastique, pour repousser un élément vers la droite ou le bas.
pub fn spacer<'a, Message: 'a>() -> Element<'a, Message> {
    Space::with_width(Length::Fill).into()
}
