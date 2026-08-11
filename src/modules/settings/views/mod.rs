//! Écran Paramètres : fournisseurs, configuration, sauvegarde — page unique
//! dans l'esprit candilog-desktop.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::settings::components::{actions, setting, setting_stacked};
use crate::modules::settings::model::ThemePref;
use crate::shared::llm::{AnalysisMode, ProviderKind};
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::provider_icon::provider_icon;
use crate::ui::components::tabs::Tab;
use crate::ui::components::{badge, field, inspector, layout, state, surface, tabs, typo};
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{button, column, container, row, slider, stack, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

/// Largeur maximale du corps de la page (`max-w-[980px]`).
const BODY_MAX_WIDTH: f32 = 980.0;
/// Largeur de la colonne latérale (`w-[280px]`).
const SIDE_WIDTH: f32 = 280.0;
/// Largeur d'une carte fournisseur de la grille (`max-w-[200px]`).
const PROVIDER_CARD_WIDTH: f32 = 200.0;
/// Couleur de l'aperçu clair d'un choix de thème (`#f4f4f7`).
const SWATCH_LIGHT: Color = Color::from_rgb(0.957, 0.957, 0.969);
/// Couleur de l'aperçu sombre d'un choix de thème (`#202026`).
const SWATCH_DARK: Color = Color::from_rgb(0.125, 0.125, 0.149);

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
        header::page_header(
            Icon::Settings,
            "Paramètres",
            "Configuration locale de Candilog",
            local_data_pill(),
        ),
        layout::workspace(surface::scroll(
            container(
                container(
                    row![
                        container(main_column(app)).width(Length::FillPortion(1)),
                        container(side_column(app)).width(Length::Fixed(SIDE_WIDTH)),
                    ]
                    .spacing(space::LG)
                    .align_y(Alignment::Start)
                    .width(Length::Fill),
                )
                .max_width(BODY_MAX_WIDTH),
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),
        )),
    )
}

/// Pill « Données locales » : point émeraude de 8 px et libellé.
fn local_data_pill<'a, Message: 'a>() -> Element<'a, Message> {
    container(
        row![
            container(Space::new(8.0, 8.0)).style(|theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(palette.success)),
                    border: Border {
                        radius: radius::PILL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            }),
            typo::caption("Données locales"),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .padding([space::XS, space::MD])
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(alpha(palette.panel, 0.45))),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: radius::PILL.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

/// Colonne principale : fournisseurs, mises à jour et à propos.
fn main_column(app: &App) -> Element<'_, Message> {
    column![provider_card(app), updates_card(app), about_card(app),]
        .spacing(space::LG)
        .width(Length::Fill)
        .into()
}

/// Colonne latérale : apparence, sauvegarde, connexion, enregistrement.
fn side_column(app: &App) -> Element<'_, Message> {
    column![
        appearance_card(app),
        backup_card(app),
        connection_card(app),
        controls::primary("Enregistrer", Some(Icon::Save))
            .on_press(Message::SaveSettings)
            .width(Length::Fill)
            .height(40.0),
    ]
    .spacing(space::LG)
    .width(Length::Fill)
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
                    field::text_field("Modèle", &llm.model, Message::SettingsModelChanged),
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
                    column![
                        typo::label("Mode d'analyse"),
                        tabs::segmented(
                            MODES.map(|mode| Tab::new(mode.to_string(), llm.mode == mode)),
                            |index| Message::SettingsModeChanged(MODES[index]),
                        ),
                    ]
                    .spacing(space::XS),
                    column![
                        typo::label("Température"),
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
                    ]
                    .spacing(space::XS),
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
                layout::spacer(),
                controls::primary("Enregistrer", Some(Icon::Save)).on_press(Message::SaveSettings),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(space::LG)
        .padding([space::LG, 0.0])
        .width(Length::Fill),
    )
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

/// Carte Sauvegarde : exports, réinitialisation et compteurs locaux.
fn backup_card(app: &App) -> Element<'_, Message> {
    let database = app.paths.as_ref().map_or_else(
        || "Non résolue".to_owned(),
        |paths| paths.database.display().to_string(),
    );

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
            surface::divider(),
            typo::label("Base locale"),
            typo::caption(database),
            surface::divider(),
            typo::label("Données suivies"),
            typo::caption(format!("{} candidatures", app.data.candidatures.len())),
            typo::caption(format!("{} entreprises", app.data.entreprises.len())),
            typo::caption(format!("{} contacts", app.data.contacts.len())),
            typo::caption(format!("{} versions de CV", app.data.cv_versions.len())),
        ]
        .spacing(space::SM)
        .width(Length::Fill),
    )
}

/// Carte Connexion : fournisseur courant et test de connexion.
fn connection_card(app: &App) -> Element<'_, Message> {
    let llm = &app.settings_form.draft.llm;
    let model = if llm.model.trim().is_empty() {
        "Modèle non défini"
    } else {
        llm.model.trim()
    };

    section_card(
        Icon::Link,
        "Connexion",
        column![
            row![
                logo_tile(provider_key(&llm.provider)),
                column![
                    typo::body(provider_label(&llm.provider)),
                    typo::caption(model),
                ]
                .spacing(1)
                .align_x(Alignment::Start),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            controls::secondary("Tester la connexion", Some(Icon::Refresh))
                .on_press(Message::TestLlmConnection)
                .width(Length::Fill),
        ]
        .spacing(space::LG)
        .width(Length::Fill),
    )
}

/// Carte Mises à jour : canal, vérification, téléchargement et progression.
fn updates_card(app: &App) -> Element<'_, Message> {
    let mut body = column![
        setting(
            "Canal",
            "Canal public GitLab, manifeste signé avec minisign.",
            badge::badge("Stable", Tone::Accent),
        ),
        actions(
            "Vérification",
            "Recherche une version plus récente et vérifie sa signature.",
            [
                controls::secondary("Rechercher une mise à jour", Some(Icon::Refresh))
                    .on_press(Message::CheckUpdate)
                    .into(),
            ],
        ),
    ]
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

/// Carte À propos : version, moteur, stockage et contenu suivi.
fn about_card(app: &App) -> Element<'_, Message> {
    section_card(
        Icon::Info,
        "À propos",
        column![
            inspector::group(
                "Application",
                [
                    inspector::property("Version", env!("CARGO_PKG_VERSION")),
                    inspector::property("Moteur", "Rust + Iced"),
                    inspector::property("Stockage", "SQLite local"),
                ],
            ),
            inspector::group(
                "Contenu",
                [
                    inspector::property(
                        "Candidatures suivies",
                        app.data.candidatures.len().to_string(),
                    ),
                    inspector::property("Appels IA", app.data.llm_calls.total.to_string()),
                    inspector::property("Scores ATS", app.data.ats_scores.total.to_string()),
                ],
            ),
            state::hint("Candilog fonctionne entièrement hors ligne, hors appels IA explicites."),
        ]
        .spacing(space::XXL)
        .padding([space::LG, 0.0])
        .width(Length::Fill),
    )
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
