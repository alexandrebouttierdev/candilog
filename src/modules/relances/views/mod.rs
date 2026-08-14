//! Vues natives des relances : lignes d'activité rattachées à une candidature.

use crate::app::{App, Message};
use crate::modules::candidatures::components::activity_row;
use crate::ui::format;
use crate::ui::theme::Tone;
use iced::Element;

/// Lignes d'activité des relances d'une candidature.
pub fn activity_rows<'a>(app: &'a App, candidature_id: uuid::Uuid) -> Vec<Element<'a, Message>> {
    app.data
        .relances
        .iter()
        .filter(|item| item.candidature_id == candidature_id)
        .map(|reminder| {
            activity_row(
                "Relance",
                format!(
                    "{} · {}",
                    format::compact_date(&reminder.date_relance),
                    reminder.type_relance
                ),
                Tone::Warning,
            )
        })
        .collect()
}
