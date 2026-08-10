//! Onglets et contrôle segmenté : cadre arrondi, actif en inversion totale.

use crate::ui::theme::metrics::{radius, size, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{button, container, row};
use iced::{Background, Border, Element, Length, Theme};

/// Entrée d'onglet : libellé, état actif.
#[derive(Debug, Clone)]
pub struct Tab {
    /// Libellé affiché.
    pub label: String,
    /// Vrai si l'onglet est sélectionné.
    pub active: bool,
}

impl Tab {
    /// Construit un onglet inactif.
    #[must_use]
    pub fn new(label: impl Into<String>, active: bool) -> Self {
        Self {
            label: label.into(),
            active,
        }
    }
}

/// Cadre commun des onglets et segments.
fn shell<'a, Message: Clone + 'a>(
    entries: impl IntoIterator<Item = Tab>,
    on_select: impl Fn(usize) -> Message + 'a,
    panel_style: bool,
) -> Element<'a, Message> {
    let mut group = row![].spacing(space::XS);
    for (index, tab) in entries.into_iter().enumerate() {
        let control = button(
            iced::widget::text(tab.label)
                .size(if panel_style { 12.5 } else { font::BODY })
                .font(if tab.active {
                    font::MEDIUM
                } else {
                    font::REGULAR
                }),
        )
        .height(size::ACTION)
        .padding([0.0, space::MD])
        .style(if tab.active {
            styles::selected_inverse
        } else {
            styles::ghost
        })
        .on_press(on_select(index));
        group = group.push(control);
    }
    container(group)
        .padding(space::XXS)
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.sunken)),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .width(if panel_style {
            Length::Fill
        } else {
            Length::Shrink
        })
        .into()
}

/// Onglets de panneau (pleine largeur, fond de carte).
pub fn tabs<'a, Message: Clone + 'a>(
    entries: impl IntoIterator<Item = Tab>,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Element<'a, Message> {
    shell(entries, on_select, true)
}

/// Contrôle segmenté compact de toolbar.
pub fn segmented<'a, Message: Clone + 'a>(
    entries: impl IntoIterator<Item = Tab>,
    on_select: impl Fn(usize) -> Message + 'a,
) -> Element<'a, Message> {
    shell(entries, on_select, false)
}

#[cfg(test)]
mod tests {
    use super::Tab;

    #[test]
    fn l_onglet_actif_est_unique() {
        let tabs = [Tab::new("A", false), Tab::new("B", false)];
        assert_eq!(tabs.iter().filter(|t| t.active).count(), 0);
        let mut tabs = [Tab::new("A", true), Tab::new("B", false)];
        tabs[0].active = true;
        assert_eq!(tabs.iter().filter(|t| t.active).count(), 1);
    }
}
