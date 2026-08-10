//! Vue de données desktop : en-tête figé, lignes denses, tri, sélection.
//!
//! L'en-tête de colonnes vit **hors** de la zone défilante : il reste visible
//! quel que soit le nombre de lignes.
//!
//! Sous [`Layout::table_secondary_columns`], les colonnes marquées
//! [`Column::secondary`] disparaissent de l'en-tête et des lignes plutôt que
//! de déborder : leur contenu revient alors en métadonnée sous le titre de
//! la ligne, il ne disparaît jamais.

use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Layout;
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
    /// Vraie si la colonne peut être repliée sous
    /// [`Layout::table_secondary_columns`]. Son contenu revient alors en
    /// métadonnée sous le titre de la ligne plutôt que de disparaître.
    pub secondary: bool,
}

impl Column {
    /// Colonne textuelle occupant une part de la largeur.
    #[must_use]
    pub const fn text(label: &'static str, portion: u16) -> Self {
        Self {
            label,
            width: Width::Portion(portion),
            align: Align::Start,
            secondary: false,
        }
    }

    /// Colonne de largeur fixe alignée à droite.
    #[must_use]
    pub const fn trailing(label: &'static str, pixels: f32) -> Self {
        Self {
            label,
            width: Width::Fixed(pixels),
            align: Align::End,
            secondary: false,
        }
    }

    /// Colonne de largeur fixe centrée.
    #[must_use]
    pub const fn centered(label: &'static str, pixels: f32) -> Self {
        Self {
            label,
            width: Width::Fixed(pixels),
            align: Align::Center,
            secondary: false,
        }
    }

    /// Marque la colonne secondaire : sous 1280 px, elle disparaît de
    /// l'en-tête et son contenu revient en métadonnée sous le titre de la
    /// ligne plutôt que de disparaître.
    #[must_use]
    pub const fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    /// Vraie si la colonne doit apparaître dans son propre emplacement,
    /// compte tenu de la mise en page courante.
    fn shown(self, layout: Layout) -> bool {
        !self.secondary || layout.table_secondary_columns()
    }
}

/// En-tête de colonnes, figé hors de la zone défilante. Les colonnes
/// secondaires disparaissent sous [`Layout::table_secondary_columns`].
pub fn header<'a, Message: 'a>(layout: Layout, columns: &[Column]) -> Element<'a, Message> {
    let mut line = row![].spacing(space::LG).align_y(Alignment::Center);
    for column in columns {
        if !column.shown(layout) {
            continue;
        }
        line = line.push(
            container(typo::label(column.label))
                .width(column.width.length())
                .align_x(column.align.alignment()),
        );
    }
    header_shell(line)
}

/// En-tête de colonnes dont certaines déclenchent un tri. Les colonnes
/// secondaires disparaissent sous [`Layout::table_secondary_columns`].
pub fn header_sortable<'a, Message: Clone + 'a>(
    layout: Layout,
    columns: &[Column],
    active: usize,
    order: SortOrder,
    on_sort: impl Fn(usize) -> Message,
) -> Element<'a, Message> {
    let mut line = row![].spacing(space::LG).align_y(Alignment::Center);
    for (index, column) in columns.iter().enumerate() {
        if !column.shown(layout) {
            continue;
        }
        let is_active = index == active;
        let mut label = row![typo::label(column.label)]
            .spacing(space::XS)
            .align_y(Alignment::Center);
        if is_active {
            label = label.push(icon::icon(
                order.chevron(),
                size::TABLE_SORT_ICON,
                Ink::Accent,
            ));
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
                    radius: radius::NONE.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Cellule alignée sur la déclaration de sa colonne, en attente d'assemblage
/// dans une ligne. [`row_button`] et [`row_static`] décident, selon la mise
/// en page, si son contenu reste dans la ligne principale ou revient en
/// métadonnée sous le titre lorsque sa colonne est secondaire et repliée.
pub struct Cell<'a, Message> {
    column: Column,
    element: Element<'a, Message>,
}

/// Cellule alignée sur la déclaration de sa colonne.
pub fn cell<'a, Message: 'a>(
    column: Column,
    content: impl Into<Element<'a, Message>>,
) -> Cell<'a, Message> {
    Cell {
        column,
        element: container(content.into())
            .width(column.width.length())
            .align_x(column.align.alignment())
            .into(),
    }
}

