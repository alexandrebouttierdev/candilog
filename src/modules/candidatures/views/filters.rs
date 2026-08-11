//! Feuille de filtres du pipeline et jetons de filtres actifs.
//!
//! Repliée, elle n'occupe aucune place. Dépliée, elle tient sur deux lignes
//! compactes plutôt que dans une grande modale.

use crate::app::state::CandidateFilters;
use crate::app::{App, Message};
use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
use crate::ui::components::button as controls;
use crate::ui::components::choice::Choice;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, field, layout, surface, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Filtre de statut, avec une option « tous » explicite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    /// Aucun filtre de statut.
    All,
    /// Statut retenu.
    Value(StatutCandidature),
}

impl From<Option<StatutCandidature>> for StatusFilter {
    fn from(value: Option<StatutCandidature>) -> Self {
        value.map_or(Self::All, Self::Value)
    }
}

impl StatusFilter {
    /// Valeur métier associée.
    #[must_use]
    pub const fn value(self) -> Option<StatutCandidature> {
        match self {
            Self::All => None,
            Self::Value(value) => Some(value),
        }
    }

    /// Options proposées par le sélecteur.
    #[must_use]
    pub fn options() -> Vec<Self> {
        std::iter::once(Self::All)
            .chain(
                crate::modules::candidatures::components::PIPELINE
                    .into_iter()
                    .map(Self::Value),
            )
            .collect()
    }
}

impl std::fmt::Display for StatusFilter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => formatter.write_str("Tous les statuts"),
            Self::Value(value) => value.fmt(formatter),
        }
    }
}

/// Filtre de contrat, avec une option « tous » explicite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractFilter {
    /// Aucun filtre de contrat.
    All,
    /// Contrat retenu.
    Value(TypeContrat),
}

impl From<Option<TypeContrat>> for ContractFilter {
    fn from(value: Option<TypeContrat>) -> Self {
        value.map_or(Self::All, Self::Value)
    }
}

impl ContractFilter {
    /// Valeur métier associée.
    #[must_use]
    pub const fn value(self) -> Option<TypeContrat> {
        match self {
            Self::All => None,
            Self::Value(value) => Some(value),
        }
    }

    /// Options proposées par le sélecteur.
    #[must_use]
    pub fn options() -> Vec<Self> {
        vec![
            Self::All,
            Self::Value(TypeContrat::Cdi),
            Self::Value(TypeContrat::Cdd),
            Self::Value(TypeContrat::Freelance),
            Self::Value(TypeContrat::Stage),
            Self::Value(TypeContrat::Alternance),
            Self::Value(TypeContrat::Interim),
            Self::Value(TypeContrat::Autre),
        ]
    }
}

impl std::fmt::Display for ContractFilter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => formatter.write_str("Tous les contrats"),
            Self::Value(value) => value.fmt(formatter),
        }
    }
}

/// Décrit chaque filtre actif sous forme de jeton lisible.
#[must_use]
pub fn active_labels(filters: &CandidateFilters, companies: &[Choice]) -> Vec<String> {
    let mut labels = Vec::new();
    if let Some(status) = filters.status {
        labels.push(format!("Statut : {status}"));
    }
    if let Some(contract) = filters.contract {
        labels.push(format!("Contrat : {contract}"));
    }
    if let Some(id) = filters.company_id {
        let name = companies
            .iter()
            .find(|choice| choice.id == id)
            .map_or("Entreprise", |choice| choice.label.as_str());
        labels.push(format!("Entreprise : {name}"));
    }
    for (label, value) in [
        ("Ville", &filters.city),
        ("Poste", &filters.position),
        ("Depuis", &filters.date_from),
        ("Jusqu'au", &filters.date_to),
    ] {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            labels.push(format!("{label} : {trimmed}"));
        }
    }
    labels
}

/// Bandeau des filtres actifs, affiché seulement quand il porte une valeur.
pub fn active_strip<'a>(labels: &[String]) -> Element<'a, Message> {
    let mut line = row![typo::label("Filtres actifs")]
        .spacing(space::SM)
        .align_y(Alignment::Center);
    for label in labels {
        line = line.push(badge::badge(label.clone(), Tone::Accent));
    }
    line = line.push(layout::spacer()).push(
        controls::ghost("Réinitialiser", Some(Icon::Close))
            .on_press(Message::ResetCandidateFilters),
    );
    container(line)
        .width(Length::Fill)
        .padding([space::LG, 0.0])
        .into()
}

