//! Helpers communs et déclaration des cas de test.
use super::*;

fn parse(json: &str) -> Profile {
    serde_json::from_str::<ExtractedProfile>(json)
        .unwrap()
        .into()
}

mod test_dedoublonne_les_competences_insensible_a_la_casse;
mod test_experience_en_cours_efface_la_date_de_fin;
mod test_extraction_tolere_scalaires_et_champs_absents;
mod test_ignore_les_entrees_sans_contenu_utile;
mod test_normalise_les_dates_mm_slash_yyyy;
mod test_normalise_les_niveaux_de_langue;
mod test_profile_is_empty_detecte_un_profil_vide;
