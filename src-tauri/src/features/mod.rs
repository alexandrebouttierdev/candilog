//! Features métier. Chacune expose `domain`, `application`, `infrastructure`
//! et `presentation`, et est ajoutée ici au fur et à mesure de la migration
//! (cf. `docs/migration/01-AUDIT.md`, §7).

pub mod analytics;
pub mod applications;
pub mod contacts;
pub mod documents;
pub mod companies;
pub mod interviews;
pub mod ai;
pub mod settings;
pub mod profile;
pub mod followups;
pub mod sectors;
