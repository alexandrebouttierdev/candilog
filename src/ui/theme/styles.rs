//! Styles Iced dérivés des jetons Candilog.
//!
//! Aucun écran ne redéfinit le style d'un widget standard : tout passe par ce
//! module, qui garantit la cohérence des états normal / survol / pression /
//! focus / sélection / désactivé.

use super::color::Tone;
use super::metrics::{elevation, radius, stroke};
use super::tokens::{alpha, tokens, Tokens};
use iced::widget::{
    button, container, pick_list, progress_bar, rule, scrollable, slider, text, text_editor,
    text_input, toggler,
};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Décale une couleur vers le clair (thème sombre) ou le foncé (thème clair).
fn press(color: Color, palette: &Tokens) -> Color {
    let delta = if palette.is_dark { 0.04 } else { -0.04 };
    Color {
        r: (color.r + delta).clamp(0.0, 1.0),
        g: (color.g + delta).clamp(0.0, 1.0),
        b: (color.b + delta).clamp(0.0, 1.0),
        a: color.a,
    }
}

const fn no_border(radius: f32) -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: iced::border::Radius {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        },
    }
}

// --------------------------------------------------------------------------
// Boutons
// --------------------------------------------------------------------------

/// Action principale : au plus une par écran.
pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let background = match status {
        button::Status::Hovered => palette.accent_hover,
        button::Status::Pressed => press(palette.accent_fill, &palette),
        button::Status::Disabled => alpha(palette.accent_fill, 0.45),
        button::Status::Active => palette.accent_fill,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if matches!(status, button::Status::Disabled) {
            alpha(palette.on_accent, 0.7)
        } else {
            palette.on_accent
        },
        border: no_border(radius::CONTROL),
        shadow: Shadow {
            color: Color {
                a: 0.35,
                ..palette.accent_fill
            },
            offset: Vector::new(0.0, 3.0),
            blur_radius: 12.0,
        },
    }
}

/// Action secondaire d'un dialogue ou d'une section.
pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let background = match status {
        button::Status::Hovered => palette.hover,
        button::Status::Pressed => press(palette.raised, &palette),
        _ => palette.raised,
    };
    let disabled = matches!(status, button::Status::Disabled);
    button::Style {
        background: Some(Background::Color(background)),
        text_color: alpha(palette.text, if disabled { 0.45 } else { 1.0 }),
        border: Border {
            color: palette.border_strong,
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        shadow: Shadow::default(),
    }
}

/// Contrôle de toolbar ou action de ligne, sans surface au repos.
pub fn ghost(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let background = match status {
        button::Status::Hovered => Some(palette.hover),
        button::Status::Pressed => Some(press(palette.hover, &palette)),
        _ => None,
    };
    let disabled = matches!(status, button::Status::Disabled);
    button::Style {
        background: background.map(Background::Color),
        text_color: alpha(palette.text_secondary, if disabled { 0.45 } else { 1.0 }),
        border: no_border(radius::CONTROL),
        shadow: Shadow::default(),
    }
}

/// Variante de `ghost` dont le libellé reste en texte principal.
pub fn ghost_strong(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    button::Style {
        text_color: alpha(
            palette.text,
            if matches!(status, button::Status::Disabled) {
                0.45
            } else {
                1.0
            },
        ),
        ..ghost(theme, status)
    }
}

/// Action destructive : discrète au repos, explicite au survol.
pub fn danger(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let engaged = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: engaged.then(|| Background::Color(Tone::Danger.surface(&palette))),
        text_color: alpha(
            palette.danger,
            if matches!(status, button::Status::Disabled) {
                0.45
            } else {
                1.0
            },
        ),
        border: Border {
            color: if engaged {
                Tone::Danger.edge(&palette)
            } else {
                Color::TRANSPARENT
            },
            width: if engaged { stroke::HAIRLINE } else { 0.0 },
            radius: radius::CONTROL.into(),
        },
        shadow: Shadow::default(),
    }
}

