//! Iconographie vectorielle monochrome de Candilog.
//!
//! Une seule famille, une seule épaisseur de trait (1,5 px sur une grille de
//! 24), trois tailles optiques. Les icônes sont embarquées dans l'exécutable et
//! teintées par le thème : l'énumération garantit qu'aucun chemin invalide ne
//! peut être demandé à l'exécution.

use crate::ui::theme::color::Tone;
use crate::ui::theme::tokens::tokens;
use iced::widget::{svg, Svg};
use iced::Theme;

macro_rules! icon_set {
    ($($variant:ident => $file:literal),* $(,)?) => {
        /// Icônes embarquées dans l'exécutable.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $(
                #[doc = concat!("Icône `", $file, "`.")]
                $variant,
            )*
        }

        impl Icon {
            const fn bytes(self) -> &'static [u8] {
                match self {
                    $(Self::$variant => include_bytes!(
                        concat!("../../../assets/icons/", $file)
                    ),)*
                }
            }

            /// Toutes les icônes du jeu, utilisée par les tests de cohérence.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];
        }
    };
}

icon_set! {
    Brand => "candilog-icon-v2.svg",

    Alert => "actions/alert.svg",
    ArrowLeft => "actions/arrow-left.svg",
    ArrowRight => "actions/arrow-right.svg",
    Check => "actions/check.svg",
    ChevronDown => "actions/chevron-down.svg",
    ChevronRight => "actions/chevron-right.svg",
    ChevronUp => "actions/chevron-up.svg",
    Clock => "actions/clock.svg",
    Close => "actions/close.svg",
    Copy => "actions/copy.svg",
    Download => "actions/download.svg",
    Edit => "actions/edit.svg",
    Filter => "actions/filter.svg",
    Inbox => "actions/inbox.svg",
    Info => "actions/info.svg",
    Link => "actions/link.svg",
    Location => "actions/location.svg",
    Mail => "actions/mail.svg",
    Moon => "actions/moon.svg",
    More => "actions/more.svg",
    Open => "actions/open.svg",
    Panel => "actions/panel.svg",
    Phone => "actions/phone.svg",
    Plus => "actions/plus.svg",
    Refresh => "actions/refresh.svg",
    Save => "actions/save.svg",
    Search => "actions/search.svg",
    Stop => "actions/stop.svg",
    Sun => "actions/sun.svg",
    Target => "actions/target.svg",
    Trash => "actions/trash.svg",

    Applications => "navigation/applications.svg",
    Building => "navigation/building.svg",
    Calendar => "navigation/calendar.svg",
    Chart => "navigation/chart.svg",
    Dashboard => "navigation/dashboard.svg",
    Document => "navigation/document.svg",
    Import => "navigation/import.svg",
    Letter => "navigation/letter.svg",
    Network => "navigation/network.svg",
    Profile => "navigation/profile.svg",
    Settings => "navigation/settings.svg",
    Sparkles => "navigation/sparkles.svg",
}

/// Taille optique d'une icône de ligne ou de bouton.
pub const SM: f32 = 16.0;
/// Taille optique d'une icône de navigation, portée par une pastille.
pub const MD: f32 = 18.0;
/// Taille optique d'une icône de toolbar ou d'en-tête.
pub const LG: f32 = 20.0;

const _: () = assert!(SM < MD, "tailles optiques non ordonnées");
const _: () = assert!(MD < LG, "tailles optiques non ordonnées");
const _: () = assert!(
    LG <= 20.0,
    "densité Confort : icône disproportionnée par rapport au texte"
);

/// Rôle d'encrage d'une icône.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ink {
    /// Icône d'accompagnement, en retrait.
    #[default]
    Muted,
    /// Icône portée par un texte principal.
    Strong,
    /// Icône d'un élément actif ou sélectionné.
    Accent,
    /// Icône posée sur une surface d'accent pleine.
    OnAccent,
    /// Icône teintée par un ton sémantique.
    Toned(Tone),
}

/// Construit une icône teintée selon son rôle.
pub fn icon<'a>(kind: Icon, size: f32, ink: Ink) -> Svg<'a> {
    svg(svg::Handle::from_memory(kind.bytes()))
        .width(size)
        .height(size)
        .style(move |theme: &Theme, _status| {
            let palette = tokens(theme);
            svg::Style {
                color: Some(match ink {
                    Ink::Muted => palette.text_secondary,
                    Ink::Strong => palette.text,
                    Ink::Accent => palette.accent,
                    Ink::OnAccent => palette.on_accent,
                    Ink::Toned(tone) => tone.color(&palette),
                }),
            }
        })
}

/// Icône d'accompagnement à la taille d'une ligne.
pub fn muted<'a>(kind: Icon) -> Svg<'a> {
    icon(kind, SM, Ink::Muted)
}

/// Icône portée par un texte principal.
pub fn strong<'a>(kind: Icon) -> Svg<'a> {
    icon(kind, SM, Ink::Strong)
}

/// Icône teintée par un ton sémantique.
pub fn toned<'a>(kind: Icon, tone: Tone) -> Svg<'a> {
    icon(kind, SM, Ink::Toned(tone))
}

/// Marque Candilog, non teintée.
pub fn brand<'a>(size: f32) -> Svg<'a> {
    svg(svg::Handle::from_memory(Icon::Brand.bytes()))
        .width(size)
        .height(size)
}

#[cfg(test)]
mod tests {
    use super::{Icon, Ink};
    use crate::ui::theme::color::Tone;

    #[test]
    fn toutes_les_icones_sont_embarquees_et_non_vides() {
        for kind in Icon::ALL {
            assert!(kind.bytes().len() > 32, "icône vide ou tronquée : {kind:?}");
        }
    }

    #[test]
    fn toutes_les_icones_partagent_la_meme_epaisseur_de_trait() {
        for kind in Icon::ALL {
            if matches!(kind, Icon::Brand) {
                continue;
            }
            let source = std::str::from_utf8(kind.bytes()).expect("SVG non UTF-8");
            assert!(
                source.contains("stroke-width=\"1.5\""),
                "épaisseur de trait non conforme : {kind:?}"
            );
        }
    }

    #[test]
    fn toutes_les_icones_partagent_la_meme_grille() {
        for kind in Icon::ALL {
            if matches!(kind, Icon::Brand) {
                continue;
            }
            let source = std::str::from_utf8(kind.bytes()).expect("SVG non UTF-8");
            assert!(
                source.contains("viewBox=\"0 0 24 24\""),
                "grille optique non conforme : {kind:?}"
            );
        }
    }

    #[test]
    fn aucune_icone_n_est_declaree_deux_fois() {
        let mut seen = std::collections::BTreeSet::new();
        for kind in Icon::ALL {
            assert!(seen.insert(format!("{kind:?}")), "icône dupliquée");
        }
        assert_eq!(seen.len(), Icon::ALL.len());
    }

    #[test]
    fn encrage_par_defaut_reste_discret() {
        assert_eq!(Ink::default(), Ink::Muted);
        assert_ne!(Ink::Muted, Ink::Toned(Tone::Danger));
    }
}
