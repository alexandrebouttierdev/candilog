//! Composition et validation du document de travail autonome d'un CV.

use super::resume_document::{format_month_date, split_bullets};
use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::search_key;
use crate::features::ai::domain::{
    score_resume_imported, AtsRecommendation, AtsRecommendationSection, GeneratedEducation,
    GeneratedExperience, GeneratedResume, ResumeGeneration, MAX_ITEMS, MAX_ITEM_CHARS,
};
use crate::features::documents::domain::{
    ResumeCertificationBlock, ResumeDocument, ResumeEducationBlock, ResumeExperienceBlock,
    ResumeIdentity, ResumeLanguageBlock, ResumeProjectBlock, ResumeProposal, ResumeProposalKind,
    ResumeProposalStatus, ResumeProposalTarget, ResumeSkillGroup, ResumeWorkspace,
    RESUME_WORKSPACE_VERSION,
};
use crate::features::profile::domain::Profile;
use uuid::Uuid;

/// Identifiant du groupe de compétences créé à la volée quand le CV n'en porte encore aucun.
const DEFAULT_SKILL_GROUP_ID: &str = "competences";

/// Fige le profil et la génération IA dans un document qui ne dépend plus de leurs sources.
///
/// # Errors
/// Retourne une validation si le document composé dépasse les bornes d'édition.
pub fn prepare_workspace(
    profile: &Profile,
    generation: ResumeGeneration,
) -> AppResult<ResumeWorkspace> {
    let ResumeGeneration {
        resume,
        analysis,
        job_offer,
        ..
    } = generation;
    let identity = &profile.identity;
    let document = ResumeDocument {
        identity: ResumeIdentity {
            full_name: [identity.first_name.trim(), identity.name.trim()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            title: identity.title.as_deref().unwrap_or_default().trim().into(),
            headline: None,
            city: trimmed_option(identity.city.as_deref()),
            phone: trimmed_option(identity.phone.as_deref()),
            email: identity.email.trim().into(),
            website: trimmed_option(identity.website.as_deref()),
            linkedin: trimmed_option(identity.linkedin.as_deref()),
            github: trimmed_option(identity.github.as_deref()),
            extra: Vec::new(),
        },
        profile: resume.resume,
        experiences: resume
            .experiences
            .into_iter()
            .map(|experience| {
                let source = profile.experiences.iter().find(|source| {
                    source.title.trim() == experience.title.trim()
                        && source.company.trim() == experience.company.trim()
                });
                ResumeExperienceBlock {
                    id: Uuid::new_v4().to_string(),
                    title: experience.title,
                    company: experience.company,
                    location: source.and_then(|value| trimmed_option(value.location.as_deref())),
                    period: source.map_or_else(String::new, |value| {
                        format_workspace_period(
                            Some(&value.start_date),
                            value.end_date.as_deref(),
                            value.current,
                        )
                    }),
                    bullets: split_bullets(&experience.description),
                }
            })
            .collect(),
        projects: profile
            .projects
            .iter()
            .map(|project| ResumeProjectBlock {
                id: Uuid::new_v4().to_string(),
                name: project.name.clone(),
                meta: trimmed_option(project.technologies.as_deref()),
                url: trimmed_option(project.url.as_deref()),
                bullets: project
                    .description
                    .as_deref()
                    .map(split_bullets)
                    .unwrap_or_default(),
            })
            .collect(),
        skill_groups: if resume.skills.is_empty() {
            Vec::new()
        } else {
            vec![ResumeSkillGroup {
                id: Uuid::new_v4().to_string(),
                name: "Compétences".into(),
                items: resume.skills,
            }]
        },
        education: resume
            .education
            .into_iter()
            .map(|education| {
                let source = profile.education.iter().find(|source| {
                    source.degree.trim() == education.degree.trim()
                        && source.school.trim() == education.school.trim()
                });
                ResumeEducationBlock {
                    id: Uuid::new_v4().to_string(),
                    degree: education.degree,
                    school: education.school,
                    location: source.and_then(|value| trimmed_option(value.location.as_deref())),
                    period: source.map_or_else(String::new, |value| {
                        format_workspace_period(
                            value.start_date.as_deref(),
                            value.end_date.as_deref(),
                            false,
                        )
                    }),
                    description: source
                        .and_then(|value| trimmed_option(value.description.as_deref())),
                }
            })
            .collect(),
        certifications: profile
            .certifications
            .iter()
            .map(|certification| ResumeCertificationBlock {
                id: Uuid::new_v4().to_string(),
                name: certification.name.clone(),
                issuer: trimmed_option(certification.issuer.as_deref()),
                date: certification
                    .date
                    .as_deref()
                    .map(format_month_date)
                    .and_then(|date| trimmed_option(Some(&date))),
            })
            .collect(),
        languages: profile
            .languages
            .iter()
            .map(|language| ResumeLanguageBlock {
                id: Uuid::new_v4().to_string(),
                name: language.name.clone(),
                level: language.level.clone(),
            })
            .collect(),
    };

    validate_document(&document)?;
    let score = score_resume_imported(&to_generated_resume(&document), &job_offer);
    let mut workspace = ResumeWorkspace {
        schema_version: RESUME_WORKSPACE_VERSION,
        document,
        job_offer,
        analysis,
        initial_score: score.total,
        score,
        proposals: Vec::new(),
    };
    workspace.proposals = build_proposals(&workspace);
    Ok(workspace)
}

/// Valide toutes les données éditables avant leur utilisation par l'export ou le score.
///
/// # Errors
/// Retourne une validation en français pour une valeur vide ou hors borne.
pub fn validate_document(document: &ResumeDocument) -> AppResult<()> {
    require_text(&document.identity.full_name, "Le nom complet du CV")?;
    validate_text(&document.identity.full_name, "Le nom complet du CV")?;
    for (value, label) in [
        (&document.identity.title, "Le titre du CV"),
        (&document.identity.email, "L'adresse e-mail du CV"),
        (&document.profile, "Le profil du CV"),
    ] {
        validate_text(value, label)?;
    }
    for (value, label) in [
        (document.identity.headline.as_deref(), "L'accroche du CV"),
        (document.identity.city.as_deref(), "La ville du CV"),
        (document.identity.phone.as_deref(), "Le téléphone du CV"),
        (document.identity.website.as_deref(), "Le site web du CV"),
        (
            document.identity.linkedin.as_deref(),
            "Le profil LinkedIn du CV",
        ),
        (
            document.identity.github.as_deref(),
            "Le profil GitHub du CV",
        ),
    ] {
        validate_optional_text(value, label)?;
    }
    validate_list(document.identity.extra.len(), "L'identité du CV")?;
    validate_strings(
        &document.identity.extra,
        "Une information complémentaire du CV",
    )?;

    for (len, label) in [
        (document.experiences.len(), "Le CV"),
        (document.projects.len(), "Le CV"),
        (document.skill_groups.len(), "Le CV"),
        (document.education.len(), "Le CV"),
        (document.certifications.len(), "Le CV"),
        (document.languages.len(), "Le CV"),
    ] {
        validate_list(len, label)?;
    }
    for experience in &document.experiences {
        validate_required_fields(&[
            (&experience.id, "L'identifiant d'une expérience du CV"),
            (&experience.title, "Le titre d'une expérience du CV"),
            (&experience.company, "L'entreprise d'une expérience du CV"),
        ])?;
        validate_optional_text(
            experience.location.as_deref(),
            "Le lieu d'une expérience du CV",
        )?;
        validate_text(&experience.period, "La période d'une expérience du CV")?;
        validate_list(experience.bullets.len(), "Une expérience du CV")?;
        validate_strings(&experience.bullets, "Une puce du CV")?;
    }
    for project in &document.projects {
        validate_required_fields(&[
            (&project.id, "L'identifiant d'un projet du CV"),
            (&project.name, "Le nom d'un projet du CV"),
        ])?;
        validate_optional_text(project.meta.as_deref(), "Les détails d'un projet du CV")?;
        validate_optional_text(project.url.as_deref(), "Le lien d'un projet du CV")?;
        validate_list(project.bullets.len(), "Un projet du CV")?;
        validate_strings(&project.bullets, "Une puce du CV")?;
    }
    for group in &document.skill_groups {
        validate_required_fields(&[
            (&group.id, "L'identifiant d'un groupe de compétences"),
            (&group.name, "Le nom d'un groupe de compétences"),
        ])?;
        validate_list(group.items.len(), "Un groupe de compétences")?;
        validate_non_empty_strings(&group.items, "Une compétence du CV")?;
    }
    let skill_count = document
        .skill_groups
        .iter()
        .map(|group| group.items.len())
        .sum::<usize>();
    if skill_count > MAX_ITEMS {
        return Err(AppError::Validation(
            "Le CV contient trop de compétences.".into(),
        ));
    }
    for education in &document.education {
        validate_required_fields(&[
            (&education.id, "L'identifiant d'une formation du CV"),
            (&education.degree, "Le diplôme d'une formation du CV"),
            (&education.school, "L'établissement d'une formation du CV"),
        ])?;
        validate_optional_text(
            education.location.as_deref(),
            "Le lieu d'une formation du CV",
        )?;
        validate_text(&education.period, "La période d'une formation du CV")?;
        validate_optional_text(
            education.description.as_deref(),
            "La description d'une formation du CV",
        )?;
    }
    for certification in &document.certifications {
        validate_required_fields(&[
            (&certification.id, "L'identifiant d'une certification du CV"),
            (&certification.name, "Le nom d'une certification du CV"),
        ])?;
        validate_optional_text(
            certification.issuer.as_deref(),
            "L'organisme d'une certification du CV",
        )?;
        validate_optional_text(
            certification.date.as_deref(),
            "La date d'une certification du CV",
        )?;
    }
    for language in &document.languages {
        validate_required_fields(&[
            (&language.id, "L'identifiant d'une langue du CV"),
            (&language.name, "Le nom d'une langue du CV"),
            (&language.level, "Le niveau d'une langue du CV"),
        ])?;
    }
    Ok(())
}

#[must_use]
pub fn to_generated_resume(document: &ResumeDocument) -> GeneratedResume {
    GeneratedResume {
        resume: document.profile.clone(),
        experiences: document
            .experiences
            .iter()
            .map(|experience| GeneratedExperience {
                title: experience.title.clone(),
                company: experience.company.clone(),
                description: experience.bullets.join("\n"),
            })
            .collect(),
        skills: document
            .skill_groups
            .iter()
            .flat_map(|group| group.items.iter().cloned())
            .collect(),
        education: document
            .education
            .iter()
            .map(|education| GeneratedEducation {
                degree: education.degree.clone(),
                school: education.school.clone(),
            })
            .collect(),
    }
}

/// Construit les propositions applicables au document courant : une par compétence de
/// l'offre encore absente du CV, une par recommandation IA dont la cible est identifiable.
/// Le gain de chacune est simulé sur une copie du document, jamais déclaré par le LLM.
#[must_use]
pub fn build_proposals(workspace: &ResumeWorkspace) -> Vec<ResumeProposal> {
    let mut proposals: Vec<ResumeProposal> = workspace
        .score
        .missing
        .iter()
        .map(|skill| missing_skill_proposal(workspace, skill))
        .collect();
    proposals.extend(
        workspace
            .analysis
            .recommendations
            .iter()
            .enumerate()
            .filter_map(|(index, recommendation)| {
                text_replacement_proposal(workspace, index, recommendation)
            }),
    );
    proposals
}

/// Recalcule le score courant puis reconstruit les propositions : leur applicabilité et leur
/// gain sont rafraîchis sur le document actuel, mais une proposition déjà acceptée ou refusée
/// conserve son identifiant et son statut, qu'elle soit encore générée ou non.
///
/// # Errors
/// Retourne une validation si le document dépasse les bornes d'édition.
pub fn recalculate(mut workspace: ResumeWorkspace) -> AppResult<ResumeWorkspace> {
    validate_document(&workspace.document)?;
    workspace.score = score_resume_imported(
        &to_generated_resume(&workspace.document),
        &workspace.job_offer,
    );
    let previous = std::mem::take(&mut workspace.proposals);
    let mut proposals = build_proposals(&workspace);
    for proposal in &mut proposals {
        if let Some(existing) = previous.iter().find(|p| p.id == proposal.id) {
            if existing.status != ResumeProposalStatus::Pending {
                proposal.status = existing.status;
            }
        }
    }
    for stale in previous
        .into_iter()
        .filter(|proposal| proposal.status != ResumeProposalStatus::Pending)
    {
        if !proposals.iter().any(|proposal| proposal.id == stale.id) {
            proposals.push(refresh_stale_proposal(&workspace, stale));
        }
    }
    workspace.proposals = proposals;
    Ok(workspace)
}

/// Applique une proposition au document puis recalcule l'ensemble du poste de travail.
///
/// # Errors
/// Retourne une validation si l'identifiant est inconnu, si la cible ne correspond plus au
/// texte d'origine de la proposition, ou si le document résultant dépasse les bornes d'édition.
pub fn apply_proposal(
    mut workspace: ResumeWorkspace,
    proposal_id: &str,
) -> AppResult<ResumeWorkspace> {
    let index = workspace
        .proposals
        .iter()
        .position(|proposal| proposal.id == proposal_id)
        .ok_or_else(|| AppError::Validation("Cette proposition n'existe plus.".into()))?;
    let proposal = workspace.proposals[index].clone();
    if !is_applicable(&workspace.document, &proposal) {
        return Err(AppError::Validation(
            "Cette proposition ne correspond plus au CV actuel.".into(),
        ));
    }
    apply_change(&mut workspace.document, &proposal);
    validate_document(&workspace.document)?;
    workspace.proposals[index].status = ResumeProposalStatus::Accepted;
    recalculate(workspace)
}

/// Refuse une proposition sans modifier le document, puis recalcule le poste de travail.
///
/// # Errors
/// Retourne une validation si l'identifiant est inconnu ou si le document dépasse les bornes
/// d'édition.
pub fn reject_proposal(
    mut workspace: ResumeWorkspace,
    proposal_id: &str,
) -> AppResult<ResumeWorkspace> {
    let proposal = workspace
        .proposals
        .iter_mut()
        .find(|proposal| proposal.id == proposal_id)
        .ok_or_else(|| AppError::Validation("Cette proposition n'existe plus.".into()))?;
    proposal.status = ResumeProposalStatus::Rejected;
    recalculate(workspace)
}

fn missing_skill_proposal(workspace: &ResumeWorkspace, skill: &str) -> ResumeProposal {
    let group_id = workspace.document.skill_groups.first().map_or_else(
        || DEFAULT_SKILL_GROUP_ID.to_owned(),
        |group| group.id.clone(),
    );
    let mut proposal = ResumeProposal {
        id: format!("skill-{}", search_key(skill)),
        kind: ResumeProposalKind::MissingSkill,
        target: ResumeProposalTarget::SkillGroup { group_id },
        label: format!("Ajouter la compétence « {skill} » attendue par l'offre"),
        original_text: None,
        proposed_text: skill.to_owned(),
        gain: 0,
        status: ResumeProposalStatus::Pending,
        applicable: true,
    };
    proposal.gain = simulate_gain(workspace, &proposal);
    proposal.applicable = is_applicable(&workspace.document, &proposal);
    proposal
}

/// `None` quand la cible de la recommandation n'existe plus dans le document (section
/// `Experience` sans `item_index` valide) : une telle recommandation ne peut être ni simulée
/// ni appliquée, elle ne doit donc jamais devenir une proposition.
fn text_replacement_proposal(
    workspace: &ResumeWorkspace,
    index: usize,
    recommendation: &AtsRecommendation,
) -> Option<ResumeProposal> {
    let target = match recommendation.section {
        AtsRecommendationSection::Profile => ResumeProposalTarget::Profile,
        AtsRecommendationSection::Experience => {
            let experience_id = workspace
                .document
                .experiences
                .get(recommendation.item_index?)?
                .id
                .clone();
            ResumeProposalTarget::ExperienceDescription { experience_id }
        }
    };
    let mut proposal = ResumeProposal {
        id: format!("ats-{index}"),
        kind: ResumeProposalKind::TextReplacement,
        target,
        label: text_replacement_label(recommendation.section),
        original_text: Some(recommendation.original_text.clone()),
        proposed_text: recommendation.proposed_text.clone(),
        gain: 0,
        status: ResumeProposalStatus::Pending,
        applicable: true,
    };
    proposal.gain = simulate_gain(workspace, &proposal);
    proposal.applicable = is_applicable(&workspace.document, &proposal);
    Some(proposal)
}

fn text_replacement_label(section: AtsRecommendationSection) -> String {
    match section {
        AtsRecommendationSection::Profile => "Reformuler le profil".into(),
        AtsRecommendationSection::Experience => "Reformuler une expérience".into(),
    }
}

/// Recalcule l'applicabilité et le gain d'une proposition déjà acceptée ou refusée que
/// `build_proposals` ne régénère plus (compétence désormais présente, par exemple).
fn refresh_stale_proposal(
    workspace: &ResumeWorkspace,
    mut proposal: ResumeProposal,
) -> ResumeProposal {
    proposal.applicable = is_applicable(&workspace.document, &proposal);
    proposal.gain = simulate_gain(workspace, &proposal);
    proposal
}

/// Une proposition n'est applicable que si sa cible existe encore et correspond toujours à
/// son texte d'origine (recommandation) ou n'est pas déjà satisfaite (compétence manquante).
/// Recalculé à partir du document courant, jamais depuis un indicateur mis en cache : une
/// modification manuelle du document entre deux recalculs ne doit jamais laisser passer une
/// proposition périmée.
fn is_applicable(document: &ResumeDocument, proposal: &ResumeProposal) -> bool {
    match proposal.kind {
        ResumeProposalKind::MissingSkill => !skill_present(document, &proposal.proposed_text),
        ResumeProposalKind::TextReplacement => {
            locate_text(document, &proposal.target).as_deref() == proposal.original_text.as_deref()
        }
    }
}

/// Simule l'application d'une proposition sur une copie du document et retourne l'écart de
/// score obtenu, jamais un impact déclaré par le LLM.
fn simulate_gain(workspace: &ResumeWorkspace, proposal: &ResumeProposal) -> i16 {
    let mut document = workspace.document.clone();
    apply_change(&mut document, proposal);
    let after = score_resume_imported(&to_generated_resume(&document), &workspace.job_offer).total;
    i16::from(after) - i16::from(workspace.score.total)
}

fn apply_change(document: &mut ResumeDocument, proposal: &ResumeProposal) {
    match &proposal.target {
        ResumeProposalTarget::Profile => document.profile = proposal.proposed_text.clone(),
        ResumeProposalTarget::ExperienceDescription { experience_id } => {
            if let Some(experience) = document
                .experiences
                .iter_mut()
                .find(|experience| &experience.id == experience_id)
            {
                experience.bullets = split_bullets(&proposal.proposed_text);
            }
        }
        ResumeProposalTarget::SkillGroup { group_id } => {
            if skill_present(document, &proposal.proposed_text) {
                return;
            }
            match document
                .skill_groups
                .iter_mut()
                .find(|group| &group.id == group_id)
            {
                Some(group) => group.items.push(proposal.proposed_text.clone()),
                None => document.skill_groups.push(ResumeSkillGroup {
                    id: group_id.clone(),
                    name: "Compétences".into(),
                    items: vec![proposal.proposed_text.clone()],
                }),
            }
        }
    }
}

fn locate_text(document: &ResumeDocument, target: &ResumeProposalTarget) -> Option<String> {
    match target {
        ResumeProposalTarget::Profile => Some(document.profile.clone()),
        ResumeProposalTarget::ExperienceDescription { experience_id } => document
            .experiences
            .iter()
            .find(|experience| &experience.id == experience_id)
            .map(|experience| experience.bullets.join("\n")),
        ResumeProposalTarget::SkillGroup { .. } => None,
    }
}

fn skill_present(document: &ResumeDocument, skill: &str) -> bool {
    let key = search_key(skill);
    document
        .skill_groups
        .iter()
        .any(|group| group.items.iter().any(|item| search_key(item) == key))
}

fn format_workspace_period(start: Option<&str>, end: Option<&str>, current: bool) -> String {
    let start = start
        .filter(|value| !value.trim().is_empty())
        .map(format_month_date);
    let end = if current {
        Some("Aujourd’hui".into())
    } else {
        end.filter(|value| !value.trim().is_empty())
            .map(format_month_date)
    };
    match (start, end) {
        (Some(start), Some(end)) => format!("{start} — {end}"),
        (Some(value), None) | (None, Some(value)) => value,
        (None, None) => String::new(),
    }
}

fn trimmed_option(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn validate_required_fields(fields: &[(&String, &str)]) -> AppResult<()> {
    for (value, label) in fields {
        require_text(value, label)?;
        validate_text(value, label)?;
    }
    Ok(())
}

fn validate_non_empty_strings(values: &[String], label: &str) -> AppResult<()> {
    for value in values {
        require_text(value, label)?;
        validate_text(value, label)?;
    }
    Ok(())
}

fn validate_strings(values: &[String], label: &str) -> AppResult<()> {
    for value in values {
        validate_text(value, label)?;
    }
    Ok(())
}

fn require_text(value: &str, label: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::Validation(format!("{label} est obligatoire.")))
    } else {
        Ok(())
    }
}

fn validate_optional_text(value: Option<&str>, label: &str) -> AppResult<()> {
    value.map_or(Ok(()), |value| validate_text(value, label))
}

fn validate_text(value: &str, label: &str) -> AppResult<()> {
    if value.chars().count() > MAX_ITEM_CHARS {
        Err(AppError::Validation(format!(
            "{label} dépasse la taille maximale autorisée."
        )))
    } else {
        Ok(())
    }
}

fn validate_list(len: usize, label: &str) -> AppResult<()> {
    if len > MAX_ITEMS {
        Err(AppError::Validation(format!(
            "{label} contient trop d'éléments."
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/cv_workspace/mod.rs"]
mod cv_workspace;
