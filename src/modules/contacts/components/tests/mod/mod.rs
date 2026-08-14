//! Helpers communs et déclaration des cas de test.
use super::{full_name, matches};
use crate::modules::contacts::model::Contact;

fn contact(poste: Option<&str>, email: Option<&str>) -> Contact {
    Contact {
        id: uuid::Uuid::new_v4(),
        entreprise_id: None,
        prenom: "Alex".into(),
        nom: "Bouttier".into(),
        poste: poste.map(str::to_owned),
        email: email.map(str::to_owned),
        telephone: None,
        linkedin: None,
        notes: None,
        created_at: "2026-08-01".into(),
        updated_at: "2026-08-01".into(),
    }
}

mod la_carte_de_contact_s_instancie_avec_et_sans_coordonnees;
mod la_recherche_couvre_nom_fonction_et_courriel;
mod le_nom_complet_est_normalise;