/// Action destructive appuyée, réservée aux confirmations de dialogue.
pub fn danger_filled(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let background = match status {
        button::Status::Hovered => press(palette.danger, &palette),
        button::Status::Pressed => press(press(palette.danger, &palette), &palette),
        button::Status::Disabled => alpha(palette.danger, 0.45),
        button::Status::Active => palette.danger,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if palette.is_dark {
            palette.chrome
        } else {
            Color::WHITE
        },
        border: no_border(radius::CONTROL),
        shadow: Shadow::default(),
    }
}

/// Segment actif d'un contrôle segmenté ou ligne sélectionnée.
pub fn selected(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let background = if matches!(status, button::Status::Pressed) {
        press(palette.raised, &palette)
    } else {
        palette.selection
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.accent,
        border: no_border(radius::CONTROL),
        shadow: Shadow::default(),
    }
}

/// Segment actif : inversion totale (texte = fond, fond = texte) — Tabs candilog-desktop.
pub fn selected_inverse(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = tokens(theme);
    button::Style {
        background: Some(Background::Color(palette.text)),
        text_color: palette.canvas,
        border: Border {
            radius: radius::CONTROL.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
    }
}

/// Entrée de navigation de la barre latérale.
pub fn nav_item(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = tokens(theme);
        let background = if active {
            Some(palette.selection)
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(alpha(
                palette.text,
                if palette.is_dark { 0.06 } else { 0.05 },
            ))
        } else {
            None
        };
        button::Style {
            background: background.map(Background::Color),
            text_color: if active {
                palette.text
            } else {
                palette.text_secondary
            },
            border: no_border(radius::CONTROL),
            shadow: Shadow::default(),
        }
    }
}

/// Ligne de données cliquable d'une table ou d'une liste.
pub fn row_item(selected_row: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = tokens(theme);
        let background = if selected_row {
            Some(palette.selection)
        } else if matches!(status, button::Status::Hovered) {
            Some(palette.hover)
        } else if matches!(status, button::Status::Pressed) {
            Some(press(palette.hover, &palette))
        } else {
            None
        };
        button::Style {
            background: background.map(Background::Color),
            text_color: palette.text,
            border: no_border(radius::NONE),
            shadow: Shadow::default(),
        }
    }
}

/// Carte Kanban : verre léger, saisissable, ombre subtile au repos.
pub fn card(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    let engaged = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: Some(Background::Color(if engaged {
            palette.raised
        } else {
            mix_panel(palette.panel, palette.canvas, 0.75)
        })),
        text_color: palette.text,
        border: Border {
            color: if engaged {
                palette.border_strong
            } else {
                alpha(palette.border, 0.60)
            },
            width: stroke::HAIRLINE,
            radius: radius::CARD.into(),
        },
        shadow: if engaged {
            Shadow {
                color: alpha(palette.shadow, palette.shadow.a * 0.5),
                offset: Vector::new(0.0, 3.0),
                blur_radius: 10.0,
            }
        } else {
            Shadow {
                color: alpha(palette.shadow, palette.shadow.a * 0.35),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            }
        },
    }
}

/// Carte Kanban sélectionnée : filet d'accent, sans fond saturé.
pub fn card_selected(theme: &Theme, status: button::Status) -> button::Style {
    let palette = tokens(theme);
    button::Style {
        background: Some(Background::Color(palette.raised)),
        border: Border {
            color: palette.accent,
            width: stroke::MARKER,
            radius: radius::CARD.into(),
        },
        ..card(theme, status)
    }
}

// --------------------------------------------------------------------------
// Conteneurs
// --------------------------------------------------------------------------

/// Fond du plan de travail.
pub fn canvas(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.canvas)),
        text_color: Some(palette.text),
        ..container::Style::default()
    }
}

/// Panneau de contenu principal : verre dépoli approximé (card 55 %, sombre 48 %).
pub fn glass_panel(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    let ratio = if palette.is_dark { 0.48 } else { 0.55 };
    let background = mix(palette.panel, palette.canvas, ratio);
    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(palette.text),
        border: Border {
            color: if palette.is_dark {
                Color {
                    a: 0.10,
                    ..Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }
                }
            } else {
                Color {
                    a: 0.70,
                    ..palette.border
                }
            },
            width: stroke::HAIRLINE,
            radius: radius::PANEL.into(),
        },
        shadow: Shadow {
            color: palette.shadow,
            offset: Vector::new(0.0, elevation::GLASS_OFFSET),
            blur_radius: elevation::GLASS_BLUR,
        },
    }
}

