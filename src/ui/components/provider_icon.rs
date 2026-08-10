//! Logos officiels des fournisseurs IA, embarqués en SVG (repli neutre sinon).

use crate::ui::components::icon::{self, Icon};
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

/// Logo du fournisseur à la taille donnée, ou repli neutre.
pub fn provider_icon<'a, Message: 'a>(provider: &str, size: f32) -> Element<'a, Message> {
    match asset_for(provider) {
        Some(bytes) => Svg::new(Handle::from_memory(bytes))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into(),
        None => icon::icon(Icon::Settings, size * 0.7, icon::Ink::Muted).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::asset_for;

    #[test]
    fn chaque_fournisseur_connu_a_un_logo() {
        for provider in ["ollama", "openai", "claude", "gemini", "mistral", "nvidia"] {
            assert!(asset_for(provider).is_some(), "{provider} sans logo");
        }
        assert!(asset_for("custom").is_none());
        assert!(asset_for("OPENAI").is_some(), "casse ignorée");
    }
}
