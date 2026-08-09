//! Rôles typographiques prêts à l'emploi.
//!
//! Aucun écran n'écrit `text(…).size(13)` : il choisit un rôle, et le rôle
//! porte la taille, la graisse et la couleur.

use crate::ui::theme::color::Tone;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use iced::widget::text::IntoFragment;
use iced::widget::{text, Text};

/// Titre d'écran, porté par la toolbar.
pub fn title<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::TITLE).font(font::SEMIBOLD)
}

/// Titre d'une section à l'intérieur d'un panneau.
pub fn section<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::SECTION).font(font::SEMIBOLD)
}

/// Titre d'un objet de liste, de ligne ou de carte.
pub fn item<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::ITEM)
}

/// Titre d'objet mis en avant dans un inspecteur.
pub fn item_strong<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::ITEM).font(font::MEDIUM)
}

/// Corps de texte et valeurs.
pub fn body<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::BODY)
}

/// Métadonnée en retrait.
pub fn meta<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::META).style(styles::secondary_text)
}

/// Étiquette de formulaire ou en-tête de colonne.
pub fn label<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value)
        .size(font::LABEL)
        .font(font::MEDIUM)
        .style(styles::secondary_text)
}

/// Légende, date ou unité en retrait.
pub fn caption<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::CAPTION).style(styles::muted_text)
}

/// Texte le plus discret, réservé à la barre d'état.
pub fn micro<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::MICRO).style(styles::muted_text)
}

/// Valeur d'un indicateur.
pub fn metric<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::METRIC).font(font::SEMIBOLD)
}

/// Valeur d'indicateur mise en avant.
pub fn display<'a>(value: impl IntoFragment<'a>) -> Text<'a> {
    text(value).size(font::DISPLAY).font(font::SEMIBOLD)
}

/// Corps de texte teinté par un ton sémantique.
pub fn toned<'a>(value: impl IntoFragment<'a>, tone: Tone) -> Text<'a> {
    text(value).size(font::BODY).style(styles::toned_text(tone))
}

/// Métadonnée teintée par un ton sémantique.
pub fn meta_toned<'a>(value: impl IntoFragment<'a>, tone: Tone) -> Text<'a> {
    text(value).size(font::META).style(styles::toned_text(tone))
}

#[cfg(test)]
mod tests {
    use crate::ui::theme::typography as font;

    #[test]
    fn les_roles_couvrent_toute_l_echelle() {
        let scale = [
            font::MICRO,
            font::CAPTION,
            font::META,
            font::BODY,
            font::ITEM,
            font::TITLE,
            font::METRIC,
            font::DISPLAY,
        ];
        assert_eq!(scale.len(), 8);
        assert!(scale.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn aucun_role_d_interface_n_atteint_la_taille_d_une_metrique() {
        for size in [
            font::MICRO,
            font::CAPTION,
            font::META,
            font::LABEL,
            font::BODY,
            font::SECTION,
            font::ITEM,
            font::TITLE,
        ] {
            assert!(size < font::METRIC);
        }
    }
}
