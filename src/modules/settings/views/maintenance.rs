//! Écrans de maintenance : sauvegardes, mises à jour et à propos.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{badge, layout, state, surface, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

use super::{action_card, section_card, settings_hero, BODY_MAX_WIDTH};

pub fn backup_view(_app: &App) -> Element<'_, Message> {
    let content = column![
        settings_hero(
            Icon::Save,
            "VOS DONNÉES",
            "Une copie sûre, quand vous le décidez.",
            "Exportez ou restaurez toute votre base Candilog depuis un fichier local.",
        ),
        row![
            action_card(
                Icon::Download,
                "Créer une sauvegarde",
                "Générez une archive complète et conservez-la où vous le souhaitez.",
                controls::primary("Exporter", Some(Icon::Download))
                    .on_press(Message::ExportBackup)
                    .width(Length::Fill)
                    .into(),
            ),
            action_card(
                Icon::Import,
                "Restaurer une sauvegarde",
                "Choisissez un fichier Candilog existant avant de confirmer la restauration.",
                controls::secondary("Choisir un fichier", Some(Icon::Import))
                    .on_press(Message::SelectBackupImport)
                    .width(Length::Fill)
                    .into(),
            ),
        ]
        .spacing(space::LG),
        section_card(
            Icon::Settings,
            "Maintenance locale",
            row![
                column![
                    typo::body("Rafraîchir les données"),
                    typo::caption("Relit la base sans modifier son contenu."),
                ]
                .spacing(space::XS),
                layout::spacer(),
                controls::secondary("Recharger", Some(Icon::Refresh)).on_press(Message::Reload),
                controls::danger("Réinitialiser", Some(Icon::Trash))
                    .on_press(Message::OpenDialog(Dialog::ResetDatabase)),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center)
            .padding([space::LG, 0.0]),
        ),
    ]
    .spacing(space::LG)
    .width(Length::Fill);
    layout::screen(
        header::route_header(
            Icon::Save,
            "Sauvegardes",
            Route::Sauvegardes,
            Message::Navigate,
            iced::widget::Space::with_width(0).into(),
        ),
        layout::workspace(surface::scroll(
            container(content)
                .width(Length::Fill)
                .max_width(BODY_MAX_WIDTH)
                .center_x(Length::Fill),
        )),
    )
}

/// Écran dédié au cycle de mise à jour.
pub fn updates_view(app: &App) -> Element<'_, Message> {
    let status: Element<'_, Message> = if let Some(update) = &app.available_update {
        column![
            badge::badge(
                format!("Version {} disponible", update.version),
                Tone::Success
            ),
            controls::primary("Télécharger la mise à jour", Some(Icon::Download))
                .on_press(Message::DownloadUpdate),
        ]
        .spacing(space::MD)
        .into()
    } else {
        column![
            badge::badge("Aucune mise à jour en attente", Tone::Neutral),
            controls::primary("Rechercher maintenant", Some(Icon::Refresh))
                .on_press(Message::CheckUpdate),
        ]
        .spacing(space::MD)
        .into()
    };
    let mut content = column![
        settings_hero(
            Icon::Download,
            "VERSION ACTUELLE",
            env!("CARGO_PKG_VERSION"),
            "Candilog vérifie les nouvelles versions uniquement lorsque vous le demandez.",
        ),
        row![
            action_card(
                Icon::Refresh,
                "Disponibilité",
                "Interrogez la source officielle et comparez-la à votre version installée.",
                status,
            ),
            action_card(
                Icon::Check,
                "Installation maîtrisée",
                "L'installeur adapté à votre système est téléchargé puis lancé avec le \
                 programme d'installation par défaut.",
                typo::caption("Aucune installation silencieuse").into(),
            ),
        ]
        .spacing(space::LG),
    ]
    .spacing(space::LG)
    .width(Length::Fill);
    if let Some(progress) = app.update_progress {
        content = content.push(section_card(
            Icon::Download,
            "Téléchargement",
            container(state::progress_step(
                "Téléchargement et vérification du paquet",
                f32::from(progress) / 100.0,
            ))
            .padding([space::LG, 0.0]),
        ));
    }
    layout::screen(
        header::route_header(
            Icon::Download,
            "Mises à jour",
            Route::MisesAJour,
            Message::Navigate,
            typo::caption(format!("Version actuelle {}", env!("CARGO_PKG_VERSION"))).into(),
        ),
        layout::workspace(surface::scroll(
            container(content)
                .width(Length::Fill)
                .max_width(BODY_MAX_WIDTH)
                .center_x(Length::Fill),
        )),
    )
}

/// Écran À propos, accessible depuis les deux zones de marque du rail.
pub fn about_view(_app: &App) -> Element<'_, Message> {
    let hero = container(
        row![
            container(icon::brand(96.0))
                .width(132.0)
                .height(132.0)
                .center(Length::Fixed(132.0))
                .style(styles::form_group),
            column![
                typo::meta_toned("CANDILOG DESKTOP", Tone::Accent),
                typo::title("Votre recherche d'emploi, enfin au même endroit."),
                typo::body(
                    "Un cockpit natif pour suivre vos candidatures, développer votre réseau et produire des documents professionnels cohérents.",
                ),
                row![
                    badge::badge(
                        format!("Version {}", env!("CARGO_PKG_VERSION")),
                        Tone::Neutral,
                    ),
                    badge::badge("Rust · Iced · SQLite", Tone::Accent),
                ]
                .spacing(space::SM),
            ]
            .spacing(space::SM)
            .width(Length::Fill),
        ]
        .spacing(space::MAX)
        .align_y(Alignment::Center),
    )
    .padding(36.0)
    .width(Length::Fill)
    .style(styles::glass_card);

    let values = row![
        action_card(
            Icon::Save,
            "Vos données restent locales",
            "Candidatures, contacts et documents sont conservés dans votre base SQLite.",
            badge::badge("Local-first", Tone::Success),
        ),
        action_card(
            Icon::Panel,
            "Une expérience vraiment native",
            "Interface desktop, raccourcis clavier et intégration au système, sans navigateur embarqué.",
            badge::badge("100 % natif", Tone::Accent),
        ),
        action_card(
            Icon::Sparkles,
            "Une IA sous votre contrôle",
            "Vous choisissez le fournisseur, le modèle et les contenus à analyser.",
            badge::badge("Configurable", Tone::Neutral),
        ),
    ]
    .spacing(space::LG);

    let author = section_card(
        Icon::Profile,
        "Un produit indépendant",
        row![
            column![
                typo::meta_toned("CONÇU ET DÉVELOPPÉ PAR", Tone::Accent),
                typo::section("Alexandre Bouttier"),
                typo::caption("Pensé pour une recherche d'emploi exigeante, concrète et locale."),
            ]
            .spacing(space::XS),
            layout::spacer(),
            controls::secondary("Visiter le site", Some(Icon::Link))
                .on_press(Message::OpenAuthorWebsite),
            controls::primary("Vérifier les mises à jour", Some(Icon::Download))
                .on_press(Message::Navigate(Route::MisesAJour)),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center)
        .padding([space::LG, 0.0]),
    );

    let content = column![hero, values, author]
        .spacing(space::LG)
        .width(Length::Fill);
    layout::screen(
        header::route_header(
            Icon::Info,
            "À propos",
            Route::APropos,
            Message::Navigate,
            iced::widget::Space::with_width(0).into(),
        ),
        layout::workspace(surface::scroll(
            container(content)
                .width(Length::Fill)
                .max_width(1120.0)
                .center_x(Length::Fill),
        )),
    )
}
