//! Écran Paramètres : sommaire à gauche, réglages à droite.

use crate::app::state::{Dialog, SettingsSection};
use crate::app::{App, Message};
use crate::modules::settings::components::{actions, setting, setting_stacked};
use crate::modules::settings::model::ThemePref;
use crate::shared::llm::{AnalysisMode, ProviderKind};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{badge, field, inspector, layout, state, surface, toolbar, typo};
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{button, column, container, row, slider};
use iced::{Alignment, Element, Length};

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

/// Rend l'écran des paramètres.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        toolbar::toolbar(
            "Paramètres",
            typo::meta("Configuration locale de Candilog"),
            badge::badge("Sur cet appareil", Tone::Success),
        ),
        layout::split_sized(size::SUMMARY, summary(app), section(app)),
    )
}

fn summary(app: &App) -> Element<'_, Message> {
    let mut items = column![].spacing(1).padding(space::MD);
    for section in SettingsSection::ALL {
        let active = app.settings_section == section;
        items = items.push(
            button(
                row![
                    icon::icon(
                        section_icon(section),
                        icon::MD,
                        if active { Ink::Accent } else { Ink::Muted },
                    ),
                    typo::body(section.label()),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(size::ROW + 2.0)
            .padding([0.0, space::MD])
            .style(styles::nav_item(active))
            .on_press(Message::SettingsSectionChanged(section)),
        );
    }
    container(surface::scroll(items).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::panel_flat)
        .into()
}

const fn section_icon(section: SettingsSection) -> Icon {
    match section {
        SettingsSection::Apparence => Icon::Sun,
        SettingsSection::Ia => Icon::Sparkles,
        SettingsSection::Donnees => Icon::Document,
        SettingsSection::MisesAJour => Icon::Download,
        SettingsSection::APropos => Icon::Info,
    }
}

fn section(app: &App) -> Element<'_, Message> {
    let body = match app.settings_section {
        SettingsSection::Apparence => appearance(app),
        SettingsSection::Ia => intelligence(app),
        SettingsSection::Donnees => data(app),
        SettingsSection::MisesAJour => updates(app),
        SettingsSection::APropos => about(app),
    };

    column![
        container(surface::section_header(
            app.settings_section.label(),
            iced::widget::Space::with_width(0),
        ))
        .height(size::TOOLBAR)
        .padding([0.0, space::XL])
        .align_y(Alignment::Center),
        surface::divider(),
        surface::scroll(container(body).padding([0.0, space::XL])).height(Length::Fill),
    ]
    .height(Length::Fill)
    .into()
}

fn appearance(app: &App) -> Element<'_, Message> {
    let themes = vec![ThemePref::System, ThemePref::Light, ThemePref::Dark];
    column![
        setting(
            "Thème de l'application",
            "Suit le système ou force une apparence pour Candilog.",
            field::select(
                themes,
                Some(app.data.settings.theme.clone()),
                Message::SettingsThemeChanged,
            )
            .width(Length::Fixed(200.0)),
        ),
        setting(
            "Apparence courante",
            "Bascule immédiate, sans modifier la préférence enregistrée.",
            controls::secondary(
                if app.is_dark {
                    "Passer en clair"
                } else {
                    "Passer en sombre"
                },
                Some(if app.is_dark { Icon::Sun } else { Icon::Moon }),
            )
            .on_press(Message::ToggleTheme),
        ),
    ]
    .into()
}

