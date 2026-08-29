//! Features métier. Chacune expose `domain`, `application`, `infrastructure`
//! et `presentation`, et est ajoutée ici au fur et à mesure de la migration
//! (cf. `docs/migration/01-AUDIT.md`, §7).

pub mod ai;
pub mod analytics;
pub mod applications;
pub mod companies;
pub mod contacts;
pub mod documents;
pub mod followups;
pub mod interviews;
pub mod profile;
pub mod sectors;
pub mod settings;
