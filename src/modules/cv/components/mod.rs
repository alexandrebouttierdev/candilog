//! Rendu des objets de la bibliothèque de CV.

use crate::modules::cv::model::CvVersionSummary;
use crate::ui::components::{list, typo};
use crate::ui::format;
use iced::Element;

/// Ligne d'une version de CV dans le volet liste.
pub fn version_row<Message: Clone + 'static>(
    version: &CvVersionSummary,
    selected: bool,
    on_select: Message,
) -> Element<'static, Message> {
    list::row_item(
        version.name.clone(),
        format::compact_datetime(&version.created_at),
        typo::caption(""),
        selected,
        on_select,
    )
}

/// Libellé résumant l'état de la bibliothèque de CV.
#[must_use]
pub fn library_summary(count: usize) -> String {
    format::plural(count, "version enregistrée", "versions enregistrées")
}

#[cfg(test)]
mod tests {
    use super::library_summary;

    #[test]
    fn le_resume_de_bibliotheque_s_accorde() {
        assert_eq!(library_summary(0), "0 version enregistrée");
        assert_eq!(library_summary(1), "1 version enregistrée");
        assert_eq!(library_summary(3), "3 versions enregistrées");
    }
}