/// Sépare les cellules d'une ligne entre celles qui restent dans la ligne
/// principale et celles dont la colonne secondaire est repliée : ces
/// dernières sont mises de côté pour rejoindre la ligne de métadonnées
/// plutôt que de disparaître. C'est cette répartition, isolée de la
/// construction des widgets, qui porte la garantie testable de
/// [`assemble`] : une colonne repliée ne reste jamais dans `shown`.
fn partition<'a, Message: 'a>(
    layout: Layout,
    cells: impl IntoIterator<Item = Cell<'a, Message>>,
) -> (Vec<Cell<'a, Message>>, Vec<Cell<'a, Message>>) {
    let mut shown = Vec::new();
    let mut folded = Vec::new();
    for cell in cells {
        if cell.column.shown(layout) {
            shown.push(cell);
        } else {
            folded.push(cell);
        }
    }
    (shown, folded)
}

/// Assemble les cellules d'une ligne entre la ligne principale et, si des
/// colonnes secondaires sont repliées, la ligne de métadonnées qui les
/// remplace sous le titre plutôt que de les faire disparaître. Le second
/// membre du tuple indique si une ligne de métadonnées a été produite.
fn assemble<'a, Message: 'a>(
    layout: Layout,
    marker: Element<'a, Message>,
    cells: impl IntoIterator<Item = Cell<'a, Message>>,
) -> (Element<'a, Message>, bool) {
    let (shown, folded) = partition(layout, cells);
    let has_meta = !folded.is_empty();

    let mut line = row![marker].spacing(space::LG).align_y(Alignment::Center);
    for cell in shown {
        line = line.push(cell.element);
    }

    if has_meta {
        let mut meta = row![].spacing(space::LG).align_y(Alignment::Center);
        for cell in folded {
            meta = meta.push(cell.element);
        }
        (column![line, meta].spacing(space::XXS).into(), true)
    } else {
        (line.into(), false)
    }
}

/// Ligne de données cliquable, avec état sélectionné marqué à gauche. Le
/// contenu des colonnes secondaires repliées revient en métadonnée sous le
/// titre de la ligne, la hauteur de ligne s'ajuste en conséquence.
pub fn row_button<'a, Message: Clone + 'a>(
    layout: Layout,
    cells: impl IntoIterator<Item = Cell<'a, Message>>,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let (content, comfortable) = assemble(layout, marker(selected), cells);
    let height = if comfortable {
        size::ROW_COMFORTABLE
    } else {
        size::ROW
    };
    column![
        button(content)
            .width(Length::Fill)
            .height(height)
            .padding([0.0, space::XL - stroke::MARKER])
            .style(styles::row_item(selected))
            .on_press(on_press),
        super::surface::divider(),
    ]
    .into()
}

/// Ligne de données non cliquable. Le contenu des colonnes secondaires
/// repliées revient en métadonnée sous le titre de la ligne, la hauteur de
/// ligne s'ajuste en conséquence.
pub fn row_static<'a, Message: 'a>(
    layout: Layout,
    cells: impl IntoIterator<Item = Cell<'a, Message>>,
) -> Element<'a, Message> {
    let (content, comfortable) = assemble(layout, marker(false), cells);
    let height = if comfortable {
        size::ROW_COMFORTABLE
    } else {
        size::ROW
    };
    column![
        container(content)
            .width(Length::Fill)
            .height(height)
            .padding([0.0, space::XL - stroke::MARKER])
            .align_y(Alignment::Center),
        super::surface::divider(),
    ]
    .into()
}

