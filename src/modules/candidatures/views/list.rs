//! Vue Liste : véritable table de données desktop, en-tête figé et triable.

use crate::app::state::{CandidateSort, Dialog};
use crate::app::{App, Message};
use crate::modules::candidatures::components::{contract_short, status_badge};
use crate::modules::candidatures::model::Candidature;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::table::{self, Column, SortOrder};
use crate::ui::components::{state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Colonnes de la table, dans l'ordre des critères de tri. Seule POSTE (le
/// titre de la ligne) reste toujours visible ; les autres sont secondaires
/// et reviennent en métadonnée sous le titre en dessous de 1280 px.
const COLUMNS: [Column; 4] = [
    Column::text("POSTE", 4),
    Column::text("ENTREPRISE", 3).secondary(),
    Column::centered("STATUT", 132.0).secondary(),
    Column::trailing("ENVOYÉE LE", 96.0).secondary(),
];

/// Colonne d'actions, hors tri.
const ACTIONS: Column = Column::trailing("", 92.0);

/// Rend la table des candidatures déjà filtrées et triées.
pub fn view<'a>(app: &'a App, candidates: &[&'a Candidature]) -> Element<'a, Message> {
    let mut columns: Vec<Column> = COLUMNS.to_vec();
    columns.push(ACTIONS);

    let order = if app.candidate_sort_descending {
        SortOrder::Descending
    } else {
        SortOrder::Ascending
    };

    let body: Element<'a, Message> = if candidates.is_empty() {
        state::empty(
            "Aucune candidature",
            "Modifiez les filtres ou créez une candidature.",
        )
    } else {
        let mut rows = column![];
        for candidate in candidates {
            rows = rows.push(data_row(app, candidate, &columns));
        }
        surface::scroll(rows).height(Length::Fill).into()
    };

    container(
        column![
            table::header_sortable_styled(
                app.layout(),
                &columns,
                app.candidate_sort.column(),
                order,
                sortable_message,
                styles::list_header,
            ),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Une colonne sans critère de tri ne renvoie pas de message de tri.
fn sortable_message(index: usize) -> Message {
    CandidateSort::from_column(index).map_or(Message::SelectCandidate(None), |_| {
        Message::SortCandidates(index)
    })
}

fn data_row<'a>(app: &App, candidate: &'a Candidature, columns: &[Column]) -> Element<'a, Message> {
    let id = candidate.id;
    let cells = vec![
        table::cell(
            columns[0],
            column![
                typo::item(format::truncate(&candidate.poste, 46)),
                typo::caption(contract_short(candidate.type_contrat)),
            ]
            .spacing(0),
        ),
        table::cell(
            columns[1],
            typo::meta(format::or_dash(candidate.entreprise_nom.as_deref())),
        ),
        table::cell(columns[2], status_badge(candidate.statut)),
        table::cell(
            columns[3],
            typo::body(format::compact_date(&candidate.date_envoi)),
        ),
        table::cell(
            columns[4],
            row![
                controls::icon_action(Icon::Edit, "Modifier", Message::EditCandidature(id),),
                controls::icon_danger(
                    Icon::Trash,
                    "Supprimer",
                    Message::OpenDialog(Dialog::DeleteCandidature(id)),
                ),
            ]
            .spacing(space::XXS)
            .align_y(Alignment::Center),
        ),
    ];
    table::row_button(
        app.layout(),
        cells,
        app.selected_candidate == Some(id),
        Message::OpenDialog(Dialog::CandidatureDetail(id)),
    )
}

#[cfg(test)]
mod tests {
    use super::{ACTIONS, COLUMNS};
    use crate::app::state::CandidateSort;
    use crate::ui::components::table::Align;

    #[test]
    fn chaque_colonne_triable_correspond_a_un_critere() {
        assert_eq!(COLUMNS.len(), CandidateSort::ALL.len());
        for (index, _) in COLUMNS.iter().enumerate() {
            assert!(CandidateSort::from_column(index).is_some());
        }
        assert!(CandidateSort::from_column(COLUMNS.len()).is_none());
    }

    #[test]
    fn les_criteres_de_tri_retrouvent_leur_colonne() {
        for sort in CandidateSort::ALL {
            assert_eq!(CandidateSort::from_column(sort.column()), Some(sort));
        }
    }

    #[test]
    fn les_alignements_suivent_la_nature_des_donnees() {
        assert_eq!(COLUMNS[0].align, Align::Start);
        assert_eq!(COLUMNS[1].align, Align::Start);
        assert_eq!(COLUMNS[2].align, Align::Center);
        assert_eq!(COLUMNS[3].align, Align::End);
        assert_eq!(ACTIONS.align, Align::End);
    }

    #[test]
    fn la_colonne_d_actions_reste_discrete() {
        assert!(ACTIONS.label.is_empty());
    }

    #[test]
    fn le_tri_par_defaut_porte_sur_la_date() {
        assert_eq!(CandidateSort::default(), CandidateSort::Date);
    }
}
