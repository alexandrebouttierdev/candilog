//! Rendu des objets de candidature : statut, carte de pipeline, ligne de table.

use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::typo;
use crate::ui::components::{badge, surface};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::{Marker, Tone};
use iced::widget::{button, column, mouse_area, row, Space};
use iced::{mouse, Alignment, Element, Length};

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

/// Carte du pipeline : un objet autonome, saisissable, volontairement sobre.
///
/// Le statut n'y figure pas : il est porté par la colonne. La carte ne
/// présente que ce qui distingue une candidature d'une autre : poste,
/// entreprise, contrat en pastille et date en chiffres.
pub fn kanban_card<Message: Clone + 'static>(
    candidate: &Candidature,
    selected: bool,
    on_select: Message,
    on_drag: Message,
) -> Element<'static, Message> {
    let content = column![
        typo::item(format::truncate(&candidate.poste, 30)),
        typo::meta(format::or_else(
            candidate.entreprise_nom.as_deref(),
            "Entreprise inconnue"
        )),
        surface::divider(),
        row![
            badge::badge(contract_short(candidate.type_contrat), Tone::Neutral),
            Space::with_width(Length::Fill),
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
        button(content)
            .width(Length::Fill)
            .padding(14.0)
            .style(if selected {
                styles::card_selected
            } else {
                styles::card
            })
            .on_press(on_select),
    )
    .on_right_press(on_drag)
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
mod tests {
    use super::{
        column_label, contract_short, next_status, previous_status, status_marker, status_tone,
        PIPELINE,
    };
    use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
    use crate::ui::theme::{Marker, Tone};

    #[test]
    fn chaque_statut_porte_un_ton_et_une_forme_propres() {
        let mut tones = Vec::new();
        let mut markers = Vec::new();
        for status in PIPELINE {
            tones.push(status_tone(status));
            markers.push(status_marker(status));
        }
        for index in 0..PIPELINE.len() {
            for other in index + 1..PIPELINE.len() {
                assert_ne!(tones[index], tones[other], "tons de statut identiques");
                assert_ne!(
                    markers[index], markers[other],
                    "formes de statut identiques"
                );
            }
        }
    }

    #[test]
    fn l_attente_reste_neutre_pour_ne_pas_saturer_le_pipeline() {
        assert_eq!(status_tone(StatutCandidature::EnAttente), Tone::Neutral);
        assert_eq!(status_marker(StatutCandidature::EnAttente), Marker::Hollow);
    }

    #[test]
    fn le_refus_est_le_seul_statut_barre() {
        assert_eq!(status_marker(StatutCandidature::Refus), Marker::Barred);
        assert_eq!(status_tone(StatutCandidature::Refus), Tone::Danger);
    }

    #[test]
    fn le_pipeline_est_parcourable_dans_les_deux_sens() {
        assert_eq!(
            next_status(StatutCandidature::EnAttente),
            Some(StatutCandidature::Relancee)
        );
        assert_eq!(next_status(StatutCandidature::Refus), None);
        assert_eq!(previous_status(StatutCandidature::EnAttente), None);
        assert_eq!(
            previous_status(StatutCandidature::Entretien),
            Some(StatutCandidature::Relancee)
        );
    }

    #[test]
    fn chaque_colonne_porte_un_libelle_distinct() {
        let labels: std::collections::BTreeSet<_> = PIPELINE
            .iter()
            .map(|status| column_label(*status))
            .collect();
        assert_eq!(labels.len(), PIPELINE.len());
    }

    #[test]
    fn les_contrats_ont_une_abreviation_courte() {
        for contract in [
            TypeContrat::Cdi,
            TypeContrat::Cdd,
            TypeContrat::Freelance,
            TypeContrat::Stage,
            TypeContrat::Alternance,
            TypeContrat::Interim,
            TypeContrat::Autre,
        ] {
            let short = contract_short(contract);
            assert!(!short.is_empty());
            assert!(short.chars().count() <= 10, "abréviation trop longue");
        }
    }
}
