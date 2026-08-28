//! Domaine des entretiens.

pub mod interview;
pub mod repository;

pub use interview::{InterviewAnalysis, Interview, NewInterview, InterviewType};
pub use repository::InterviewRepository;
