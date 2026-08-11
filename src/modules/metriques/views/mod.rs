//! Écrans Tableau de bord et Statistiques.

pub mod dashboard;
pub mod statistics;

pub use dashboard::view as dashboard_view;
pub use statistics::view as statistics_view;
pub use statistics::PAGE_SIZE;
