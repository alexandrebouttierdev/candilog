//! Contrats d'entrée du domaine contacts, un DTO par fichier.

mod create_contact;
mod update_contact;

pub use create_contact::CreateContactDto;
pub use update_contact::UpdateContactDto;