fn intelligence(app: &App) -> Element<'_, Message> {
    let llm = &app.data.settings.llm;
    let modes = vec![
        AnalysisMode::Auto,
        AnalysisMode::Small,
        AnalysisMode::Standard,
        AnalysisMode::Advanced,
    ];

    column![
        setting(
            "Fournisseur",
            "Service utilisé pour l'analyse d'offre, la génération et le scoring ATS.",
            field::select(
                providers(),
                Some(llm.provider.clone()),
                Message::SettingsProviderChanged,
            )
            .width(Length::Fixed(200.0)),
        ),
        setting(
            "Modèle",
            "Identifiant du modèle tel qu'attendu par le fournisseur.",
            field::input("Modèle", &llm.model)
                .on_input(Message::SettingsModelChanged)
                .width(Length::Fixed(200.0)),
        ),
        setting(
            "Endpoint",
            "URL du service. Laissez vide pour utiliser la valeur par défaut.",
            field::input("https://…", llm.endpoint.as_deref().unwrap_or_default())
                .on_input(Message::SettingsEndpointChanged)
                .width(Length::Fixed(280.0)),
        ),
        setting(
            "Clé API",
            "Stockée dans le coffre système, jamais en clair dans la base.",
            field::input("Clé API", llm.api_key.as_deref().unwrap_or_default())
                .secure(true)
                .on_input(Message::SettingsApiKeyChanged)
                .width(Length::Fixed(280.0)),
        ),
        setting(
            "Mode d'analyse",
            "Arbitrage entre rapidité et profondeur d'analyse.",
            field::select(modes, Some(llm.mode), Message::SettingsModeChanged)
                .width(Length::Fixed(200.0)),
        ),
        setting_stacked(
            "Température",
            "Plus la valeur est basse, plus les réponses sont déterministes.",
            row![
                slider(
                    0.0..=2.0,
                    llm.temperature,
                    Message::SettingsTemperatureChanged
                )
                .step(0.1_f32)
                .style(styles::range)
                .width(Length::Fixed(260.0)),
                typo::body(format!("{:.1}", llm.temperature)),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        ),
        actions(
            "Connexion",
            "Vérifie que le fournisseur répond avec les paramètres saisis.",
            [
                controls::ghost("Tester la connexion", Some(Icon::Refresh))
                    .on_press(Message::TestLlmConnection)
                    .into(),
                controls::ghost("Vider le cache IA", Some(Icon::Trash))
                    .on_press(Message::OpenDialog(Dialog::ResetAiCache))
                    .into(),
                controls::primary("Enregistrer", Some(Icon::Save))
                    .on_press(Message::SaveSettings)
                    .into(),
            ],
        ),
    ]
    .into()
}

fn data(app: &App) -> Element<'_, Message> {
    let database = app.paths.as_ref().map_or_else(
        || "Non résolue".to_owned(),
        |paths| paths.database.display().to_string(),
    );

    column![
        setting_stacked(
            "Base locale",
            "Toutes les données restent sur cet appareil.",
            typo::caption(database),
        ),
        actions(
            "Sauvegardes",
            "Export cohérent de la base SQLite et restauration validée.",
            [
                controls::ghost("Exporter un backup", Some(Icon::Download))
                    .on_press(Message::ExportBackup)
                    .into(),
                controls::ghost("Importer un backup", Some(Icon::Import))
                    .on_press(Message::SelectBackupImport)
                    .into(),
                controls::ghost("Recharger", Some(Icon::Refresh))
                    .on_press(Message::Reload)
                    .into(),
            ],
        ),
        actions(
            "Zone sensible",
            "La réinitialisation supprime définitivement toutes les données locales.",
            [
                controls::danger("Réinitialiser Candilog", Some(Icon::Trash))
                    .on_press(Message::OpenDialog(Dialog::ResetDatabase))
                    .into()
            ],
        ),
        inspector::property("Candidatures", app.data.candidatures.len().to_string()),
        inspector::property("Entreprises", app.data.entreprises.len().to_string()),
        inspector::property("Contacts", app.data.contacts.len().to_string()),
        inspector::property("Versions de CV", app.data.cv_versions.len().to_string()),
    ]
    .into()
}

fn updates(app: &App) -> Element<'_, Message> {
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
                    .into()
            ],
        ),
    ];

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
        body = body.push(state::progress_step(
            "Téléchargement du paquet",
            f32::from(progress) / 100.0,
        ));
    }
    if let Some(path) = &app.verified_update_path {
        body = body.push(state::hint("Paquet vérifié et prêt à être installé."));
        body = body.push(inspector::property(
            "Emplacement",
            path.display().to_string(),
        ));
    }
    if app.available_update.is_none() && app.update_progress.is_none() {
        body = body.push(state::empty_slot("Aucune mise à jour en attente."));
    }
    body.into()
}

fn about(app: &App) -> Element<'_, Message> {
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
                inspector::property("Appels IA", app.data.llm_calls.len().to_string()),
                inspector::property("Scores ATS", app.data.ats_scores.len().to_string()),
            ],
        ),
        state::hint("Candilog fonctionne entièrement hors ligne, hors appels IA explicites."),
    ]
    .spacing(space::XXL)
    .padding([space::LG, 0.0])
    .into()
}
