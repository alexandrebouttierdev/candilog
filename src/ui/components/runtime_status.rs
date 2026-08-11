//! Pastille d'état du modèle IA, portée par la barre de titre.

use super::provider_icon;
use super::typo;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::tokens::tokens;
use iced::widget::{container, row, Space};
use iced::{Alignment, Background, Border, Color, Element, Theme};

/// Santé du fournisseur IA.
///
/// La pastille était alimentée par `if ai_is_running { Checking } else { Ok }` : elle passait
/// donc au vert dès qu'aucune opération n'était en cours, **sans qu'aucune vérification n'ait
/// jamais eu lieu** — vert et « llama3.2:3b » sur un premier lancement où aucun service Ollama
/// n'était joignable et aucune clé configurée. L'utilisateur découvrait l'indisponibilité au
/// moment de lancer une génération, après avoir attendu.
///
/// `Unknown` existe pour ne rien affirmer tant que rien n'a été mesuré.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Health {
    /// Aucune vérification effectuée à ce jour.
    #[default]
    Unknown,
    /// Répond normalement.
    Ok,
    /// En cours de vérification ou d'analyse.
    Checking,
    /// Injoignable.
    Error,
}

const fn color_unknown() -> Color {
    Color {
        r: 0.58,
        g: 0.60,
        b: 0.64,
        a: 1.0,
    }
}

const fn color_ok() -> Color {
    Color {
        r: 0.16,
        g: 0.78,
        b: 0.42,
        a: 1.0,
    }
}

const fn color_error() -> Color {
    Color {
        r: 0.92,
        g: 0.26,
        b: 0.21,
        a: 1.0,
    }
}

const fn color_checking() -> Color {
    Color {
        r: 0.96,
        g: 0.65,
        b: 0.14,
        a: 1.0,
    }
}

/// Couleur du point d'état pour la santé donnée.
#[must_use]
pub fn health_color(health: Health) -> Color {
    match health {
        Health::Unknown => color_unknown(),
        Health::Ok => color_ok(),
        Health::Checking => color_checking(),
        Health::Error => color_error(),
    }
}

/// Pastille d'état du fournisseur : logo, point, nom du modèle.
pub fn runtime_status<'a, Message: 'a>(
    provider: &'a str,
    model: &'a str,
    health: Health,
) -> Element<'a, Message> {
    let dot_color = health_color(health);
    let content = row![
        container(provider_icon::provider_icon(provider, 12.0))
            .width(22.0)
            .height(22.0)
            .center(iced::Length::Fixed(22.0))
            .style(move |theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(palette.panel)),
                    border: Border {
                        color: palette.border_strong,
                        width: 1.0,
                        radius: radius::PILL.into(),
                    },
                    ..container::Style::default()
                }
            }),
        container(Space::new(7.0, 7.0)).style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(dot_color)),
            border: Border {
                color: dot_color,
                width: 0.0,
                radius: radius::PILL.into(),
            },
            ..container::Style::default()
        }),
        typo::body(if model.is_empty() {
            "IA non configurée"
        } else {
            model
        }),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center);

    container(content)
        .padding([4.0, space::MD])
        .max_width(220.0)
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.panel)),
                border: Border {
                    color: Color {
                        a: 0.60,
                        ..palette.border
                    },
                    width: 1.0,
                    radius: radius::PILL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Version de l'application en pied de barre latérale.
pub fn app_version<'a, Message: 'a>() -> Element<'a, Message> {
    iced::widget::text(format!("Candilog v{}", env!("CARGO_PKG_VERSION")))
        .size(10.0)
        .font(crate::ui::theme::typography::MONO_REGULAR)
        .style(crate::ui::theme::styles::muted_text)
        .into()
}

#[cfg(test)]
mod tests {
    use super::{health_color, Health};

    #[test]
    fn la_sante_est_tri_etat() {
        assert_eq!(health_color(Health::Ok), super::color_ok());
        assert_ne!(health_color(Health::Ok), health_color(Health::Checking));
        assert_ne!(health_color(Health::Ok), health_color(Health::Error));
        assert_ne!(health_color(Health::Checking), health_color(Health::Error));
        // L'état « inconnu » doit se distinguer des trois autres : c'est lui qui évite
        // d'affirmer que le fournisseur répond alors que rien n'a été vérifié.
        for autre in [Health::Ok, Health::Checking, Health::Error] {
            assert_ne!(health_color(Health::Unknown), health_color(autre));
        }
        assert_eq!(Health::default(), Health::Unknown);
    }
}
