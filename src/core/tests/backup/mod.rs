//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations, DERNIERE_VERSION};

mod export_produit_une_base_candilog_valide;
mod import_echoue_sans_detruire_la_base_active;
mod import_restaure_une_base_validee;
mod validation_refuse_un_fichier_texte;
mod validation_refuse_une_base_plus_recente;
