//! Référentiels métier : secteurs d'activité, domaines professionnels, types d'entreprise
//! et types de contrat.
//!
//! Quatre catalogues **distincts** et jamais fusionnés : le secteur qualifie l'activité de
//! l'entreprise, le domaine professionnel le poste visé, le type d'entreprise la nature de
//! l'organisation, le type de contrat l'engagement.
//!
//! En lecture seule côté application : les listes sont semées par `init_schema.sql`, qui en
//! est l'unique source. Ni Rust ni React n'en tient de copie — un catalogue recopié dans un
//! tableau du frontend finirait par diverger de la base au premier ajout.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
