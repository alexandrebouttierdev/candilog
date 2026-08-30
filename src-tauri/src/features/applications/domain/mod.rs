//! Domaine des candidatures.

pub mod application;
pub mod application_type;
pub mod repository;
pub mod schedule;
pub mod status;

pub use application::{Application, NewApplication};
pub use application_type::ApplicationType;
pub use repository::{
    ApplicationFilter, ApplicationRepository, ApplicationSort, PipelineBreakdown,
};
pub use schedule::{WeeklyWorkSchedule, MAX_WEEKLY_HOURS};
pub use status::ApplicationStatus;
