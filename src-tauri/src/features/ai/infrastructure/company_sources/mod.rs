//! Sources publiques, gratuites et sans clé de la recherche d'entreprise.
//!
//! Chacune implémente un contrat de `domain::company_research` ; en ajouter ou en remplacer
//! une ne touche ni au service ni à l'interface.

mod http;
mod pages;
mod registry;
mod searxng;
mod wikidata;

use crate::features::ai::domain::{CompanySources, SourceError};
use std::sync::Arc;

pub use pages::PublicPages;
pub use registry::RechercheEntreprises;
pub use searxng::Searxng;
pub use wikidata::Wikidata;

/// Sources de production : annuaire officiel, Wikidata, SearXNG et lecture des sites.
///
/// # Errors
/// `Unavailable` si un client HTTP ne peut être construit.
pub fn production_sources() -> Result<CompanySources, SourceError> {
    Ok(CompanySources {
        registry: Arc::new(RechercheEntreprises::new(registry::API_URL)?),
        web: Arc::new(Searxng::new(Some(searxng::DIRECTORY_URL))?),
        knowledge: Arc::new(Wikidata::new(wikidata::SPARQL_URL)?),
        pages: Arc::new(PublicPages::new()),
    })
}
