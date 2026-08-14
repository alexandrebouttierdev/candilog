//! Transitions d'état du domaine `settings`.

use super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::SettingsProviderChanged(..)
            | Message::SettingsModelChanged(..)
            | Message::RefreshLlmModels
            | Message::LlmModelsLoaded(..)
            | Message::SettingsEndpointChanged(..)
            | Message::SettingsApiKeyChanged(..)
            | Message::SettingsTemperatureChanged(..)
            | Message::SettingsModeChanged(..)
            | Message::SettingsThemeChanged(..)
            | Message::SystemThemeDetected(..)
            | Message::SaveSettings
            | Message::SettingsSaved(..)
            | Message::TestLlmConnection
            | Message::LlmConnectionTested(..)
            | Message::OpenAuthorWebsite
            | Message::AuthorWebsiteOpened(..)
            | Message::ExportBackup
            | Message::BackupExported(..)
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::SettingsProviderChanged(provider) => {
            app.settings_form.draft.llm.provider = provider;
            app.available_models.clear();
            if matches!(
                app.settings_form.draft.llm.provider,
                crate::shared::llm::ProviderKind::Ollama
            ) && app.settings_form.draft.llm.endpoint.is_none()
            {
                app.settings_form.draft.llm.endpoint = Some("http://localhost:11434".into());
            }
            if app
                .settings_form
                .draft
                .llm
                .api_key
                .as_deref()
                .is_some_and(|key| !key.trim().is_empty())
                || matches!(
                    app.settings_form.draft.llm.provider,
                    crate::shared::llm::ProviderKind::Ollama
                )
            {
                return Task::done(Message::RefreshLlmModels);
            }
        }
        Message::SettingsModelChanged(value) => app.settings_form.draft.llm.model = value,
        Message::RefreshLlmModels => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let draft = app.settings_form.draft.llm.clone();
            app.provider_health = crate::ui::components::runtime_status::Health::Checking;
            return Task::perform(
                async move {
                    let mut config = draft;
                    if config.api_key.is_none() {
                        config.api_key = backend
                            .secure_settings_async()
                            .await
                            .map_err(|error| error.to_string())?
                            .llm
                            .api_key;
                    }
                    let pin = crate::shared::llm::validate_llm_endpoint(&config)
                        .await
                        .map_err(|error| error.to_string())?;
                    crate::modules::ia::factory::build_provider_pinned(&config, pin)
                        .list_models()
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::LlmModelsLoaded,
            );
        }
        Message::LlmModelsLoaded(result) => match result {
            Ok(mut models) => {
                models.sort();
                models.dedup();
                if !app.settings_form.draft.llm.model.trim().is_empty()
                    && !models.contains(&app.settings_form.draft.llm.model)
                {
                    models.insert(0, app.settings_form.draft.llm.model.clone());
                }
                app.available_models = models;
                app.provider_health = crate::ui::components::runtime_status::Health::Ok;
                app.notify_success("Liste des modèles actualisée.");
            }
            Err(error) => {
                app.provider_health = crate::ui::components::runtime_status::Health::Error;
                app.notify_failure(format!("Modèles indisponibles : {error}"));
            }
        },
        Message::SettingsEndpointChanged(value) => {
            app.settings_form.draft.llm.endpoint = optional(&value);
        }
        Message::SettingsApiKeyChanged(value) => {
            app.settings_form.draft.llm.api_key = optional(&value);
        }
        Message::SettingsTemperatureChanged(value) => {
            app.settings_form.draft.llm.temperature = value;
        }
        Message::SettingsModeChanged(value) => app.settings_form.draft.llm.mode = value,
        Message::SettingsThemeChanged(value) => {
            // Le thème est la seule exception assumée : l'aperçu doit être immédiat.
            app.is_dark = crate::core::theme_systeme::resoudre(value, app.system_dark, app.is_dark);
            app.settings_form.draft.theme = value;
            // Le système a pu changer d'avis depuis le démarrage : on redemande.
            if matches!(value, crate::modules::settings::model::ThemePref::System) {
                return Task::perform(
                    crate::core::theme_systeme::detecter(),
                    Message::SystemThemeDetected,
                );
            }
        }
        Message::SystemThemeDetected(sombre) => {
            app.system_dark = sombre;
            app.is_dark = crate::core::theme_systeme::resoudre(
                app.data.settings.theme,
                app.system_dark,
                app.is_dark,
            );
        }
        Message::SaveSettings => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let settings = app.settings_form.draft.clone();
            return Task::perform(
                async move {
                    backend
                        .update_secure_settings(settings)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::SettingsSaved,
            );
        }
        Message::SettingsSaved(result) => match result {
            Ok(settings) => {
                app.settings_form = crate::app::state::SettingsForm::from_settings(&settings);
                app.data.settings = settings;
                app.notify_success("Paramètres enregistrés.");
                return Task::done(Message::ProbeProviderHealth);
            }
            Err(error) => app.notify_failure(error),
        },
        Message::TestLlmConnection => {
            // Teste ce que l'utilisateur vient de saisir, pas ce qui est déjà enregistré.
            let config = app.settings_form.draft.llm.clone();
            app.provider_health = crate::ui::components::runtime_status::Health::Checking;
            return Task::perform(
                async move {
                    let pin = crate::shared::llm::validate_llm_endpoint(&config)
                        .await
                        .map_err(|error| error.to_string())?;
                    crate::modules::ia::factory::build_provider_pinned(&config, pin)
                        .health_check()
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::LlmConnectionTested,
            );
        }
        Message::LlmConnectionTested(result) => match result {
            Ok(()) => {
                app.provider_health = crate::ui::components::runtime_status::Health::Ok;
                app.notify_success("Connexion IA opérationnelle.");
            }
            Err(error) => {
                app.provider_health = crate::ui::components::runtime_status::Health::Error;
                app.notify_failure(format!("Connexion IA impossible : {error}"));
            }
        },
        Message::OpenAuthorWebsite => {
            return Task::perform(
                async {
                    tokio::task::spawn_blocking(|| {
                        crate::core::external::open_https("https://www.alexandrebouttier.fr")
                    })
                    .await
                    .unwrap_or_else(|error| Err(format!("Ouverture interrompue : {error}")))
                },
                Message::AuthorWebsiteOpened,
            );
        }
        Message::AuthorWebsiteOpened(result) => {
            if let Err(error) = result {
                app.notify_failure(error);
            }
        }
        Message::ExportBackup => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            return Task::perform(
                async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .set_title("Exporter la base Candilog")
                        .set_file_name("candilog-backup.sqlite")
                        .add_filter("SQLite", &["sqlite"])
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé.".into());
                    };
                    let path = file.path().to_path_buf();
                    crate::core::backup::export(&backend.sqlite, &path)
                        .map_err(|error| error.to_string())?;
                    Ok(path)
                },
                Message::BackupExported,
            );
        }
        Message::BackupExported(result) => match result {
            Ok(path) => app.notify_success(format!("Backup créé : {}", path.display())),
            Err(error) => app.notify_failure(error),
        },
        _ => unreachable!("message routé vers un domaine incorrect"),
    }
    Task::none()
}
