//! Rendu des objets de candidature : statut, carte de pipeline, ligne de table.

use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::typo;
use crate::ui::components::{badge, surface};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::{Marker, Tone};
use iced::widget::{column, container, mouse_area, row, Space};
use iced::{mouse, Alignment, Element, Length};

pub mod form;

/// Ton sémantique associé à un statut de candidature.
#[must_use]
pub const fn status_tone(status: StatutCandidature) -> Tone {
    match status {
        StatutCandidature::EnAttente => Tone::Neutral,
        StatutCandidature::Relancee => Tone::Warning,
        StatutCandidature::Entretien => Tone::Success,
        StatutCandidature::Refus => Tone::Danger,
    }
}

/// Forme du marqueur associé à un statut : l'information ne dépend jamais de
/// la seule couleur.
#[must_use]
pub const fn status_marker(status: StatutCandidature) -> Marker {
    match status {
        StatutCandidature::EnAttente => Marker::Hollow,
        StatutCandidature::Relancee => Marker::Half,
        StatutCandidature::Entretien => Marker::Solid,
        StatutCandidature::Refus => Marker::Barred,
    }
}

/// Ordre du pipeline, du premier contact à l'issue.
pub const PIPELINE: [StatutCandidature; 4] = [
    StatutCandidature::EnAttente,
    StatutCandidature::Relancee,
    StatutCandidature::Entretien,
    StatutCandidature::Refus,
];

/// Libellé de colonne du pipeline.
#[must_use]
pub const fn column_label(status: StatutCandidature) -> &'static str {
    match status {
        StatutCandidature::EnAttente => "En attente",
        StatutCandidature::Relancee => "Relancées",
        StatutCandidature::Entretien => "Entretiens",
        StatutCandidature::Refus => "Refusées",
    }
}

/// Statut suivant dans le pipeline, s'il existe.
#[must_use]
pub fn next_status(status: StatutCandidature) -> Option<StatutCandidature> {
    let index = PIPELINE.iter().position(|value| *value == status)?;
    PIPELINE.get(index + 1).copied()
}

/// Statut précédent dans le pipeline, s'il existe.
#[must_use]
pub fn previous_status(status: StatutCandidature) -> Option<StatutCandidature> {
    let index = PIPELINE.iter().position(|value| *value == status)?;
    index
        .checked_sub(1)
        .and_then(|value| PIPELINE.get(value))
        .copied()
}

/// Jeton de statut complet, libellé et forme compris.
pub fn status_badge<'a, Message: 'a>(status: StatutCandidature) -> Element<'a, Message> {
    badge::status(
        status.to_string(),
        status_tone(status),
        status_marker(status),
    )
}

/// Abréviation d'un type de contrat, adaptée aux colonnes denses.
#[must_use]
pub const fn contract_short(contract: TypeContrat) -> &'static str {
    match contract {
        TypeContrat::Cdi => "CDI",
        TypeContrat::Cdd => "CDD",
        TypeContrat::Freelance => "Freelance",
        TypeContrat::Stage => "Stage",
        TypeContrat::Alternance => "Alternance",
        TypeContrat::Interim => "Intérim",
        TypeContrat::Autre => "Autre",
    }
}

/// Carte du pipeline : un objet autonome, saisissable au bouton gauche.
///
/// Le statut n'y figure pas : il est porté par la colonne. La carte présente
/// ce qui distingue une candidature d'une autre : poste, entreprise, contrat
/// en pastille et date. L'appui gauche lance un glisser passé un seuil de
/// déplacement ; un simple clic ouvre le détail (tranché dans `update`).
#[allow(clippy::too_many_arguments)]
pub fn kanban_card<'a, Message: Clone + 'a>(
    candidate: &'a Candidature,
    selected: bool,
    hovered: bool,
    on_press: Message,
    on_move: impl Fn(iced::Point) -> Message + 'a,
    on_release: Message,
    on_hover: Message,
    on_exit: Message,
) -> Element<'a, Message> {
    let content = column![
        crate::ui::components::tooltip::tip(
            typo::item(format::truncate(&candidate.poste, 30)).font(font::SEMIBOLD),
            candidate.poste.as_str(),
            crate::ui::components::tooltip::Side::Bottom,
        ),
        row![
            icon::icon(Icon::Building, 12.0, Ink::Muted),
            typo::meta(format::truncate(
                &format::or_else(candidate.entreprise_nom.as_deref(), "Entreprise inconnue"),
                30,
            )),
        ]
        .spacing(space::XS)
        .align_y(Alignment::Center),
        surface::divider(),
        row![
            badge::badge(contract_short(candidate.type_contrat), Tone::Neutral),
            Space::with_width(Length::Fill),
            icon::icon(Icon::Calendar, 11.0, Ink::Muted),
            typo::text_mono(
                format::compact_date(&candidate.date_envoi),
                font::CAPTION,
                font::MONO_REGULAR,
            )
            .style(styles::muted_text),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    ]
    .spacing(space::XS);

    mouse_area(
        container(content)
            .width(Length::Fill)
            .padding(12.0)
            .style(styles::kanban_card(selected, hovered)),
    )
    .on_press(on_press)
    .on_move(on_move)
    .on_release(on_release)
    .on_enter(on_hover)
    .on_exit(on_exit)
    .interaction(mouse::Interaction::Grab)
    .into()
}

/// Ligne d'activité liée à une candidature (entretien ou relance).
pub fn activity_row<'a, Message: 'a>(
    label: &'a str,
    detail: String,
    tone: Tone,
) -> Element<'a, Message> {
    row![
        badge::marker(tone, Marker::Solid),
        typo::body(label),
        Space::with_width(Length::Fill),
        typo::caption(detail),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .height(24)
    .into()
}

/// Contrôles de déplacement d'une candidature dans le pipeline.
pub fn move_controls<Message: Clone + 'static>(
    status: StatutCandidature,
    on_move: impl Fn(StatutCandidature) -> Message,
) -> Element<'static, Message> {
    let mut controls = row![].spacing(space::XS).align_y(Alignment::Center);
    if let Some(target) = previous_status(status) {
        controls = controls.push(controls::icon_action(
            Icon::ArrowLeft,
            "Étape précédente",
            on_move(target),
        ));
    }
    if let Some(target) = next_status(status) {
        controls = controls.push(controls::icon_action(
            Icon::ArrowRight,
            "Étape suivante",
            on_move(target),
        ));
    }
    controls.into()
}

/// Icône d'accompagnement d'une candidature dans une liste mixte.
pub fn glyph<'a, Message: 'a>(status: StatutCandidature) -> Element<'a, Message> {
    icon::toned(Icon::Applications, status_tone(status)).into()
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
