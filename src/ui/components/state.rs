//! États vides, chargements et erreurs inline.
//!
//! Les états vides desktop restent sobres : un plancher de 256 px, une icône
//! et le texte expliquant la suite possible, sans blason ni illustration.

use super::button as controls;
use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, progress_bar, row, scrollable, Space, Stack};
use iced::{Alignment, Element, Length};

/// Plancher de hauteur des états vides (`min-h-64`) : Iced 0.13 ne connaît pas
/// de hauteur minimale, une base invisible de 256 px la fixe dans un `Stack`.
///
/// Ce plancher rend la hauteur **intrinsèque** de l'état vide supérieure à son contenu réel.
/// Quand le conteneur parent en accorde moins, Iced rogne par le bas au lieu d'adapter : sur
/// le tableau de bord d'un premier lancement, l'aide se réduisait à un liseré de quelques
/// pixels et le bouton « Nouvelle candidature » n'était **pas rendu du tout** — l'incitation
/// à l'action disparaissait exactement au moment où elle est utile.
///
/// Le défilement garantit que le contenu reste atteignable quelle que soit la place accordée ;
/// la barre n'apparaît que lorsqu'elle est nécessaire.
fn empty_floor<'a, Message: 'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    scrollable(
        Stack::with_children(vec![
            container(Space::with_height(Length::Fixed(256.0)))
                .width(Length::Fill)
                .into(),
            container(content).center(Length::Fill).into(),
        ])
        .width(Length::Fill),
    )
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(6).scroller_width(5),
    ))
    .into()
}

/// État vide intégré à un panneau.
pub fn empty<'a, Message: 'a>(title: &'a str, hint: &'a str) -> Element<'a, Message> {
    container(empty_floor(
        column![
            icon::icon(Icon::Inbox, 48.0, Ink::Muted),
            typo::body(title).size(crate::ui::theme::typography::ITEM),
            typo::caption(hint),
        ]
        .spacing(space::MD)
        .align_x(Alignment::Center)
        .into(),
    ))
    .padding([space::XL, space::XL])
    .width(Length::Fill)
    .style(styles::dashed)
    .into()
}

/// État vide proposant l'action qui le résout.
pub fn empty_with_action<'a, Message: Clone + 'a>(
    title: &'a str,
    hint: &'a str,
    action: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    container(empty_floor(
        column![
            icon::icon(Icon::Inbox, 48.0, Ink::Muted),
            typo::body(title).size(crate::ui::theme::typography::ITEM),
            typo::caption(hint),
            controls::ghost(action, Some(Icon::Plus)).on_press(on_press),
        ]
        .spacing(space::MD)
        .align_x(Alignment::Center)
        .into(),
    ))
    .padding([space::XL, space::XL])
    .width(Length::Fill)
    .style(styles::dashed)
    .into()
}

