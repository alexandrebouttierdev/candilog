//! Rail de navigation par groupes.
//!
//! Motif retenu : rail vertical à tuiles carrées, icône au-dessus du libellé.
//! L'état actif n'est pas un aplat pleine tuile : seule l'icône est posée dans
//! une pastille pleine et le libellé passe en accent. La masse colorée reste
//! ainsi minuscule, ce qui garde le rail calme.

use super::badge;
use super::icon::{self, Ink};
use super::tooltip::{tip, Side};
use super::typo;
use crate::navigation::{Route, Section};
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Layout;
use iced::widget::{button, column, container, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Charge à traiter, agrégée par groupe.
#[derive(Debug, Clone, Copy, Default)]
pub struct Load {
    /// Candidatures en attente d'action.
    pub candidatures: usize,
    /// Événements à venir dans l'agenda.
    pub agenda: usize,
    /// Relances arrivées à échéance.
    pub relances: usize,
}

impl Load {
    /// Compteur affiché sur la tuile d'un groupe : somme des charges de ses
    /// écrans, pour que le rail ne mente jamais sur le contenu du volet.
    #[must_use]
    pub const fn badge_for(self, section: Section) -> Option<usize> {
        let total = match section {
            Section::Recherche => self.candidatures + self.agenda + self.relances,
            _ => 0,
        };
        if total > 0 {
            Some(total)
        } else {
            None
        }
    }

    /// Compteur affiché en regard d'un écran, dans le volet.
    #[must_use]
    pub const fn badge_for_route(self, route: Route) -> Option<usize> {
        let total = match route {
            Route::Candidatures => self.candidatures,
            Route::Calendrier => self.agenda + self.relances,
            _ => 0,
        };
        if total > 0 {
            Some(total)
        } else {
            None
        }
    }
}

/// Construit le rail complet : marque, tuiles de groupe, pied.
pub fn rail<'a, Message: Clone + 'a>(
    active: Section,
    load: Load,
    layout: Layout,
    on_navigate: impl Fn(Section) -> Message + Copy + 'a,
    on_toggle_theme: Message,
    is_dark: bool,
) -> Element<'a, Message> {
    let compact = layout.rail_compact();

    let mut tiles = column![]
        .spacing(space::XS)
        .width(Length::Fill)
        .align_x(Alignment::Center);
    for section in Section::ALL {
        tiles = tiles.push(tile(
            section.icon(),
            section.short_label(),
            section.long_label(),
            section == active,
            load.badge_for(section),
            compact,
            on_navigate(section),
        ));
    }

    let footer = tile(
        if is_dark { icon::Icon::Sun } else { icon::Icon::Moon },
        if is_dark { "Nuit" } else { "Jour" },
        "Changer d'apparence",
        false,
        None,
        compact,
        on_toggle_theme,
    );

    let content = column![
        brand(compact),
        super::surface::divider(),
        container(tiles).padding([space::SM, space::XS]),
        Space::with_height(Length::Fill),
        super::surface::divider(),
        container(footer).padding([space::SM, space::XS]),
    ]
    .width(Length::Fill)
    .align_x(Alignment::Center);

    container(content)
        .width(Length::Fixed(layout.rail_width()))
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.chrome)),
                text_color: Some(palette.text),
                // Bord franc à droite : c'est ce qui fait lire le rail comme un
                // panneau d'application et non comme un aplat de couleur.
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// En-tête du rail : la marque reprend le motif des tuiles, en plus discret.
fn brand<'a, Message: 'a>(compact: bool) -> Element<'a, Message> {
    let mut block = column![icon::brand(size::PASTILLE * 0.75)]
        .align_x(Alignment::Center)
        .spacing(space::XS);
    if !compact {
        block = block.push(typo::caption("Candilog"));
    }
    container(block)
        .width(Length::Fill)
        .padding([space::LG, space::XS])
        .align_x(Alignment::Center)
        .into()
}

/// Fond, contour et teinte d'icône de la pastille, selon l'état.
///
/// La pastille inactive utilise `panel` et non `sunken` : sur le `chrome` du
/// rail, `sunken` serait indiscernable, et la forme carrée disparaîtrait pour
/// toutes les tuiles sauf l'active.
fn pastille_colors(active: bool, palette: &crate::ui::theme::Tokens) -> (iced::Color, iced::Color) {
    if active {
        (palette.accent_fill, palette.accent_fill)
    } else {
        (palette.panel, palette.border)
    }
}

/// Encrage de l'icône d'une pastille : second signal, indépendant du fond.
const fn pastille_ink(active: bool) -> Ink {
    if active {
        Ink::OnAccent
    } else {
        Ink::Muted
    }
}

