//! Domaine des entreprises.

pub mod company;
pub mod repository;

pub use company::{Company, CompanyUpdate, NewCompany};
pub use repository::CompanyRepository;
