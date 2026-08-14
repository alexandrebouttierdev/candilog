//! Configuration du fournisseur et du modèle IA.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::shared::llm::{AnalysisMode, ProviderKind};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::provider_icon::provider_icon;
use crate::ui::components::{field, typo};
use crate::ui::theme::metrics::{radius, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use iced::widget::{button, column, container, row, slider, stack, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

/// Largeur d'une carte fournisseur de la grille (`max-w-[200px]`).
const PROVIDER_CARD_WIDTH: f32 = 200.0;

use super::section_card;

/// Fournisseurs IA proposés.
fn providers() -> Vec<ProviderKind> {
    vec![
        ProviderKind::Ollama,
        ProviderKind::Claude,
        ProviderKind::OpenAI,
        ProviderKind::Gemini,
        ProviderKind::Mistral,
        ProviderKind::Nvidia,
        ProviderKind::Custom("custom".into()),
    ]
}

/// Clé d'icône d'un fournisseur, pour le logo embarqué.
fn provider_key(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Ollama => "ollama",
        ProviderKind::Claude => "claude",
        ProviderKind::OpenAI => "openai",
        ProviderKind::Gemini => "gemini",
        ProviderKind::Mistral => "mistral",
        ProviderKind::Nvidia => "nvidia",
        ProviderKind::Custom(_) => "custom",
    }
}

/// Libellé français d'un fournisseur.
fn provider_label(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Ollama => "Ollama",
        ProviderKind::Claude => "Claude",
        ProviderKind::OpenAI => "OpenAI",
        ProviderKind::Gemini => "Gemini",
        ProviderKind::Mistral => "Mistral",
        ProviderKind::Nvidia => "NVIDIA",
        ProviderKind::Custom(_) => "Personnalisé",
    }
}

/// Modes d'analyse proposés, dans l'ordre du contrôle segmenté.
const MODES: [AnalysisMode; 4] = [
    AnalysisMode::Auto,
    AnalysisMode::Small,
    AnalysisMode::Standard,
    AnalysisMode::Advanced,
];

