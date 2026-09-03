//! Modèle métier du profil professionnel.

pub mod import;
pub mod photo;
pub mod profile;
pub mod repository;

pub use import::{
    apply_decisions, build_preview, ImportCertificationDecision, ImportCertificationItem,
    ImportDetectedCounts, ImportEducationDecision, ImportEducationItem, ImportExperienceDecision,
    ImportExperienceItem, ImportLanguageDecision, ImportLanguageItem, ImportProfilePreview,
    ImportProfileRequest, ImportProfileResult, ImportProjectDecision, ImportProjectItem,
    ImportResolution, ImportScalarDecision, ImportScalarItem, ImportSkillDecision, ImportSkillItem,
};
pub use photo::{normaliser, nouveau_nom_fichier, ACCEPTED_EXTENSIONS, MAX_SIDE, MAX_SOURCE_BYTES};
pub use profile::{
    Certification, Education, Experience, Identity, Language, Profile, ProfilePayload, Project,
    Skill,
};
pub use repository::ProfileRepository;