/// Panneau de filtres déplié sous la bande de pilotage, en verre.
pub fn sheet<'a>(app: &'a App, companies: Vec<Choice>) -> Element<'a, Message> {
    let filters = &app.candidate_filters;
    let selected_company = Choice::find(&companies, filters.company_id);
    let labels = active_labels(filters, &companies);
    let strip: Element<'_, Message> = if labels.is_empty() {
        iced::widget::Space::with_height(0).into()
    } else {
        column![surface::divider(), active_strip(&labels)]
            .spacing(0)
            .into()
    };

    container(
        column![
            surface::section_header(
                "Filtrer le pipeline",
                controls::ghost("Fermer", Some(Icon::Close)).on_press(Message::ToggleFilters),
            ),
            surface::divider(),
            column![
                field::form_row([
                    field::labeled(
                        "Statut",
                        field::select(
                            StatusFilter::options(),
                            Some(StatusFilter::from(filters.status)),
                            |choice| Message::CandidateFilterStatus(choice.value()),
                        )
                        .width(Length::Fill),
                    ),
                    field::labeled(
                        "Contrat",
                        field::select(
                            ContractFilter::options(),
                            Some(ContractFilter::from(filters.contract)),
                            |choice| Message::CandidateFilterContract(choice.value()),
                        )
                        .width(Length::Fill),
                    ),
                    field::labeled(
                        "Entreprise",
                        field::select(companies, selected_company, |choice| {
                            Message::CandidateFilterCompany(choice.value())
                        })
                        .width(Length::Fill),
                    ),
                ]),
                field::form_row([
                    field::text_field("Ville", &filters.city, Message::CandidateFilterCity),
                    field::text_field("Poste", &filters.position, Message::CandidateFilterPosition),
                    field::date_field(
                        "Depuis",
                        &filters.date_from,
                        None,
                        Message::CandidateFilterDateFrom,
                    ),
                    field::date_field(
                        "Jusqu'au",
                        &filters.date_to,
                        None,
                        Message::CandidateFilterDateTo,
                    ),
                ]),
            ]
            .spacing(space::LG)
            .padding([space::LG, 0.0]),
            strip,
        ]
        .spacing(0),
    )
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{active_labels, ContractFilter, StatusFilter};
    use crate::app::state::CandidateFilters;
    use crate::modules::candidatures::model::{StatutCandidature, TypeContrat};
    use crate::ui::components::choice::Choice;

    #[test]
    fn le_filtre_de_statut_expose_une_option_par_etape_plus_l_echappement() {
        let options = StatusFilter::options();
        assert_eq!(options.len(), 5);
        assert_eq!(options[0], StatusFilter::All);
        assert_eq!(options[0].value(), None);
        assert_eq!(options[1].value(), Some(StatutCandidature::EnAttente));
    }

    #[test]
    fn le_filtre_de_contrat_couvre_tous_les_contrats() {
        let options = ContractFilter::options();
        assert_eq!(options.len(), 8);
        assert_eq!(options[0].value(), None);
        assert_eq!(options[1].value(), Some(TypeContrat::Cdi));
    }

    #[test]
    fn les_conversions_depuis_l_etat_sont_reversibles() {
        assert_eq!(StatusFilter::from(None).value(), None);
        assert_eq!(
            StatusFilter::from(Some(StatutCandidature::Refus)).value(),
            Some(StatutCandidature::Refus)
        );
        assert_eq!(ContractFilter::from(None).value(), None);
        assert_eq!(
            ContractFilter::from(Some(TypeContrat::Stage)).value(),
            Some(TypeContrat::Stage)
        );
    }

    #[test]
    fn aucun_jeton_n_est_produit_sans_filtre_actif() {
        let filters = CandidateFilters::default();
        assert!(active_labels(&filters, &[]).is_empty());
        assert_eq!(filters.active_count(), 0);
    }

    #[test]
    fn chaque_filtre_actif_produit_un_jeton_lisible() {
        let company = uuid::Uuid::new_v4();
        let filters = CandidateFilters {
            status: Some(StatutCandidature::Entretien),
            contract: Some(TypeContrat::Cdi),
            company_id: Some(company),
            city: "  Rennes  ".into(),
            position: String::new(),
            date_from: "01-01-2026".into(),
            date_to: String::new(),
        };
        let companies = vec![Choice::new(company, "Agrial")];
        let labels = active_labels(&filters, &companies);
        assert_eq!(labels.len(), filters.active_count());
        assert!(labels.iter().any(|label| label == "Entreprise : Agrial"));
        assert!(labels.iter().any(|label| label == "Ville : Rennes"));
        assert!(!labels.iter().any(|label| label.starts_with("Poste")));
    }

    #[test]
    fn un_champ_rempli_d_espaces_n_est_pas_un_filtre() {
        let filters = CandidateFilters {
            city: "   ".into(),
            ..CandidateFilters::default()
        };
        assert_eq!(filters.active_count(), 0);
        assert!(active_labels(&filters, &[]).is_empty());
    }

    #[test]
    fn les_libelles_de_selecteur_restent_explicites() {
        assert_eq!(StatusFilter::All.to_string(), "Tous les statuts");
        assert_eq!(ContractFilter::All.to_string(), "Tous les contrats");
    }
}
