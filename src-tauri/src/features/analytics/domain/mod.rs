//! Domaine des analyses.

pub mod metrics;
pub mod period;
pub mod repository;

pub use metrics::{
    ToFollowUp, Analytics, UpcomingItem, Step, Metrics, Performance, ActivityWeek, Dashboard,
};
pub use period::Period;
pub use repository::AnalyticsRepository;