/// Carte en verre (sidebar, cartes de contenu) : card 55 %, bordure 60 %.
pub fn glass_card(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    let background = mix(palette.panel, palette.canvas, 0.55);
    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(palette.text),
        border: Border {
            color: if palette.is_dark {
                Color {
                    a: 0.08,
                    ..Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }
                }
            } else {
                Color {
                    a: 0.60,
                    ..palette.border
                }
            },
            width: stroke::HAIRLINE,
            radius: radius::CARD.into(),
        },
        ..container::Style::default()
    }
}

/// Mélange le panneau sur le fond ambiant (verre approximé).
#[must_use]
pub fn mix_panel(panel: Color, canvas: Color, ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    Color {
        r: panel.r * ratio + canvas.r * (1.0 - ratio),
        g: panel.g * ratio + canvas.g * (1.0 - ratio),
        b: panel.b * ratio + canvas.b * (1.0 - ratio),
        a: 1.0,
    }
}

/// Mélange deux couleurs : `ratio` de `front` sur `back`.
fn mix(front: Color, back: Color, ratio: f32) -> Color {
    mix_panel(front, back, ratio)
}

/// Barre latérale et barre d'état.
pub fn chrome(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.chrome)),
        text_color: Some(palette.text),
        ..container::Style::default()
    }
}

/// Panneau de données.
pub fn panel(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.panel)),
        text_color: Some(palette.text),
        border: Border {
            color: Color {
                a: 0.60,
                ..palette.border
            },
            width: stroke::HAIRLINE,
            radius: radius::PANEL.into(),
        },
        shadow: Shadow {
            color: palette.shadow,
            offset: Vector::new(0.0, 18.0),
            blur_radius: 55.0,
        },
    }
}

/// Panneau sans rayon, utilisé quand il touche les bords d'une zone.
pub fn panel_flat(theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            radius: radius::NONE.into(),
            ..panel(theme).border
        },
        ..panel(theme)
    }
}

/// État vide : bordure pointillée sur fond de carte.
pub fn dashed(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.35,
            ..palette.panel
        })),
        border: Border {
            color: palette.border_strong,
            width: stroke::HAIRLINE,
            radius: radius::PANEL.into(),
            // Iced 0.13 ne supporte pas les pointillés : filet simple renforcé.
        },
        ..container::Style::default()
    }
}

/// Surface en creux : en-tête de table, plan de travail d'un document.
pub fn sunken(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.sunken)),
        text_color: Some(palette.text),
        border: Border {
            color: palette.border,
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        shadow: Shadow::default(),
    }
}

/// Creux sans rayon, quand il touche les bords d'une zone.
pub fn sunken_flat(theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            radius: radius::NONE.into(),
            ..sunken(theme).border
        },
        ..sunken(theme)
    }
}

/// Colonne Kanban : verre léger, bordure discrète, sans ombre.
pub fn kanban_column(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(mix_panel(
            palette.panel,
            palette.canvas,
            0.20,
        ))),
        text_color: Some(palette.text),
        border: Border {
            color: palette.border,
            width: stroke::HAIRLINE,
            radius: radius::PANEL.into(),
        },
        shadow: Shadow::default(),
    }
}

/// Zone de dépôt d'une colonne Kanban : neutre au repos, marquée d'accent
/// pendant un glisser-déposer (le pointillé se simule par un filet renforcé).
pub fn drop_zone(active: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = tokens(theme);
        container::Style {
            background: active.then_some(Background::Color(palette.selection)),
            border: Border {
                color: if active {
                    palette.border_strong
                } else {
                    palette.border
                },
                width: stroke::HAIRLINE,
                radius: radius::PANEL.into(),
            },
            ..container::Style::default()
        }
    }
}

/// Bande d'en-tête d'une liste en verre : creux adouci, coins carrés.
pub fn list_header(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(mix_panel(
            palette.sunken,
            palette.canvas,
            0.25,
        ))),
        text_color: Some(palette.text_secondary),
        border: Border {
            color: palette.border_strong,
            width: stroke::HAIRLINE,
            radius: radius::NONE.into(),
        },
        shadow: Shadow::default(),
    }
}

