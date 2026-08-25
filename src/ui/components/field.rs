//! Champs et contrôles de saisie.
//!
//! Hauteur unique de 34 px, étiquette au-dessus, description et erreur sous le
//! champ concerné : jamais dans un bandeau distant.

use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{
    column, container, row, text_editor, toggler, PickList, TextEditor, TextInput, Toggler,
};
use iced::{Alignment, Element, Length};
use std::borrow::Borrow;

/// Champ texte nu, à composer dans une ligne ou une toolbar.
pub fn input<'a, Message: Clone>(placeholder: &'a str, value: &'a str) -> TextInput<'a, Message> {
    iced::widget::text_input(placeholder, value)
        .size(font::BODY)
        .padding([0.0, space::MD])
        .line_height(iced::widget::text::LineHeight::Absolute(
            size::FIELD_CONTROL.into(),
        ))
        .style(styles::input)
}

/// Champ de recherche : icône dessinée à gauche, largeur maîtrisée.
pub fn search<'a, Message: Clone + 'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    width: Length,
) -> Element<'a, Message> {
    row![
        icon::icon(Icon::Search, icon::SM, Ink::Muted),
        input(placeholder, value).on_input(on_input).width(width),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center)
    .into()
}

/// Champ de recherche avec échappement explicite quand une valeur est saisie.
///
/// Le bouton n'est pas rendu à vide : la largeur utile du champ reste ainsi stable et
/// l'utilisateur dispose d'une action locale, sans devoir sélectionner puis effacer le texte.
pub fn search_resettable<'a, Message: Clone + 'a>(
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_reset: Message,
    width: Length,
    focus_id: Option<&'static str>,
) -> Element<'a, Message> {
    let mut field = input(placeholder, value)
        .on_input(on_input)
        .width(Length::Fill);
    if let Some(id) = focus_id {
        field = field.id(iced::widget::text_input::Id::new(id));
    }
    let mut control = row![icon::icon(Icon::Search, icon::SM, Ink::Muted), field]
        .spacing(space::SM)
        .align_y(Alignment::Center)
        .width(width);
    if !value.trim().is_empty() {
        control = control.push(crate::ui::components::button::icon_action(
            Icon::Close,
            "Effacer la recherche",
            on_reset,
        ));
    }
    control.into()
}

/// Sélecteur au style du design system.
pub fn select<'a, T, L, V, Message>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
{
    iced::widget::pick_list(options, selected, on_selected)
        .placeholder("Sélectionner…")
        .text_size(font::BODY)
        .text_line_height(iced::widget::text::LineHeight::Absolute(
            size::FIELD_CONTROL.into(),
        ))
        .padding([0.0, space::MD])
        .handle(iced::widget::pick_list::Handle::Arrow {
            size: Some(icon::SM.into()),
        })
        .style(styles::select)
        .menu_style(styles::select_menu)
}

/// Éditeur multiligne au style du design system.
pub fn editor<'a, Message: Clone + 'a>(
    content: &'a text_editor::Content,
    placeholder: &'a str,
) -> TextEditor<'a, iced::advanced::text::highlighter::PlainText, Message> {
    iced::widget::text_editor(content)
        .placeholder(placeholder)
        .size(font::BODY)
        .padding(space::SM)
        .style(styles::editor)
}

/// Interrupteur d'une ligne de réglage.
pub fn switch<'a, Message: Clone + 'a>(
    active: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Toggler<'a, Message> {
    toggler(active)
        .on_toggle(on_toggle)
        .size(size::SWITCH)
        .style(styles::switch)
}

/// Enveloppe un contrôle d'une étiquette et, si besoin, d'une description.
pub fn labeled<'a, Message: 'a>(
    label: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![typo::label(label), control.into()]
        .spacing(space::XS)
        .width(Length::Fill)
        .into()
}

/// Champ texte étiqueté, cas le plus courant des formulaires.
pub fn text_field<'a, Message: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    labeled(
        label,
        input(label, value).on_input(on_input).width(Length::Fill),
    )
}

/// Champ texte étiqueté portant une erreur de validation sous le champ.
pub fn text_field_error<'a, Message: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    error: Option<&'a str>,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let mut group = column![
        typo::label(label),
        input(label, value).on_input(on_input).width(Length::Fill),
    ]
    .spacing(space::XS)
    .width(Length::Fill);
    if let Some(message) = error {
        group = group.push(typo::meta_toned(message, Tone::Danger));
    }
    group.into()
}

/// Champ de date au masque JJ/MM/AAAA, avec erreur sous le champ.
pub fn date_field<'a, Message: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    error: Option<&'a str>,
    on_input: impl Fn(String) -> Message + 'a,
    on_pick: Message,
) -> Element<'a, Message> {
    let mut group = column![
        typo::label(label),
        row![
            input("JJ-MM-AAAA", value)
                .on_input(on_input)
                .width(Length::Fill),
            crate::ui::components::button::icon_action(Icon::Calendar, "Choisir une date", on_pick),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    ]
    .spacing(space::XS)
    .width(Length::Fill);
    if let Some(message) = error {
        group = group.push(typo::meta_toned(message, Tone::Danger));
    }
    group.into()
}

/// Champ de date et heure au masque `JJ-MM-AAAA HH:MM`.
pub fn datetime_field<'a, Message: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    error: Option<&'a str>,
    on_input: impl Fn(String) -> Message + 'a,
    on_pick: Message,
) -> Element<'a, Message> {
    let mut group = column![
        typo::label(label),
        row![
            input("JJ-MM-AAAA HH:MM", value)
                .on_input(on_input)
                .width(Length::Fill),
            crate::ui::components::button::icon_action(Icon::Calendar, "Choisir une date", on_pick),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    ]
    .spacing(space::XS)
    .width(Length::Fill);
    if let Some(message) = error {
        group = group.push(typo::meta_toned(message, Tone::Danger));
    }
    group.into()
}

/// Champ masqué, pour un secret stocké dans le coffre système.
pub fn secret_field<'a, Message: Clone + 'a>(
    label: &'a str,
    value: &'a str,
    hint: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        typo::label(label),
        input(label, value)
            .secure(true)
            .on_input(on_input)
            .width(Length::Fill),
        typo::caption(hint),
    ]
    .spacing(space::XS)
    .width(Length::Fill)
    .into()
}

/// Aligne plusieurs champs sur une même ligne de formulaire.
pub fn form_row<'a, Message: 'a>(
    fields: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![].spacing(space::LG).align_y(Alignment::End);
    for field in fields {
        line = line.push(field);
    }
    line.into()
}

/// Section de formulaire avec repère iconographique, titre, description et contenu.
pub fn form_section<'a, Message: 'a>(
    glyph: Icon,
    title: &'a str,
    description: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            row![
                container(icon::icon(glyph, icon::SM, Ink::Accent))
                    .width(32.0)
                    .height(32.0)
                    .center(Length::Fixed(32.0))
                    .style(styles::icon_tile(crate::ui::theme::Tone::Accent)),
                column![typo::label(title), typo::caption(description)].spacing(space::XXS),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            crate::ui::components::surface::divider(),
            content.into(),
        ]
        .spacing(space::MD),
    )
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::form_group)
    .into()
}
