//! Échelle typographique desktop de Candilog (famille Geist, monospace Geist Mono).
//!
//! Aucun texte d'interface ne dépasse 20 px. La hiérarchie s'obtient par la
//! graisse, la couleur et l'espacement.

use iced::font::{Family, Stretch, Style, Weight};
use iced::Font;

/// Famille sans embarquée dans l'exécutable.
pub const FAMILY: Family = Family::Name("Geist");
/// Famille monospace embarquée dans l'exécutable.
pub const MONO_FAMILY: Family = Family::Name("Geist Mono");

const fn font(family: Family, weight: Weight) -> Font {
    Font {
        family,
        weight,
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}

/// Graisse courante des textes et valeurs.
pub const REGULAR: Font = font(FAMILY, Weight::Normal);
/// Graisse des étiquettes, en-têtes de colonnes et éléments actifs.
pub const MEDIUM: Font = font(FAMILY, Weight::Medium);
/// Graisse des titres de section, de toolbar et des métriques.
pub const SEMIBOLD: Font = font(FAMILY, Weight::Semibold);
/// Chiffres et valeurs monospace, graisse courante.
pub const MONO_REGULAR: Font = font(MONO_FAMILY, Weight::Normal);
/// Chiffres monospace des métriques secondaires.
pub const MONO_MEDIUM: Font = font(MONO_FAMILY, Weight::Medium);
/// Chiffres monospace des métriques principales.
pub const MONO_SEMIBOLD: Font = font(MONO_FAMILY, Weight::Semibold);

/// Micro-texte et versions.
pub const MICRO: f32 = 10.0;
/// Légendes et dates en liste.
pub const CAPTION: f32 = 10.5;
/// Métadonnées et texte secondaire.
pub const META: f32 = 11.5;
/// Étiquettes de formulaire et en-têtes de colonnes.
pub const LABEL: f32 = 12.0;
/// Corps de texte et valeurs.
pub const BODY: f32 = 13.0;
/// Titre d'un objet de liste ou de carte.
pub const ITEM: f32 = 13.5;
/// Titre de panneau.
pub const SECTION: f32 = 15.0;
/// Titre de toolbar et d'écran.
pub const TITLE: f32 = 20.0;
/// Valeur d'indicateur (Geist Mono semibold au rendu).
pub const METRIC: f32 = 28.0;
/// Valeur d'indicateur mise en avant.
pub const DISPLAY: f32 = 34.0;

const _: () = assert!(MICRO >= 9.0, "texte trop petit pour un écran desktop");
const _: () = assert!(TITLE <= 20.0, "titre d'interface disproportionné");

#[cfg(test)]
mod tests {
    use super::{
        BODY, CAPTION, DISPLAY, ITEM, LABEL, MEDIUM, META, METRIC, MICRO, MONO_REGULAR,
        MONO_SEMIBOLD, REGULAR, SECTION, SEMIBOLD, TITLE,
    };
    use iced::font::Weight;

    #[test]
    fn echelle_est_strictement_croissante() {
        let scale = [
            MICRO, CAPTION, META, LABEL, BODY, ITEM, SECTION, TITLE, METRIC, DISPLAY,
        ];
        for pair in scale.windows(2) {
            assert!(pair[0] < pair[1], "échelle typographique non ordonnée");
        }
    }

    #[test]
    fn interface_ne_depasse_jamais_vingt_pixels() {
        for size in [MICRO, CAPTION, META, LABEL, BODY, ITEM, SECTION, TITLE] {
            assert!(size <= TITLE, "taille d'interface disproportionnée");
            assert!(size <= 20.0, "taille d'interface disproportionnée");
        }
    }

    #[test]
    fn graisses_partagent_la_meme_famille() {
        assert_eq!(REGULAR.family, MEDIUM.family);
        assert_eq!(MEDIUM.family, SEMIBOLD.family);
        assert_eq!(REGULAR.weight, Weight::Normal);
        assert_eq!(MEDIUM.weight, Weight::Medium);
        assert_eq!(SEMIBOLD.weight, Weight::Semibold);
    }

    #[test]
    fn monospace_partage_la_meme_famille_mono() {
        assert_eq!(MONO_REGULAR.family, MONO_SEMIBOLD.family);
        assert_ne!(MONO_REGULAR.family, REGULAR.family);
    }
}
