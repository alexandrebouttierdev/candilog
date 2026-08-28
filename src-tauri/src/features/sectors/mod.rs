//! Référentiel des secteurs d'activité.
//!
//! Alimente le sélecteur du formulaire entreprise. En lecture seule côté application : la
//! liste canonique est insérée au démarrage, les valeurs libres héritées de l'ancienne base
//! y sont rattachées, et rien d'autre n'écrit dans la table.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
