//! Échelle typographique desktop de Candilog.
//!
//! Volontairement contenue : aucun texte d'interface ne dépasse 20 px. La
//! hiérarchie s'obtient par la graisse, la couleur et l'espacement.

use iced::font::{Family, Stretch, Style, Weight};
use iced::Font;

/// Famille embarquée dans l'exécutable.
const FAMILY: Family = Family::Name("Cantarell");

const fn font(weight: Weight) -> Font {
    Font {
        family: FAMILY,
        weight,
        stretch: Stretch::Normal,
        style: Style::Normal,
    }
}

/// Graisse courante des textes et valeurs.
pub const REGULAR: Font = font(Weight::Normal);
/// Graisse des étiquettes, en-têtes de colonnes et éléments actifs.
pub const MEDIUM: Font = font(Weight::Medium);
/// Graisse des titres de section, de toolbar et des métriques.
pub const SEMIBOLD: Font = font(Weight::Semibold);

/// Barre d'état et unités.
pub const MICRO: f32 = 10.5;
/// Légendes et dates en liste.
pub const CAPTION: f32 = 11.5;
/// Métadonnées et texte secondaire.
pub const META: f32 = 12.0;
/// Étiquettes de formulaire et en-têtes de colonnes.
pub const LABEL: f32 = 12.0;
/// Corps de texte et valeurs.
pub const BODY: f32 = 13.5;
/// Titre de section.
pub const SECTION: f32 = 13.0;
/// Titre d'un objet de liste ou de carte.
pub const ITEM: f32 = 14.0;
/// Titre de toolbar et d'écran.
pub const TITLE: f32 = 19.0;
/// Valeur d'indicateur.
pub const METRIC: f32 = 28.0;
/// Valeur d'indicateur mise en avant, réservée à un usage rare.
pub const DISPLAY: f32 = 34.0;

const _: () = assert!(MICRO >= 10.0, "texte trop petit pour un écran desktop");
const _: () = assert!(TITLE <= 20.0, "titre d'interface disproportionné");

#[cfg(test)]
mod tests {
    use super::{
        BODY, CAPTION, DISPLAY, ITEM, LABEL, MEDIUM, META, METRIC, MICRO, REGULAR, SECTION,
        SEMIBOLD, TITLE,
    };
    use iced::font::Weight;

    #[test]
    fn echelle_est_strictement_croissante() {
        let scale = [MICRO, CAPTION, META, BODY, ITEM, TITLE, METRIC, DISPLAY];
        for pair in scale.windows(2) {
            assert!(pair[0] < pair[1], "échelle typographique non ordonnée");
        }
    }

    #[test]
    fn interface_ne_depasse_jamais_vingt_pixels() {
        for size in [MICRO, CAPTION, META, LABEL, BODY, SECTION, ITEM, TITLE] {
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
}
