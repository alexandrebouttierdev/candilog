//! Domaine des analyses.

pub mod metrics;
pub mod period;
pub mod repository;

pub use metrics::{
    ActivityWeek, Analytics, Dashboard, Metrics, Performance, Step, ToFollowUp, UpcomingItem,
};
pub use period::Period;
pub use repository::AnalyticsRepository;
