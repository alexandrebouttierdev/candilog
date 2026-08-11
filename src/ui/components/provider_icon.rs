//! Logos officiels des fournisseurs IA, embarqués en SVG (repli neutre sinon).

use crate::ui::components::icon::{self, Icon};
use crate::ui::theme::tokens::tokens;
use iced::widget::svg::{Handle, Svg};
use iced::{Element, Length};

/// Chemin du logo embarqué d'un fournisseur, casse insensible.
#[must_use]
pub fn asset_for(provider: &str) -> Option<&'static [u8]> {
    match provider.to_lowercase().as_str() {
        "ollama" => Some(include_bytes!("../../../assets/providers/ollama.svg")),
        "openai" => Some(include_bytes!("../../../assets/providers/openai.svg")),
        "claude" => Some(include_bytes!("../../../assets/providers/claude.svg")),
        "gemini" => Some(include_bytes!("../../../assets/providers/gemini.svg")),
        "mistral" => Some(include_bytes!("../../../assets/providers/mistral.svg")),
        "nvidia" => Some(include_bytes!("../../../assets/providers/nvidia.svg")),
        _ => None,
    }
}

/// Logo rendu en monochrome (`fill="currentColor"`), à teinter du texte du thème
/// pour rester lisible dans les deux modes. Les autres logos portent leur couleur
/// de marque et ne doivent pas être teintés.
#[must_use]
fn monochrome(provider: &str) -> bool {
    matches!(provider.to_lowercase().as_str(), "ollama" | "openai")
}

/// Logo du fournisseur à la taille donnée, ou repli neutre.
pub fn provider_icon<'a, Message: 'a>(provider: &str, size: f32) -> Element<'a, Message> {
    match asset_for(provider) {
        Some(bytes) => {
            let mut svg = Svg::new(Handle::from_memory(bytes))
                .width(Length::Fixed(size))
                .height(Length::Fixed(size));
            if monochrome(provider) {
                svg = svg.style(move |theme, _status| iced::widget::svg::Style {
                    color: Some(tokens(theme).text_secondary),
                });
            }
            svg.into()
        }
        None => icon::icon(Icon::Settings, size * 0.7, icon::Ink::Muted).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_for, monochrome};

    #[test]
    fn chaque_fournisseur_connu_a_un_logo() {
        for provider in ["ollama", "openai", "claude", "gemini", "mistral", "nvidia"] {
            assert!(asset_for(provider).is_some(), "{provider} sans logo");
        }
        assert!(asset_for("custom").is_none());
        assert!(asset_for("OPENAI").is_some(), "casse ignorée");
    }

    #[test]
    fn les_logos_monochromes_sont_reconnus() {
        assert!(monochrome("ollama"));
        assert!(monochrome("openai"));
        assert!(monochrome("OPENAI"), "casse ignorée");
    }

    #[test]
    fn les_logos_en_couleur_ne_sont_pas_tentes() {
        for provider in ["claude", "gemini", "mistral", "nvidia", "custom"] {
            assert!(
                !monochrome(provider),
                "{provider} ne doit pas être monochrome"
            );
        }
    }
}