/// Surface surélevée : menu, dialogue, feuille.
pub fn raised(theme: &Theme) -> container::Style {
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
            offset: Vector::new(0.0, elevation::OFFSET),
            blur_radius: elevation::BLUR,
        },
    }
}

/// Voile posé sous une modale.
pub fn scrim(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(tokens(theme).scrim)),
        ..container::Style::default()
    }
}

/// Surface teintée par un ton sémantique.
pub fn toned(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(tone.surface(&palette))),
            text_color: Some(tone.color(&palette)),
            border: Border {
                color: tone.edge(&palette),
                width: stroke::HAIRLINE,
                radius: radius::PILL.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

/// Bandeau d'alerte ambre : fond teinté discret et filet ambre (relances à faire).
pub fn amber_band(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(mix(
            palette.warning,
            palette.canvas,
            0.04,
        ))),
        text_color: Some(palette.text),
        border: Border {
            color: alpha(palette.warning, 0.20),
            width: stroke::HAIRLINE,
            radius: radius::PANEL.into(),
        },
        shadow: Shadow::default(),
    }
}

/// Surface pleine d'une couleur arbitraire, réservée aux marqueurs dessinés.
pub fn marker(color: Color, corner: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(color)),
        border: no_border(corner),
        ..container::Style::default()
    }
}

/// Contour dessiné sans remplissage, pour un marqueur creux.
pub fn marker_outline(color: Color, corner: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: None,
        border: Border {
            color,
            width: 1.5,
            radius: corner.into(),
        },
        ..container::Style::default()
    }
}

// --------------------------------------------------------------------------
// Champs
// --------------------------------------------------------------------------

/// Champ texte compact.
pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = tokens(theme);
    let focused = matches!(status, text_input::Status::Focused);
    let disabled = matches!(status, text_input::Status::Disabled);
    text_input::Style {
        background: Background::Color(if disabled {
            palette.panel
        } else {
            mix(palette.panel, palette.canvas, 0.60)
        }),
        border: Border {
            color: if focused {
                palette.accent
            } else {
                palette.border_strong
            },
            width: if focused {
                stroke::FOCUS
            } else {
                stroke::HAIRLINE
            },
            radius: radius::FIELD.into(),
        },
        icon: palette.text_muted,
        placeholder: palette.text_muted,
        value: alpha(palette.text, if disabled { 0.45 } else { 1.0 }),
        selection: palette.selection,
    }
}

/// Éditeur multiligne assorti aux champs simples.
pub fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let palette = tokens(theme);
    let focused = matches!(status, text_editor::Status::Focused);
    let disabled = matches!(status, text_editor::Status::Disabled);
    text_editor::Style {
        background: Background::Color(if disabled {
            palette.panel
        } else {
            palette.sunken
        }),
        border: Border {
            color: if focused {
                palette.accent
            } else {
                palette.border_strong
            },
            width: if focused {
                stroke::FOCUS
            } else {
                stroke::HAIRLINE
            },
            radius: radius::CONTROL.into(),
        },
        icon: palette.text_muted,
        placeholder: palette.text_muted,
        value: alpha(palette.text, if disabled { 0.45 } else { 1.0 }),
        selection: palette.selection,
    }
}

