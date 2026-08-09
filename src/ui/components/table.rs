//! Vue de données desktop : en-tête figé, lignes denses, tri, sélection.
//!
//! L'en-tête de colonnes vit **hors** de la zone défilante : il reste visible
//! quel que soit le nombre de lignes.

use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Alignement d'une colonne.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Texte : aligné à gauche.
    #[default]
    Start,
    /// Statut ou marqueur : centré.
    Center,
    /// Nombre, score ou date : aligné à droite.
    End,
}

impl Align {
    const fn alignment(self) -> Alignment {
        match self {
            Self::Start => Alignment::Start,
            Self::Center => Alignment::Center,
            Self::End => Alignment::End,
        }
    }
}

/// Sens de tri appliqué à une colonne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ordre croissant.
    Ascending,
    /// Ordre décroissant.
    Descending,
}

impl SortOrder {
    /// Bascule le sens de tri.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    const fn chevron(self) -> Icon {
        match self {
            Self::Ascending => Icon::ChevronUp,
            Self::Descending => Icon::ChevronDown,
        }
    }
}

/// Largeur d'une colonne, en portions ou en pixels.
#[derive(Debug, Clone, Copy)]
pub enum Width {
    /// Part proportionnelle de l'espace disponible.
    Portion(u16),
    /// Largeur fixe, pour une colonne d'actions ou de score.
    Fixed(f32),
}

impl Width {
    const fn length(self) -> Length {
        match self {
            Self::Portion(portion) => Length::FillPortion(portion),
            Self::Fixed(pixels) => Length::Fixed(pixels),
        }
    }
}

/// Déclaration d'une colonne de table.
#[derive(Debug, Clone, Copy)]
pub struct Column {
    /// Libellé affiché dans l'en-tête.
    pub label: &'static str,
    /// Largeur de la colonne.
    pub width: Width,
    /// Alignement du contenu.
    pub align: Align,
}

impl Column {
    /// Colonne textuelle occupant une part de la largeur.
    #[must_use]
    pub const fn text(label: &'static str, portion: u16) -> Self {
        Self {
            label,
            width: Width::Portion(portion),
            align: Align::Start,
        }
    }

    /// Colonne de largeur fixe alignée à droite.
    #[must_use]
    pub const fn trailing(label: &'static str, pixels: f32) -> Self {
        Self {
            label,
            width: Width::Fixed(pixels),
            align: Align::End,
        }
    }

    /// Colonne de largeur fixe centrée.
    #[must_use]
    pub const fn centered(label: &'static str, pixels: f32) -> Self {
        Self {
            label,
            width: Width::Fixed(pixels),
            align: Align::Center,
        }
    }
}

/// En-tête de colonnes, figé hors de la zone défilante.
pub fn header<'a, Message: 'a>(columns: &[Column]) -> Element<'a, Message> {
    let mut line = row![].spacing(space::LG).align_y(Alignment::Center);
    for column in columns {
        line = line.push(
            container(typo::label(column.label))
                .width(column.width.length())
                .align_x(column.align.alignment()),
        );
    }
    header_shell(line)
}

/// En-tête de colonnes dont certaines déclenchent un tri.
pub fn header_sortable<'a, Message: Clone + 'a>(
    columns: &[Column],
    active: usize,
    order: SortOrder,
    on_sort: impl Fn(usize) -> Message,
) -> Element<'a, Message> {
    let mut line = row![].spacing(space::LG).align_y(Alignment::Center);
    for (index, column) in columns.iter().enumerate() {
        let is_active = index == active;
        let mut label = row![typo::label(column.label)]
            .spacing(space::XS)
            .align_y(Alignment::Center);
        if is_active {
            label = label.push(icon::icon(order.chevron(), 11.0, Ink::Accent));
        }
        let control = button(
            container(label)
                .width(Length::Fill)
                .align_x(column.align.alignment()),
        )
        .padding(0)
        .height(size::TABLE_HEADER)
        .width(column.width.length())
        .style(styles::ghost)
        .on_press(on_sort(index));
        line = line.push(control);
    }
    header_shell(line)
}

fn header_shell<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content.into())
        .height(size::TABLE_HEADER)
        .padding([0.0, space::XL])
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.sunken)),
                text_color: Some(palette.text_secondary),
                border: Border {
                    color: palette.border_strong,
                    width: stroke::HAIRLINE,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Cellule alignée sur la déclaration de sa colonne.
pub fn cell<'a, Message: 'a>(
    column: Column,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content.into())
        .width(column.width.length())
        .align_x(column.align.alignment())
        .into()
}

/// Ligne de données cliquable, avec état sélectionné marqué à gauche.
pub fn row_button<'a, Message: Clone + 'a>(
    cells: impl IntoIterator<Item = Element<'a, Message>>,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let mut line = row![marker(selected)]
        .spacing(space::LG)
        .align_y(Alignment::Center);
    for cell in cells {
        line = line.push(cell);
    }
    column![
        button(line)
            .width(Length::Fill)
            .height(size::ROW)
            .padding([0.0, space::XL - 2.0])
            .style(styles::row_item(selected))
            .on_press(on_press),
        super::surface::divider(),
    ]
    .into()
}

/// Ligne de données non cliquable.
pub fn row_static<'a, Message: 'a>(
    cells: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![marker(false)]
        .spacing(space::LG)
        .align_y(Alignment::Center);
    for cell in cells {
        line = line.push(cell);
    }
    column![
        container(line)
            .width(Length::Fill)
            .height(size::ROW)
            .padding([0.0, space::XL - 2.0])
            .align_y(Alignment::Center),
        super::surface::divider(),
    ]
    .into()
}

fn marker<'a, Message: 'a>(selected: bool) -> Element<'a, Message> {
    container(Space::new(stroke::MARKER, size::ROW - 8.0))
        .style(move |theme: &Theme| container::Style {
            background: selected
                .then(|| Background::Color(tokens(theme).accent))
                .or(None),
            border: Border {
                radius: 1.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::{Align, Column, SortOrder, Width};

    #[test]
    fn le_sens_de_tri_bascule_et_revient() {
        assert_eq!(SortOrder::Ascending.toggled(), SortOrder::Descending);
        assert_eq!(
            SortOrder::Ascending.toggled().toggled(),
            SortOrder::Ascending
        );
    }

    #[test]
    fn les_colonnes_declarent_leur_alignement_metier() {
        assert_eq!(Column::text("POSTE", 3).align, Align::Start);
        assert_eq!(Column::trailing("DATE", 90.0).align, Align::End);
        assert_eq!(Column::centered("STATUT", 120.0).align, Align::Center);
    }

    #[test]
    fn une_colonne_proportionnelle_se_convertit_en_portion() {
        assert!(matches!(Column::text("POSTE", 4).width, Width::Portion(4)));
        assert!(matches!(
            Column::trailing("SCORE", 72.0).width,
            Width::Fixed(value) if (value - 72.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn l_alignement_par_defaut_est_textuel() {
        assert_eq!(Align::default(), Align::Start);
    }
}