fn marker<'a, Message: 'a>(selected: bool) -> Element<'a, Message> {
    container(Space::new(stroke::MARKER, size::ROW_MARKER))
        .style(move |theme: &Theme| container::Style {
            background: selected
                .then(|| Background::Color(tokens(theme).accent))
                .or(None),
            border: Border {
                radius: radius::MARKER.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::{Align, Column, Element, SortOrder, Width};
    use crate::ui::theme::layout::{MIN_HEIGHT, MIN_WIDTH};
    use crate::ui::theme::Layout;
    use iced::Size;

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

    #[test]
    fn une_colonne_est_principale_par_defaut_et_se_marque_secondaire() {
        assert!(!Column::text("ENTREPRISE", 3).secondary);
        assert!(Column::text("ENTREPRISE", 3).secondary().secondary);
    }

    #[test]
    fn une_colonne_secondaire_se_replie_sous_le_seuil_des_tables() {
        let etroit = Layout::from_size(Size::new(MIN_WIDTH, MIN_HEIGHT));
        let large = Layout::from_size(Size::new(1600.0, 900.0));
        let colonne = Column::text("ENTREPRISE", 3).secondary();
        assert!(!colonne.shown(etroit));
        assert!(colonne.shown(large));
    }

    #[test]
    fn une_colonne_principale_reste_visible_a_toute_largeur() {
        let etroit = Layout::from_size(Size::new(MIN_WIDTH, MIN_HEIGHT));
        assert!(Column::text("POSTE", 4).shown(etroit));
    }

    /// Deux cellules types, l'une principale (POSTE) et l'autre secondaire
    /// (ENTREPRISE), pour exercer directement la répartition faite par
    /// `partition`/`assemble`.
    fn cellules_de_test() -> Vec<super::Cell<'static, ()>> {
        vec![
            super::cell(Column::text("POSTE", 4), iced::widget::text("Poste")),
            super::cell(
                Column::text("ENTREPRISE", 3).secondary(),
                iced::widget::text("Entreprise"),
            ),
        ]
    }

    #[test]
    fn sous_le_seuil_la_colonne_secondaire_ne_reste_pas_dans_la_ligne_principale() {
        let etroit = Layout::from_size(Size::new(MIN_WIDTH, MIN_HEIGHT));
        let (shown, folded) = super::partition(etroit, cellules_de_test());

        // La colonne secondaire (ENTREPRISE) est repliée : elle ne doit
        // apparaître nulle part dans `shown`, la ligne principale. C'est
        // l'assertion qui échouerait si la branche de repli disparaissait
        // et que tout finissait, à tort, dans la ligne principale.
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].column.label, "POSTE");
        assert_eq!(folded.len(), 1);
        assert_eq!(folded[0].column.label, "ENTREPRISE");
    }

    #[test]
    fn au_dessus_du_seuil_aucune_colonne_n_est_repliee() {
        let large = Layout::from_size(Size::new(1600.0, 900.0));
        let (shown, folded) = super::partition(large, cellules_de_test());

        assert_eq!(shown.len(), 2);
        assert!(folded.is_empty());
    }

    #[test]
    fn assemble_signale_un_repli_uniquement_sous_le_seuil() {
        let etroit = Layout::from_size(Size::new(MIN_WIDTH, MIN_HEIGHT));
        let large = Layout::from_size(Size::new(1600.0, 900.0));
        let marqueur = || Element::from(iced::widget::text(""));

        let (_, replie_etroit) = super::assemble(etroit, marqueur(), cellules_de_test());
        let (_, replie_large) = super::assemble(large, marqueur(), cellules_de_test());

        assert!(
            replie_etroit,
            "une colonne secondaire masquée doit signaler un repli"
        );
        assert!(
            !replie_large,
            "aucune colonne masquée : pas de repli à signaler"
        );
    }
}