/// Sélecteur assorti aux champs texte.
pub fn select(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = tokens(theme);
    let engaged = matches!(
        status,
        pick_list::Status::Hovered | pick_list::Status::Opened
    );
    pick_list::Style {
        text_color: palette.text,
        placeholder_color: palette.text_muted,
        handle_color: palette.text_secondary,
        background: Background::Color(palette.sunken),
        border: Border {
            color: if engaged {
                palette.accent
            } else {
                palette.border_strong
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
    }
}

/// Liste déroulante d'un sélecteur.
pub fn select_menu(theme: &Theme) -> iced::overlay::menu::Style {
    let palette = tokens(theme);
    iced::overlay::menu::Style {
        background: Background::Color(palette.raised),
        border: Border {
            color: palette.border_strong,
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        text_color: palette.text,
        selected_background: Background::Color(palette.selection),
        selected_text_color: palette.accent,
    }
}

/// Interrupteur d'une ligne de réglage.
pub fn switch(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let palette = tokens(theme);
    let (active, hovered) = match status {
        toggler::Status::Active { is_toggled } => (is_toggled, false),
        toggler::Status::Hovered { is_toggled } => (is_toggled, true),
        toggler::Status::Disabled => (false, false),
    };
    let track = if active {
        if hovered {
            palette.accent_hover
        } else {
            palette.accent_fill
        }
    } else if hovered {
        palette.hover
    } else {
        palette.sunken
    };
    toggler::Style {
        background: track,
        background_border_width: stroke::HAIRLINE,
        background_border_color: if active {
            palette.accent_fill
        } else {
            palette.border_strong
        },
        foreground: if active {
            palette.on_accent
        } else {
            palette.text_secondary
        },
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
    }
}

/// Curseur d'un réglage continu.
pub fn range(theme: &Theme, status: slider::Status) -> slider::Style {
    let palette = tokens(theme);
    let engaged = matches!(status, slider::Status::Hovered | slider::Status::Dragged);
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(palette.accent_fill),
                Background::Color(palette.sunken),
            ),
            width: 3.0,
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: radius::PILL.into(),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle {
                radius: if engaged { 7.0 } else { 6.0 },
            },
            background: Background::Color(if engaged {
                palette.accent_hover
            } else {
                palette.accent_fill
            }),
            border_width: stroke::FOCUS,
            border_color: palette.panel,
        },
    }
}

// --------------------------------------------------------------------------
// Indicateurs et filets
// --------------------------------------------------------------------------

/// Barre de progression teintée par un ton sémantique.
pub fn progress(tone: Tone) -> impl Fn(&Theme) -> progress_bar::Style {
    move |theme| {
        let palette = tokens(theme);
        progress_bar::Style {
            background: Background::Color(palette.sunken),
            bar: Background::Color(tone.color(&palette)),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: radius::PILL.into(),
            },
        }
    }
}

/// Filet de séparation intérieur à un panneau.
pub fn divider(theme: &Theme) -> rule::Style {
    let palette = tokens(theme);
    rule::Style {
        color: palette.border,
        width: 1,
        radius: radius::NONE.into(),
        fill_mode: rule::FillMode::Full,
    }
}

/// Filet de séparation appuyé, sous un en-tête de table.
pub fn divider_strong(theme: &Theme) -> rule::Style {
    rule::Style {
        color: tokens(theme).border_strong,
        ..divider(theme)
    }
}

/// Barre de défilement fine, dans l'esprit des applications desktop.
pub fn scroller(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let palette = tokens(theme);
    let engaged = !matches!(status, scrollable::Status::Active);
    let rail = scrollable::Rail {
        background: None,
        border: no_border(radius::PILL),
        scroller: scrollable::Scroller {
            color: alpha(
                palette.text_secondary,
                if engaged {
                    0.55
                } else if palette.is_dark {
                    0.24
                } else {
                    0.30
                },
            ),
            border: no_border(radius::PILL),
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
    }
}

/// Texte secondaire adaptatif.
pub fn secondary_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).text_secondary),
    }
}

/// Texte indicatif adaptatif.
pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tokens(theme).text_muted),
    }
}

