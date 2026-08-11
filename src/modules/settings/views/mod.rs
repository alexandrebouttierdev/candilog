//! Écran Paramètres : fournisseurs, configuration, sauvegarde — page unique
//! dans l'esprit candilog-desktop.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::settings::components::{actions, setting_stacked};
use crate::modules::settings::model::ThemePref;
use crate::navigation::Route;
use crate::shared::llm::{AnalysisMode, ProviderKind};
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::provider_icon::provider_icon;
use crate::ui::components::{badge, field, layout, state, surface, typo};
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens, DAY, NIGHT};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{button, column, container, row, slider, stack, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

/// Largeur maximale du corps de la page (`max-w-[980px]`).
const BODY_MAX_WIDTH: f32 = 1120.0;
/// Largeur de la colonne latérale.
const SIDE_WIDTH: f32 = 320.0;
/// Largeur d'une carte fournisseur de la grille (`max-w-[200px]`).
const PROVIDER_CARD_WIDTH: f32 = 200.0;
/// Couleurs réelles des plans de travail présentés dans le choix de thème.
const SWATCH_LIGHT: Color = DAY.canvas;
const SWATCH_DARK: Color = NIGHT.canvas;

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

/// Rend l'écran des paramètres.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::route_header(
            Icon::Settings,
            "Intelligence artificielle",
            Route::Parametres,
            Message::Navigate,
            controls::primary("Enregistrer", Some(Icon::Save))
                .on_press(Message::SaveSettings)
                .into(),
        ),
        layout::workspace(
            column![
                surface::scroll(
                    container(
                        row![
                            container(main_column(app)).width(Length::FillPortion(1)),
                            container(side_column(app)).width(Length::Fixed(SIDE_WIDTH)),
                        ]
                        .spacing(space::LG)
                        .align_y(Alignment::Start)
                        .width(Length::Fill),
                    )
                    .max_width(BODY_MAX_WIDTH)
                    .center_x(Length::Fill),
                )
                .height(Length::Fill),
                container(footer_info())
                    .max_width(BODY_MAX_WIDTH)
                    .center_x(Length::Fill),
            ]
            .spacing(space::MD)
            .height(Length::Fill),
        ),
    )
}

/// Colonne principale dédiée à la configuration IA.
fn main_column(app: &App) -> Element<'_, Message> {
    column![provider_card(app)]
        .spacing(space::LG)
        .width(Length::Fill)
        .into()
}

/// Colonne latérale : apparence, sauvegardes et mises à jour.
fn side_column(app: &App) -> Element<'_, Message> {
    column![appearance_card(app), backup_card(), updates_card(app),]
        .spacing(space::LG)
        .width(Length::Fill)
        .into()
}

