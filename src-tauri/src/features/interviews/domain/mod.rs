//! Domaine des entretiens.

pub mod interview;
pub mod repository;

pub use interview::{Interview, InterviewAnalysis, InterviewType, NewInterview};
pub use repository::InterviewRepository;