/// Une tuile carrée : pastille d'icône au-dessus, libellé dessous.
fn tile<'a, Message: Clone + 'a>(
    kind: icon::Icon,
    short: &'a str,
    long: &'a str,
    active: bool,
    count: Option<usize>,
    compact: bool,
    message: Message,
) -> Element<'a, Message> {
    let side = if compact {
        size::NAV_TILE_COMPACT
    } else {
        size::NAV_TILE
    };

    let pastille = container(icon::icon(kind, icon::MD, pastille_ink(active)))
    .width(Length::Fixed(size::PASTILLE))
    .height(Length::Fixed(size::PASTILLE))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |theme: &Theme| {
        let palette = tokens(theme);
        let (fill, edge) = pastille_colors(active, &palette);
        container::Style {
            background: Some(Background::Color(fill)),
            border: Border {
                color: edge,
                width: stroke::HAIRLINE,
                radius: radius::CONTROL.into(),
            },
            ..container::Style::default()
        }
    });

    let marked: Element<'a, Message> = match count {
        Some(value) => iced::widget::row![pastille, badge::count(value)]
            .spacing(space::XXS)
            .align_y(Alignment::Start)
            .into(),
        None => pastille.into(),
    };

    let mut content = column![marked]
        .align_x(Alignment::Center)
        .spacing(space::XS);

    if !compact {
        content = content.push(
            typo::caption(short)
                .style(move |theme: &Theme| {
                    let palette = tokens(theme);
                    iced::widget::text::Style {
                        color: Some(if active {
                            palette.accent
                        } else {
                            palette.text_secondary
                        }),
                    }
                })
                // Le libellé ne doit jamais faire grandir la tuile : il est
                // coupé plutôt que replié, l'infobulle donne l'intitulé complet.
                .wrapping(iced::widget::text::Wrapping::None),
        );
    }

    let control = button(
        container(content)
            .width(Length::Fixed(side))
            .height(Length::Fixed(side))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
    .padding(0)
    .on_press(message)
    .style(move |theme: &Theme, status| {
        let palette = tokens(theme);
        let background = if active {
            Some(palette.selection)
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(palette.hover)
        } else {
            None
        };
        button::Style {
            background: background.map(Background::Color),
            text_color: palette.text,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: radius::CONTROL.into(),
            },
            shadow: iced::Shadow::default(),
        }
    });

    tip(control, long, Side::Right)
}

#[cfg(test)]
mod tests {
    use super::Load;
    use crate::navigation::{Route, Section};

    #[test]
    fn une_charge_nulle_n_affiche_aucun_compteur() {
        let load = Load::default();
        for section in Section::ALL {
            assert_eq!(load.badge_for(section), None);
        }
        for route in Route::ALL {
            assert_eq!(load.badge_for_route(route), None);
        }
    }

    /// La tuile d'un groupe porte la somme des charges de ses écrans.
    #[test]
    fn le_groupe_porte_la_somme_des_charges_de_ses_ecrans() {
        let load = Load {
            candidatures: 4,
            agenda: 2,
            relances: 3,
        };
        assert_eq!(load.badge_for(Section::Recherche), Some(9));
        assert_eq!(load.badge_for(Section::Pilotage), None);
        assert_eq!(load.badge_for(Section::Relations), None);
        assert_eq!(load.badge_for(Section::Documents), None);
        assert_eq!(load.badge_for(Section::Systeme), None);
    }

    /// Le détail reste lisible écran par écran dans le volet.
    #[test]
    fn le_detail_par_ecran_reste_disponible() {
        let load = Load {
            candidatures: 4,
            agenda: 2,
            relances: 3,
        };
        assert_eq!(load.badge_for_route(Route::Candidatures), Some(4));
        assert_eq!(load.badge_for_route(Route::Calendrier), Some(5));
        assert_eq!(load.badge_for_route(Route::Statistiques), None);
        assert_eq!(load.badge_for_route(Route::Parametres), None);
    }

    /// La somme du groupe doit égaler la somme du détail de ses écrans : sans
    /// cela, le compteur du rail mentirait sur ce que contient le volet.
    #[test]
    fn la_somme_du_groupe_egale_le_detail_de_ses_ecrans() {
        let load = Load {
            candidatures: 4,
            agenda: 2,
            relances: 3,
        };
        for section in Section::ALL {
            let detail: usize = Route::of_section(section)
                .into_iter()
                .filter_map(|route| load.badge_for_route(route))
                .sum();
            assert_eq!(load.badge_for(section), (detail > 0).then_some(detail));
        }
    }

    /// L'entrée active se distingue par deux signaux indépendants : le
    /// remplissage de la pastille et l'encrage de son icône. La couleur seule
    /// ne porte jamais l'information.
    #[test]
    fn l_entree_active_se_distingue_doublement() {
        use crate::ui::theme::tokens::{DAY, NIGHT};

        for palette in [NIGHT, DAY] {
            let (fond_actif, _) = super::pastille_colors(true, &palette);
            let (fond_inactif, _) = super::pastille_colors(false, &palette);
            assert_ne!(fond_actif, fond_inactif, "fonds de pastille identiques");
            assert_eq!(fond_actif, palette.accent_fill);
        }
        assert_ne!(
            super::pastille_ink(true),
            super::pastille_ink(false),
            "encrages d'icône identiques"
        );
    }

    /// Les cinq tuiles, la marque et le pied doivent tenir dans la hauteur
    /// minimale de la fenêtre : un rail qui scrolle perd sa raison d'être.
    #[test]
    fn le_rail_tient_dans_la_hauteur_minimale() {
        use crate::ui::theme::layout::MIN_HEIGHT;
        use crate::ui::theme::metrics::{size, space};

        let tuiles = Section::ALL.len() as f32 * size::NAV_TILE
            + (Section::ALL.len() as f32 - 1.0) * space::XS;
        let marque = size::PASTILLE * 0.75 + 2.0 * space::LG + 16.0;
        let pied = size::NAV_TILE + 2.0 * space::SM;
        assert!(
            marque + tuiles + pied <= MIN_HEIGHT,
            "le rail déborde de la hauteur minimale de la fenêtre"
        );
    }
}