/// Informations essentielles, discrètes et toujours placées en bas de page.
fn footer_info<'a>() -> Element<'a, Message> {
    container(
        row![
            typo::caption(format!("Candilog v{}", env!("CARGO_PKG_VERSION"))),
            layout::spacer(),
            typo::caption("Créé par Alexandre Bouttier"),
            controls::ghost("alexandrebouttier.fr", Some(Icon::Link))
                .on_press(Message::OpenAuthorWebsite),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .height(48.0)
    .padding([0.0, space::LG])
    .width(Length::Fill)
    .style(styles::panel_flat)
    .into()
}

/// Carte d'une section des paramètres : en-tête (icône, titre) et contenu.
fn section_card<'a>(
    glyph: Icon,
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = content.into();
    container(
        column![
            container(
                row![
                    header_tile(glyph),
                    text(title).size(font::LABEL).font(font::SEMIBOLD),
                    layout::spacer(),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .height(size::SECTION_HEADER)
            .align_y(Alignment::Center),
            surface::divider(),
            content,
        ]
        .width(Length::Fill),
    )
    .padding([space::MD, space::LG])
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Pastille d'icône d'un en-tête de section (`bg-secondary/70 rounded-xl`).
fn header_tile<'a, Message: 'a>(glyph: Icon) -> Element<'a, Message> {
    container(icon::icon(glyph, icon::SM, Ink::Muted))
        .width(28.0)
        .height(28.0)
        .center(Length::Fixed(28.0))
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(alpha(palette.sunken, 0.70))),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Carte Fournisseur : grille de choix, bloc de configuration et actions.
fn provider_card(app: &App) -> Element<'_, Message> {
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

/// Carte Apparence : choix du thème et bascule rapide.
fn appearance_card(app: &App) -> Element<'_, Message> {
    let theme = &app.settings_form.draft.theme;
    section_card(
        Icon::Sun,
        "Apparence",
        column![
            row![
                theme_choice(ThemePref::System, "Système", *theme == ThemePref::System),
                theme_choice(ThemePref::Light, "Clair", *theme == ThemePref::Light),
                theme_choice(ThemePref::Dark, "Sombre", *theme == ThemePref::Dark),
            ]
            .spacing(space::SM),
            surface::divider(),
            controls::secondary(
                if app.is_dark {
                    "Passer en clair"
                } else {
                    "Passer en sombre"
                },
                Some(if app.is_dark { Icon::Sun } else { Icon::Moon }),
            )
            .on_press(Message::ToggleTheme)
            .width(Length::Fill),
        ]
        .spacing(space::LG)
        .width(Length::Fill),
    )
}

/// Bouton d'un choix de thème : aperçu coloré et libellé.
fn theme_choice<'a>(pref: ThemePref, label: &'a str, active: bool) -> Element<'a, Message> {
    button(
        column![theme_preview(&pref), typo::caption(label),]
            .spacing(space::SM)
            .align_x(Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .padding(space::SM)
    .style(choice_style(active, 1.0, 0.08))
    .on_press(Message::SettingsThemeChanged(pref))
    .into()
}

/// Aperçu coloré d'un thème (`h-10 rounded-md`), bicolore pour le système.
fn theme_preview<'a, Message: 'a>(pref: &ThemePref) -> Element<'a, Message> {
    match pref {
        ThemePref::System => row![
            swatch(SWATCH_LIGHT, true, false),
            swatch(SWATCH_DARK, false, true),
        ]
        .width(Length::Fill)
        .height(40.0)
        .into(),
        ThemePref::Light => swatch(SWATCH_LIGHT, true, true),
        ThemePref::Dark => swatch(SWATCH_DARK, true, true),
    }
}

/// Pastille de couleur d'un aperçu de thème.
fn swatch<'a, Message: 'a>(
    color: Color,
    rounded_left: bool,
    rounded_right: bool,
) -> Element<'a, Message> {
    container(Space::new(0.0, 0.0))
        .width(Length::FillPortion(1))
        .height(40.0)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: iced::border::Radius {
                    top_left: if rounded_left { radius::CONTROL } else { 0.0 },
                    bottom_left: if rounded_left { radius::CONTROL } else { 0.0 },
                    top_right: if rounded_right { radius::CONTROL } else { 0.0 },
                    bottom_right: if rounded_right { radius::CONTROL } else { 0.0 },
                },
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

/// Carte Sauvegarde : uniquement les actions utiles.
fn backup_card<'a>() -> Element<'a, Message> {
    section_card(
        Icon::Document,
        "Sauvegarde",
        column![
            controls::secondary("Exporter un backup", Some(Icon::Download))
                .on_press(Message::ExportBackup)
                .width(Length::Fill),
            controls::secondary("Importer un backup", Some(Icon::Import))
                .on_press(Message::SelectBackupImport)
                .width(Length::Fill),
            controls::secondary("Recharger", Some(Icon::Refresh))
                .on_press(Message::Reload)
                .width(Length::Fill),
            controls::danger("Réinitialiser la base", Some(Icon::Trash))
                .on_press(Message::OpenDialog(Dialog::ResetDatabase))
                .width(Length::Fill),
        ]
        .spacing(space::SM)
        .width(Length::Fill),
    )
}

/// Carte Mises à jour : vérification, téléchargement et progression.
fn updates_card(app: &App) -> Element<'_, Message> {
    let mut body = column![setting_stacked(
        "Vérification",
        "Recherche une version plus récente de Candilog.",
        controls::secondary("Rechercher une mise à jour", Some(Icon::Refresh))
            .on_press(Message::CheckUpdate)
            .width(Length::Fill),
    )]
    .width(Length::Fill);

    if let Some(update) = &app.available_update {
        body = body.push(actions(
            "Version disponible",
            "Téléchargement vérifié avant toute installation.",
            [
                badge::badge(update.version.to_string(), Tone::Success),
                controls::primary("Télécharger", Some(Icon::Download))
                    .on_press(Message::DownloadUpdate)
                    .into(),
            ],
        ));
    }
    if let Some(progress) = app.update_progress {
        body = body.push(setting_stacked(
            "Progression",
            "Paquet en cours de téléchargement.",
            state::progress_step("Téléchargement du paquet", f32::from(progress) / 100.0),
        ));
    }
    if let Some(path) = &app.verified_update_path {
        body = body.push(setting_stacked(
            "Paquet prêt",
            "Signature vérifiée, prêt à être installé.",
            typo::caption(path.display().to_string()),
        ));
    }
    if app.available_update.is_none() && app.update_progress.is_none() {
        body = body.push(
            container(typo::caption("Aucune mise à jour en attente.")).padding([space::MD, 0.0]),
        );
    }

    section_card(Icon::Download, "Mises à jour", body)
}

#[cfg(test)]
mod tests {
    use super::{provider_label, providers};
    use crate::shared::llm::ProviderKind;

    #[test]
    fn le_libelle_couvre_les_sept_fournisseurs() {
        assert_eq!(provider_label(&ProviderKind::Ollama), "Ollama");
        assert_eq!(provider_label(&ProviderKind::Claude), "Claude");
        assert_eq!(provider_label(&ProviderKind::OpenAI), "OpenAI");
        assert_eq!(provider_label(&ProviderKind::Gemini), "Gemini");
        assert_eq!(provider_label(&ProviderKind::Mistral), "Mistral");
        assert_eq!(provider_label(&ProviderKind::Nvidia), "NVIDIA");
        assert_eq!(
            provider_label(&ProviderKind::Custom("personnalisé".into())),
            "Personnalisé"
        );
    }

    #[test]
    fn la_grille_propose_sept_fournisseurs() {
        let grille = providers();
        assert_eq!(grille.len(), 7);
        assert_eq!(
            grille.iter().map(provider_label).collect::<Vec<_>>(),
            [
                "Ollama",
                "Claude",
                "OpenAI",
                "Gemini",
                "Mistral",
                "NVIDIA",
                "Personnalisé",
            ]
        );
    }
}