/// Texte teinté par un ton sémantique.
pub fn toned_text(tone: Tone) -> impl Fn(&Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(tone.resolve(theme)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canvas, card, chrome, danger, divider, drop_zone, ghost, glass_card, glass_panel,
        kanban_column, list_header, mix_panel, nav_item, panel, press, primary, progress, raised,
        row_item, scroller, secondary, selected, selected_inverse, sunken,
    };
    use crate::ui::theme::color::Tone;
    use crate::ui::theme::tokens::{alpha, tokens, NIGHT};
    use crate::ui::theme::{dark, light};
    use iced::widget::{button, scrollable};

    #[test]
    fn pression_eclaircit_en_sombre_et_assombrit_en_clair() {
        let base = iced::Color::from_rgb(0.5, 0.5, 0.5);
        assert!(press(base, &NIGHT).r > base.r);
        assert!(press(base, &super::super::tokens::DAY).r < base.r);
    }

    #[test]
    fn action_principale_change_a_chaque_etat() {
        for theme in [dark(), light()] {
            let states = [
                primary(&theme, button::Status::Active),
                primary(&theme, button::Status::Hovered),
                primary(&theme, button::Status::Pressed),
                primary(&theme, button::Status::Disabled),
            ];
            for (index, style) in states.iter().enumerate() {
                for other in &states[index + 1..] {
                    assert_ne!(
                        format!("{:?}", style.background),
                        format!("{:?}", other.background),
                        "deux états de l'action principale sont identiques"
                    );
                }
            }
        }
    }

    #[test]
    fn controles_discrets_n_ont_pas_de_surface_au_repos() {
        for theme in [dark(), light()] {
            assert!(ghost(&theme, button::Status::Active).background.is_none());
            assert!(danger(&theme, button::Status::Active).background.is_none());
            assert!(ghost(&theme, button::Status::Hovered).background.is_some());
            assert!(danger(&theme, button::Status::Hovered).background.is_some());
        }
    }

    #[test]
    fn aucune_ombre_sur_les_surfaces_statiques() {
        for theme in [dark(), light()] {
            assert!(sunken(&theme).shadow.blur_radius.abs() < f32::EPSILON);
            assert!(canvas(&theme).shadow.blur_radius.abs() < f32::EPSILON);
            assert!(chrome(&theme).shadow.blur_radius.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn la_carte_porte_une_ombre_subtile_au_repos() {
        for theme in [dark(), light()] {
            let repos = card(&theme, button::Status::Active);
            let survol = card(&theme, button::Status::Hovered);
            assert!(
                repos.shadow.blur_radius > 0.0,
                "carte au repos sans ombre subtile"
            );
            assert!(
                repos.shadow.blur_radius < survol.shadow.blur_radius,
                "ombre de survol pas plus marquée que celle du repos"
            );
        }
    }

    #[test]
    fn seules_les_surfaces_flottantes_portent_une_ombre() {
        for theme in [dark(), light()] {
            assert!(raised(&theme).shadow.blur_radius > 0.0);
            assert!(card(&theme, button::Status::Hovered).shadow.blur_radius > 0.0);
        }
    }

    /// Une surface surélevée doit porter une ombre réelle : c'est ce qui la
    /// détache d'un simple panneau. Sans elle, `raised` et `panel` se
    /// confondent dès que leurs teintes sont proches.
    #[test]
    fn une_surface_surelevee_porte_une_ombre() {
        for theme in [dark(), light()] {
            let style = raised(&theme);
            assert!(
                style.shadow.blur_radius > 0.0,
                "surface surélevée sans ombre"
            );
            assert!(style.shadow.color.a > 0.0, "ombre transparente");
        }
    }

    /// Le panneau porte désormais l'ombre douce du handoff (0 18px 55px).
    #[test]
    fn le_panneau_porte_une_ombre_douce() {
        for theme in [dark(), light()] {
            let style = panel(&theme);
            assert_eq!(style.shadow.offset.y, 18.0);
            assert_eq!(style.shadow.blur_radius, 55.0);
            assert!(style.shadow.color.a > 0.0, "ombre transparente");
        }
    }

    #[test]
    fn etat_selectionne_se_distingue_du_repos() {
        for theme in [dark(), light()] {
            let palette = tokens(&theme);
            assert_eq!(
                selected(&theme, button::Status::Active).text_color,
                palette.accent
            );
            let idle = nav_item(false)(&theme, button::Status::Active);
            let active = nav_item(true)(&theme, button::Status::Active);
            assert!(idle.background.is_none());
            assert!(active.background.is_some());
            assert_ne!(idle.text_color, active.text_color);
        }
    }

    #[test]
    fn segment_actif_inverse_le_texte_et_le_fond() {
        for theme in [dark(), light()] {
            let palette = tokens(&theme);
            let style = selected_inverse(&theme, button::Status::Active);
            assert_eq!(
                style.background,
                Some(iced::Background::Color(palette.text))
            );
            assert_eq!(style.text_color, palette.canvas);
        }
    }

    #[test]
    fn surfaces_en_verre_melangent_panneau_et_canvas() {
        let palette = tokens(&dark());
        assert_eq!(
            glass_panel(&dark()).background,
            Some(iced::Background::Color(mix_panel(
                palette.panel,
                palette.canvas,
                0.48
            )))
        );
        assert_eq!(
            glass_card(&dark()).background,
            Some(iced::Background::Color(mix_panel(
                palette.panel,
                palette.canvas,
                0.55
            )))
        );
        let palette = tokens(&light());
        assert_eq!(
            glass_panel(&light()).background,
            Some(iced::Background::Color(mix_panel(
                palette.panel,
                palette.canvas,
                0.55
            )))
        );
    }

    #[test]
    fn colonne_kanban_et_bande_de_liste_melangent_leurs_surfaces() {
        for theme in [dark(), light()] {
            let palette = tokens(&theme);
            assert_eq!(
                kanban_column(&theme).background,
                Some(iced::Background::Color(mix_panel(
                    palette.panel,
                    palette.canvas,
                    0.20
                )))
            );
            assert_eq!(
                list_header(&theme).background,
                Some(iced::Background::Color(mix_panel(
                    palette.sunken,
                    palette.canvas,
                    0.25
                )))
            );
        }
    }

    #[test]
    fn zone_de_depot_neutre_au_repos_et_marquee_au_survol() {
        for theme in [dark(), light()] {
            let palette = tokens(&theme);
            let repos = drop_zone(false)(&theme);
            let actif = drop_zone(true)(&theme);
            assert!(repos.background.is_none(), "zone au repos non transparente");
            assert_eq!(
                actif.background,
                Some(iced::Background::Color(palette.selection)),
                "zone survolée sans fond d'accent"
            );
            assert_eq!(repos.border.color, palette.border);
            assert_eq!(actif.border.color, palette.border_strong);
        }
    }

    #[test]
    fn mix_panel_interpole_les_couleurs() {
        let front = iced::Color::from_rgb(1.0, 0.0, 0.0);
        let back = iced::Color::from_rgb(0.0, 0.0, 1.0);
        assert_eq!(mix_panel(front, back, 0.0), back);
        assert_eq!(mix_panel(front, back, 1.0), front);
        let moitie = mix_panel(front, back, 0.5);
        assert!((moitie.r - 0.5).abs() < f32::EPSILON);
        assert!((moitie.b - 0.5).abs() < f32::EPSILON);
        assert_eq!(moitie.a, 1.0);
    }

    #[test]
    fn ligne_selectionnee_reste_plate() {
        let theme = dark();
        let style = row_item(true)(&theme, button::Status::Active);
        assert!(style.background.is_some());
        assert!(style.border.radius.top_left.abs() < f32::EPSILON);
    }

    #[test]
    fn secondaire_conserve_un_filet_visible() {
        for theme in [dark(), light()] {
            let style = secondary(&theme, button::Status::Active);
            assert!(style.border.width > 0.0);
        }
    }

    #[test]
    fn barre_de_progression_suit_son_ton() {
        let theme = dark();
        let style = progress(Tone::Success)(&theme);
        assert_eq!(
            format!("{:?}", style.bar),
            format!("{:?}", iced::Background::Color(NIGHT.success))
        );
    }

    #[test]
    fn le_bandeau_ambre_melange_4_pourcent_et_filet_a_20_pourcent() {
        for theme in [dark(), light()] {
            let palette = tokens(&theme);
            let style = super::amber_band(&theme);
            assert_eq!(
                style.background,
                Some(iced::Background::Color(super::mix(
                    palette.warning,
                    palette.canvas,
                    0.04
                )))
            );
            assert_eq!(style.border.color, alpha(palette.warning, 0.20));
        }
    }

    #[test]
    fn filet_reste_d_un_pixel() {
        for theme in [dark(), light()] {
            assert_eq!(divider(&theme).width, 1);
        }
    }

    #[test]
    fn barre_de_defilement_s_affirme_au_survol() {
        let theme = dark();
        let idle = scroller(&theme, scrollable::Status::Active);
        let hovered = scroller(
            &theme,
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
            },
        );
        assert!(hovered.vertical_rail.scroller.color.a > idle.vertical_rail.scroller.color.a);
    }
}