/// Carte Fournisseur : grille de choix, bloc de configuration et actions.
pub(super) fn provider_card(app: &App) -> Element<'_, Message> {
    let llm = &app.settings_form.draft.llm;
    let models = model_options(app);
    let selected_model = (!llm.model.trim().is_empty()).then(|| llm.model.clone());

    let mut grid = row![].spacing(space::SM).width(Length::Fill);
    for provider in providers() {
        grid = grid.push(
            container(provider_choice(&provider, llm.provider == provider))
                .width(Length::Fixed(PROVIDER_CARD_WIDTH)),
        );
    }

    section_card(
        Icon::Sparkles,
        "Fournisseur",
        column![
            grid.wrap(),
            container(
                column![
                    field::labeled(
                        "Modèle",
                        row![
                            field::select(models, selected_model, Message::SettingsModelChanged,)
                                .width(Length::Fill),
                            controls::secondary("Actualiser", Some(Icon::Refresh))
                                .on_press(Message::RefreshLlmModels),
                        ]
                        .spacing(space::SM)
                        .align_y(Alignment::Center),
                    ),
                    field::text_field(
                        "Endpoint",
                        llm.endpoint.as_deref().unwrap_or_default(),
                        Message::SettingsEndpointChanged,
                    ),
                    field::secret_field(
                        "Clé API",
                        llm.api_key.as_deref().unwrap_or_default(),
                        "Stockée dans le coffre système, jamais en clair dans la base.",
                        Message::SettingsApiKeyChanged,
                    ),
                    field::form_row([
                        field::labeled(
                            "Mode d'analyse",
                            field::select(
                                MODES.to_vec(),
                                Some(llm.mode),
                                Message::SettingsModeChanged,
                            )
                            .width(Length::Fill),
                        ),
                        field::labeled(
                            "Température",
                            row![
                                slider(
                                    0.0..=2.0,
                                    llm.temperature,
                                    Message::SettingsTemperatureChanged,
                                )
                                .step(0.1_f32)
                                .style(styles::range)
                                .width(Length::Fill),
                                temperature_value(llm.temperature),
                            ]
                            .spacing(space::MD)
                            .align_y(Alignment::Center),
                        ),
                    ]),
                ]
                .spacing(space::LG),
            )
            .padding(space::LG)
            .width(Length::Fill)
            .style(styles::sunken),
            row![
                controls::ghost("Tester la connexion", Some(Icon::Refresh))
                    .on_press(Message::TestLlmConnection),
                controls::ghost("Vider le cache IA", Some(Icon::Trash))
                    .on_press(Message::OpenDialog(Dialog::ResetAiCache)),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(space::LG)
        .padding([space::LG, 0.0])
        .width(Length::Fill),
    )
}

/// Modèles disponibles dans le select : le modèle courant reste toujours visible,
/// puis viennent ceux réellement annoncés par le fournisseur.
fn model_options(app: &App) -> Vec<String> {
    let current = app.settings_form.draft.llm.model.trim();
    let mut models = app.available_models.clone();
    if !current.is_empty() && !models.iter().any(|model| model == current) {
        models.insert(0, current.to_owned());
    }
    models
}

/// Carte d'un fournisseur : logo, nom, état actif.
fn provider_choice(provider: &ProviderKind, active: bool) -> Element<'static, Message> {
    let card = button(
        column![
            logo_tile(provider_key(provider)),
            text(provider_label(provider))
                .size(font::CAPTION)
                .font(font::SEMIBOLD),
        ]
        .spacing(space::SM)
        .align_x(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([space::MD, space::SM])
    .style(choice_style(active, 0.60, 0.10))
    .on_press(Message::SettingsProviderChanged(provider.clone()));

    if active {
        stack![card, check_chip()].into()
    } else {
        card.into()
    }
}

/// Coche de sélection : carré 16 px primary, dans le coin supérieur droit.
fn check_chip<'a, Message: 'a>() -> Element<'a, Message> {
    container(
        container(icon::icon(Icon::Check, icon::SM, Ink::OnAccent))
            .width(16.0)
            .height(16.0)
            .center(Length::Fixed(16.0))
            .style(|theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(palette.accent_fill)),
                    border: Border {
                        radius: radius::PILL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(space::SM)
    .align_x(Alignment::End)
    .align_y(Alignment::Start)
    .into()
}

/// Carré de 40 px portant le logo du fournisseur (`bg-white border`, panel en sombre).
fn logo_tile<'a, Message: 'a>(provider: &str) -> Element<'a, Message> {
    container(provider_icon(provider, 32.0))
        .width(40.0)
        .height(40.0)
        .center(Length::Fixed(40.0))
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(if palette.is_dark {
                    palette.panel
                } else {
                    Color::WHITE
                })),
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: radius::CONTROL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Style d'une carte sélectionnable : filet et fond d'accent si active.
fn choice_style(
    active: bool,
    accent_border: f32,
    accent_background: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = tokens(theme);
        let engaged = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let background = if active {
            Some(Background::Color(alpha(palette.accent, accent_background)))
        } else if engaged {
            Some(Background::Color(palette.hover))
        } else {
            None
        };
        button::Style {
            background,
            text_color: if active {
                palette.text
            } else {
                palette.text_secondary
            },
            border: Border {
                color: if active {
                    alpha(palette.accent, accent_border)
                } else {
                    palette.border
                },
                width: stroke::HAIRLINE,
                radius: radius::CONTROL.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

/// Valeur de température en monospace (`rounded-md bg-secondary`).
fn temperature_value<'a, Message: 'a>(value: f32) -> Element<'a, Message> {
    container(typo::text_mono(
        format!("{value:.1}"),
        font::MICRO,
        font::MONO_SEMIBOLD,
    ))
    .padding([space::XXS, space::SM])
    .style(|theme: &Theme| {
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
    .into()
}

#[cfg(test)]
#[path = "provider/tests/mod.rs"]
mod tests;
