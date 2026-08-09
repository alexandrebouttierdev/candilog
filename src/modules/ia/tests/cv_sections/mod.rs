//! Helpers communs et déclaration des cas de test.
use super::*;

const CV: &str = "Ada Lovelace\nada@x.io — Londres\n\nExpériences professionnelles\nIngénieure, ACME (2022-)\n\nFormation\nMSc Maths, Cambridge\n\nCompétences\nRust, SQL\n\nLangues\nAnglais C1\n\nProjets\nMoteur analytique\n\nCertifications\nAWS Architect";

mod test_detect_heading_ignore_les_lignes_longues_et_le_corps;
mod test_detect_heading_licence_diplome_va_en_formation;
mod test_split_cv_formation_manquante_replie_le_parcours_seulement;
mod test_split_cv_fragmente_chaque_section_au_bon_appel;
mod test_split_cv_projets_absents_replie_le_portfolio;
mod test_split_cv_sans_structure_replie_sur_le_texte_complet;
