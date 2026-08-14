//! Import d'un CV PDF : extraction et validation explicite avant ajout au profil.

use super::sections::header_tile;
use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{badge, layout, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

mod groups;

use groups::{import_groups, ImportGroup};

/// Carte Import : choix du PDF, extraction et validation explicite.
pub(super) fn import_section(app: &App) -> Element<'_, Message> {
    let file = app.profile_import_path.as_ref().map_or_else(
        || "Aucun CV sélectionné".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("cv.pdf")
                .to_owned()
        },
    );

    let source = container(
        row![
            header_tile(Icon::Document),
            column![
                typo::label("Document source"),
                typo::item(format::truncate(&file, 42)),
            ]
            .spacing(space::XS),
            layout::spacer(),
            controls::ghost("Parcourir", Some(Icon::Import)).on_press(Message::SelectProfilePdf),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::MD)
    .width(Length::FillPortion(2))
    .style(styles::sunken);

    let action: Element<'_, Message> = if app.ai_is_running {
        container(state::running(
            "Extraction du profil",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        ))
        .width(Length::FillPortion(2))
        .into()
    } else if app.extracted_profile.is_some() {
        column![
            badge::badge("Analyse terminée", Tone::Success),
            controls::primary("Vérifier les données", Some(Icon::ArrowRight))
                .on_press(Message::OpenDialog(Dialog::ProfileImport)),
        ]
        .spacing(space::SM)
        .align_x(Alignment::End)
        .into()
    } else {
        let mut analyze = controls::secondary("Analyser le CV", Some(Icon::Sparkles));
        if app.profile_import_path.is_some() {
            analyze = analyze.on_press(Message::ExtractProfile);
        }
        analyze.into()
    };

    container(
        row![
            row![
                header_tile(Icon::Import),
                column![
                    typo::section("Importer depuis un CV"),
                    typo::caption("L’IA prépare les données ; vous gardez le dernier mot."),
                ]
                .spacing(space::XS),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center)
            .width(Length::FillPortion(2)),
            source,
            action,
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Grand volet de validation, calqué sur le parcours de candilog-desktop.
pub fn import_review_drawer(app: &App) -> Element<'_, Message> {
    let Some(profile) = &app.extracted_profile else {
        return column![
            drawer_header(),
            container(state::empty_slot("Aucune donnée extraite à vérifier.")).center(Length::Fill),
        ]
        .height(Length::Fill)
        .into();
    };
    let groups = import_groups(profile);
    let total = groups.iter().map(|group| group.items.len()).sum::<usize>();
    let accepted = groups
        .iter()
        .flat_map(|group| &group.items)
        .filter(|item| !app.profile_import_excluded.contains(&item.key))
        .count();
    let breakdown = import_breakdown(&groups, &app.profile_import_excluded);
    let file = app.profile_import_path.as_ref().map_or_else(
        || "CV analysé".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("cv.pdf")
                .to_owned()
        },
    );

    let summary = container(
        column![
            container(icon::icon(Icon::Check, icon::LG, Ink::Toned(Tone::Success)))
                .center(Length::Fixed(42.0))
                .style(|theme: &Theme| import_toned_tile(theme, Tone::Success)),
            typo::section("CV analysé"),
            typo::caption(format::truncate(&file, 28)),
            surface::divider(),
            row![
                import_metric(accepted, "sélectionnées", Tone::Accent),
                import_metric(total.saturating_sub(accepted), "ignorées", Tone::Neutral),
            ]
            .spacing(space::SM),
            surface::divider(),
            typo::label("Sélection rapide"),
            controls::wide("Tout sélectionner", Some(Icon::Check))
                .on_press(Message::AcceptAllProfileImportItems),
            controls::wide("Tout ignorer", Some(Icon::Close))
                .on_press(Message::RejectAllProfileImportItems),
            surface::divider(),
            typo::label("Répartition"),
            breakdown,
            Space::with_height(Length::Fill),
            state::hint("Votre profil actuel reste prioritaire et les doublons sont écartés automatiquement."),
        ]
        .spacing(space::MD),
    )
    .width(Length::Fixed(220.0))
    .height(Length::Fill)
    .padding(space::XL)
    .style(styles::sunken);

    let mut suggestions = column![row![
        column![
            typo::section("Informations détectées"),
            typo::caption("Choisissez précisément ce qui rejoindra votre profil."),
        ]
        .spacing(space::XS),
        layout::spacer(),
        badge::count(total),
    ]
    .align_y(Alignment::Center),]
    .spacing(space::MD)
    .padding([0.0, space::XL]);
    for group in groups {
        suggestions = suggestions.push(import_group_card(group, &app.profile_import_excluded));
    }

    let mut apply = controls::primary("Ajouter au profil", Some(Icon::Check));
    if accepted > 0 {
        apply = apply.on_press(Message::ApplyExtractedProfile);
    }
    column![
        drawer_header(),
        row![
            summary,
            container(surface::scroll(suggestions).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(space::XL)
        .height(Length::Fill),
        surface::divider(),
        row![
            controls::ghost("Changer de CV", Some(Icon::Refresh))
                .on_press(Message::SelectProfilePdf),
            layout::spacer(),
            badge::badge(format!("{accepted} à ajouter"), Tone::Accent),
            apply,
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    ]
    .padding(space::XL)
    .spacing(space::XL)
    .height(Length::Fill)
    .into()
}

fn import_breakdown(
    groups: &[ImportGroup],
    excluded: &std::collections::HashSet<String>,
) -> Element<'static, Message> {
    let mut content = column![].spacing(space::SM);
    for group in groups {
        let selected = group
            .items
            .iter()
            .filter(|item| !excluded.contains(&item.key))
            .count();
        content = content.push(
            row![
                icon::icon(group.kind, icon::SM, Ink::Muted),
                typo::caption(group.title),
                layout::spacer(),
                typo::text_mono(
                    format!("{selected}/{}", group.items.len()),
                    font::MICRO,
                    font::MONO_SEMIBOLD,
                ),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        );
    }
    content.into()
}

fn drawer_header() -> Element<'static, Message> {
    row![
        header_tile(Icon::Sparkles),
        column![
            typo::text_uppercase("IMPORT INTELLIGENT", font::MICRO, font::SEMIBOLD),
            typo::title("Vérifier les données du CV"),
            typo::caption("Rien n’est enregistré avant votre validation finale."),
        ]
        .spacing(space::XS),
        layout::spacer(),
        controls::icon_action(Icon::Close, "Fermer", Message::CloseDialog),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .into()
}

fn import_metric(value: usize, label: &'static str, tone: Tone) -> Element<'static, Message> {
    container(
        column![typo::metric(value.to_string()), typo::caption(label)]
            .spacing(space::XS)
            .align_x(Alignment::Start),
    )
    .padding(space::SM)
    .width(Length::Fill)
    .style(move |theme: &Theme| import_metric_style(theme, tone))
    .into()
}

fn import_group_card(
    group: ImportGroup,
    excluded: &std::collections::HashSet<String>,
) -> Element<'static, Message> {
    let count = group.items.len();
    let mut content = column![
        row![
            container(icon::icon(group.kind, icon::SM, Ink::Accent))
                .center(Length::Fixed(28.0))
                .style(|theme: &Theme| import_toned_tile(theme, Tone::Accent)),
            typo::section(group.title),
            layout::spacer(),
            badge::count(count),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
        surface::divider(),
    ]
    .spacing(space::SM);
    for item in group.items {
        let is_included = !excluded.contains(&item.key);
        let mut description = column![].spacing(space::XS);
        if !item.label.is_empty() {
            description = description.push(typo::label(item.label));
        }
        description = description.push(typo::item(format::truncate(&item.value, 72)));
        if let Some(meta) = item.meta {
            description = description.push(typo::caption(format::truncate(&meta, 78)));
        }
        content = content.push(
            container(
                row![
                    description.width(Length::Fill),
                    badge::badge(
                        if is_included {
                            "À importer"
                        } else {
                            "Ignoré"
                        },
                        if is_included {
                            Tone::Success
                        } else {
                            Tone::Neutral
                        },
                    ),
                    controls::icon_action(
                        if is_included { Icon::Close } else { Icon::Plus },
                        if is_included { "Ignorer" } else { "Inclure" },
                        Message::ToggleProfileImportItem(item.key),
                    ),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .padding([space::SM, space::MD])
            .style(move |theme: &Theme| import_proposal_style(theme, is_included)),
        );
    }
    container(content)
        .padding(space::MD)
        .style(styles::panel)
        .into()
}

fn import_toned_tile(theme: &Theme, tone: Tone) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(tone.surface(&palette))),
        border: Border {
            color: tone.edge(&palette),
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}

fn import_metric_style(theme: &Theme, tone: Tone) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(if tone == Tone::Neutral {
            palette.panel
        } else {
            tone.surface(&palette)
        })),
        border: Border {
            color: if tone == Tone::Neutral {
                palette.border
            } else {
                tone.edge(&palette)
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}

fn import_proposal_style(theme: &Theme, included: bool) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(if included {
            alpha(palette.success, if palette.is_dark { 0.055 } else { 0.035 })
        } else {
            alpha(palette.sunken, 0.55)
        })),
        border: Border {
            color: if included {
                alpha(palette.success, 0.22)
            } else {
                palette.border
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}
