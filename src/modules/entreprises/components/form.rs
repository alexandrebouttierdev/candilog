//! Formulaire de création/édition d'une entreprise et avertissement de
//! suppression calculé depuis les contraintes du domaine.

use crate::app::{App, Message};
use crate::ui::components::field;
use crate::ui::format;
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Formulaire de création/édition d'une entreprise.
pub fn form(app: &App) -> Element<'_, Message> {
    let mut company_types = vec![
        "PME".to_owned(),
        "GROUPE".to_owned(),
        "ESN".to_owned(),
        "STARTUP".to_owned(),
        "TPE".to_owned(),
        "ETI".to_owned(),
        "ASSOCIATION".to_owned(),
        "PUBLIC".to_owned(),
        "CABINET".to_owned(),
        "AUTRE".to_owned(),
    ];
    if !app.entreprise_form.type_.trim().is_empty()
        && !company_types.contains(&app.entreprise_form.type_)
    {
        company_types.insert(0, app.entreprise_form.type_.clone());
    }
    let selected_type =
        (!app.entreprise_form.type_.trim().is_empty()).then(|| app.entreprise_form.type_.clone());
    column![
        field::text_field(
            "Nom *",
            &app.entreprise_form.nom,
            Message::EntrepriseNomChanged
        ),
        field::form_row([
            field::text_field(
                "Secteur",
                &app.entreprise_form.secteur,
                Message::EntrepriseSecteurChanged,
            ),
            field::labeled(
                "Type",
                field::select(company_types, selected_type, Message::EntrepriseTypeChanged,)
                    .width(iced::Length::Fill),
            ),
        ]),
        field::form_row([
            field::text_field(
                "Site web",
                &app.entreprise_form.site_web,
                Message::EntrepriseSiteChanged,
            ),
            field::text_field(
                "Ville",
                &app.entreprise_form.ville,
                Message::EntrepriseVilleChanged,
            ),
        ]),
        field::text_field(
            "Adresse",
            &app.entreprise_form.adresse,
            Message::EntrepriseAdresseChanged,
        ),
        field::labeled(
            "Notes",
            field::editor(
                &app.entreprise_form.notes,
                "Contexte, culture, informations utiles…"
            )
            .height(112.0)
            .on_action(Message::EntrepriseNotesChanged),
        ),
    ]
    .spacing(space::LG)
    .into()
}

/// Décrit le refus attendu quand l'entreprise porte encore des candidatures
/// (contrainte `RESTRICT` de la base).
pub fn consequences_suppression(app: &App, id: uuid::Uuid) -> String {
    let nom = app
        .data
        .entreprises
        .iter()
        .find(|item| item.id == id)
        .map_or_else(|| "Cette entreprise".to_owned(), |item| item.nom.clone());
    let liees = app
        .data
        .candidatures
        .iter()
        .filter(|item| item.entreprise_id == id)
        .count();
    if liees > 0 {
        format!(
            "« {nom} » ne peut pas être supprimée : {} y {} encore rattachée{}. Supprimez-les \
             d'abord, ou conservez l'entreprise.",
            format::plural(liees, "candidature", "candidatures"),
            if liees > 1 { "sont" } else { "est" },
            if liees > 1 { "s" } else { "" },
        )
    } else {
        format!("« {nom} » sera supprimée. Cette action est définitive.")
    }
}
