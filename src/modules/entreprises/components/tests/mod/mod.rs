//! Helpers communs et déclaration des cas de test.
use super::{matches, subtitle};
use crate::modules::entreprises::model::Entreprise;

fn entreprise(ville: Option<&str>, secteur: Option<&str>) -> Entreprise {
    Entreprise {
        id: uuid::Uuid::new_v4(),
        nom: "Agrial".into(),
        secteur: secteur.map(str::to_owned),
        type_: None,
        site_web: None,
        ville: ville.map(str::to_owned),
        adresse: None,
        notes: None,
        created_at: "2026-08-01".into(),
        updated_at: "2026-08-01".into(),
    }
}

mod la_recherche_couvre_nom_secteur_et_ville;
mod la_ville_prime_sur_le_secteur_en_sous_titre;
mod le_secteur_prend_le_relais_sans_ville;