/// État vide minimal, pour une colonne étroite.
pub fn empty_slot<'a, Message: 'a>(hint: &'a str) -> Element<'a, Message> {
    container(typo::caption(hint))
        .padding([space::XL, space::MD])
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// État vide d'un volet de détail sans sélection.
pub fn no_selection<'a, Message: 'a>(hint: &'a str) -> Element<'a, Message> {
    container(
        column![
            icon::icon(Icon::Panel, icon::LG, Ink::Muted),
            typo::caption(hint),
        ]
        .spacing(space::MD)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}

/// Opération longue non bloquante, avec étape, durée et arrêt.
pub fn running<'a, Message: Clone + 'a>(
    step: &'a str,
    elapsed_seconds: u64,
    on_cancel: Message,
) -> Element<'a, Message> {
    container(
        column![
            row![
                icon::toned(Icon::Sparkles, Tone::Accent),
                typo::body(step),
                Space::with_width(Length::Fill),
                typo::caption(format!("{elapsed_seconds} s")),
                controls::ghost("Arrêter", Some(Icon::Stop)).on_press(on_cancel),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            progress_bar(0.0..=1.0, indeterminate(elapsed_seconds))
                .height(size::PROGRESS_BAR)
                .style(styles::progress(Tone::Accent)),
        ]
        .spacing(space::SM),
    )
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::sunken)
    .into()
}

/// Bloc de progression IA : icône 64 px, titre, chrono `mm:ss`, arrêt.
pub fn ai_progress<'a, Message: Clone + 'a>(
    step: &'a str,
    elapsed_seconds: u64,
    on_cancel: Message,
) -> Element<'a, Message> {
    let minutes = elapsed_seconds / 60;
    let seconds = elapsed_seconds % 60;
    container(
        column![
            container(icon::icon(Icon::Sparkles, 28.0, Ink::Accent))
                .width(64.0)
                .height(64.0)
                .center(Length::Fixed(64.0))
                .style(styles::toned(Tone::Accent)),
            typo::section(step),
            typo::text_mono(
                format!("{minutes:02}:{seconds:02}"),
                13.0,
                crate::ui::theme::typography::MONO_SEMIBOLD,
            ),
            controls::ghost("Arrêter", Some(Icon::Stop)).on_press(on_cancel),
        ]
        .spacing(space::MD)
        .align_x(Alignment::Center),
    )
    .padding(space::XXL)
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}

/// Progression déterminée d'une opération longue.
pub fn progress_step<'a, Message: 'a>(step: &'a str, ratio: f32) -> Element<'a, Message> {
    let percent = (ratio.clamp(0.0, 1.0) * 100.0).round() as u8;
    container(
        column![
            row![
                typo::body(step),
                Space::with_width(Length::Fill),
                typo::caption(format!("{percent} %")),
            ]
            .align_y(Alignment::Center),
            progress_bar(0.0..=1.0, ratio.clamp(0.0, 1.0))
                .height(size::PROGRESS_BAR)
                .style(styles::progress(Tone::Accent)),
        ]
        .spacing(space::SM),
    )
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::sunken)
    .into()
}

/// Message d'erreur inline, posé au plus près de la zone concernée.
pub fn error<'a, Message: 'a>(message: impl Into<String>) -> Element<'a, Message> {
    container(
        row![
            icon::toned(Icon::Alert, Tone::Danger),
            typo::toned(message.into(), Tone::Danger),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .padding([space::SM, space::LG])
    .width(Length::Fill)
    .style(styles::toned(Tone::Danger))
    .into()
}

/// Encart d'information contextuelle.
pub fn hint<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    container(
        row![icon::toned(Icon::Info, Tone::Info), typo::meta(message),]
            .spacing(space::SM)
            .align_y(Alignment::Center),
    )
    .padding([space::SM, space::LG])
    .width(Length::Fill)
    .style(styles::sunken)
    .into()
}

/// Écran d'erreur bloquante d'initialisation, avec ses issues de secours.
///
/// L'écran n'offrait qu'un bouton « Réessayer » câblé sur le rechargement des *données*, qui
/// abandonne dès sa première ligne quand le backend n'a pas pu être construit — soit exactement
/// la situation qui amène ici. Le bouton ne faisait rien, et les mécanismes de récupération
/// existants n'étaient joignables que depuis l'écran Paramètres, lui-même inatteignable.
///
/// `actions` porte donc toutes les issues, la première étant mise en avant.
pub fn fatal<'a, Message: Clone + 'a>(
    message: &'a str,
    actions: Vec<(&'a str, Icon, Message)>,
) -> Element<'a, Message> {
    let mut boutons = row![].spacing(space::MD).align_y(Alignment::Center);
    for (index, (label, glyph, action)) in actions.into_iter().enumerate() {
        boutons = boutons.push(if index == 0 {
            controls::secondary(label, Some(glyph)).on_press(action)
        } else {
            controls::ghost(label, Some(glyph)).on_press(action)
        });
    }
    container(
        container(
            column![
                icon::toned(Icon::Alert, Tone::Danger),
                typo::title("Candilog ne peut pas démarrer"),
                typo::body(message),
                typo::caption(
                    "Vos données n'ont pas été modifiées. Réessayez, restaurez un backup, ou \
                     redémarrez sur une base neuve — l'ancienne sera conservée."
                ),
                boutons,
            ]
            .spacing(space::LG)
            .align_x(Alignment::Center),
        )
        .padding(space::MAX)
        .max_width(size::DIALOG_FORM)
        .style(styles::panel),
    )
    .center(Length::Fill)
    .style(styles::canvas)
    .into()
}

/// Progression cyclique d'une opération dont la durée est inconnue.
fn indeterminate(elapsed_seconds: u64) -> f32 {
    let cycle = (elapsed_seconds % 12) as f32 / 12.0;
    0.08 + cycle * 0.84
}

#[cfg(test)]
mod tests {
    use super::indeterminate;

    #[test]
    fn la_progression_indeterminee_reste_dans_les_bornes() {
        for seconds in 0..120 {
            let value = indeterminate(seconds);
            assert!((0.0..=1.0).contains(&value), "progression hors bornes");
        }
    }

    #[test]
    fn la_progression_indeterminee_avance_puis_recommence() {
        assert!(indeterminate(1) > indeterminate(0));
        assert!((indeterminate(12) - indeterminate(0)).abs() < f32::EPSILON);
    }

    #[test]
    fn la_progression_indeterminee_ne_touche_jamais_les_extremes() {
        assert!(indeterminate(0) > 0.0);
        assert!(indeterminate(11) < 1.0);
    }
}
