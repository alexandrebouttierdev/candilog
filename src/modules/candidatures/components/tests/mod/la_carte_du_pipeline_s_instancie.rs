//! Cas de test isolé.

use super::*;
use crate::modules::candidatures::components::kanban_card;

#[test]
fn la_carte_du_pipeline_s_instancie() {
    use crate::modules::candidatures::model::Candidature;
    use uuid::Uuid;
    let candidature = Candidature {
        id: Uuid::new_v4(),
        poste: "Développeur Rust".into(),
        entreprise_id: Uuid::new_v4(),
        entreprise_nom: Some("Agrial".into()),
        contact_id: None,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-08-01".into(),
        lien_offre: None,
        notes: None,
        created_at: "2026-08-01".into(),
        updated_at: "2026-08-01".into(),
    };
    let _: iced::Element<'_, ()> = kanban_card(&candidature, false, false, (), |_| (), (), (), ());
}
