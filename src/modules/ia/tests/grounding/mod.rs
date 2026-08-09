//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::profile::{Certification, Experience, Profile, Skill};

const CV: &str = "Ingénieure logicielle chez ACME Corporation. Compétences : Rust, PostgreSQL. \
                      Certifiée AWS Solutions Architect.";

mod test_ground_profile_conserve_les_champs_cles_vides;
mod test_ground_profile_supprime_une_entreprise_inventee;
mod test_is_grounded_rejette_une_valeur_absente;
mod test_is_grounded_tolere_casse_et_accents;
