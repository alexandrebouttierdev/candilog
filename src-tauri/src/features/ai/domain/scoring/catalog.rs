//! Catalogue du profil et recadrage des recommandations du modèle sur ce catalogue.

use super::*;

/// Entrée compacte transmise au modèle : l'identifiant est la seule valeur qu'une
/// recommandation de contenu peut ensuite cibler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileContentCatalogEntry {
    pub id: String,
    pub kind: &'static str,
    pub label: String,
    pub context: String,
}

#[must_use]
pub fn profile_content_catalog(profile: &Profile) -> Vec<ProfileContentCatalogEntry> {
    let skills =
        profile
            .skills
            .iter()
            .enumerate()
            .map(|(index, item)| ProfileContentCatalogEntry {
                id: format!("skill-{index}"),
                kind: "skill",
                label: item.name.trim().to_owned(),
                context: String::new(),
            });
    let projects =
        profile
            .projects
            .iter()
            .enumerate()
            .map(|(index, item)| ProfileContentCatalogEntry {
                id: format!("project-{index}"),
                kind: "project",
                label: item.name.trim().to_owned(),
                context: format!(
                    "{} {}",
                    item.technologies.as_deref().unwrap_or_default(),
                    item.description.as_deref().unwrap_or_default()
                )
                .trim()
                .to_owned(),
            });
    let certifications = profile
        .certifications
        .iter()
        .enumerate()
        .map(|(index, item)| ProfileContentCatalogEntry {
            id: format!("certification-{index}"),
            kind: "certification",
            label: item.name.trim().to_owned(),
            context: item.issuer.as_deref().unwrap_or_default().trim().to_owned(),
        });
    let languages =
        profile
            .languages
            .iter()
            .enumerate()
            .map(|(index, item)| ProfileContentCatalogEntry {
                id: format!("language-{index}"),
                kind: "language",
                label: item.name.trim().to_owned(),
                context: item.level.trim().to_owned(),
            });

    skills
        .chain(projects)
        .chain(certifications)
        .chain(languages)
        .filter(|entry| !entry.label.is_empty())
        .collect()
}

/// Écarte les identifiants inventés, les doublons et les recommandations sans motif.
pub fn ground_content_recommendations(
    catalog: &[ProfileContentCatalogEntry],
    analysis: &mut AtsAnalysis,
) {
    let allowed: HashSet<&str> = catalog.iter().map(|entry| entry.id.as_str()).collect();
    let mut seen = HashSet::new();
    analysis.content_recommendations.retain(|recommendation| {
        allowed.contains(recommendation.item_id.as_str())
            && !recommendation.reason.trim().is_empty()
            && seen.insert(recommendation.item_id.clone())
    });
    analysis.content_recommendations.truncate(8);
}

/// Recadre les reformulations sur une cible réelle du CV et sur les exigences calculées.
///
/// Le modèle aide à rédiger, mais ne peut ni cibler un texte inexistant, ni glisser dans la
/// proposition une exigence que le moteur vient d'identifier comme absente.
pub fn ground_ats_recommendations(
    resume: &GeneratedResume,
    score: &MatchScore,
    analysis: &mut AtsAnalysis,
) {
    let resume_source = resume_text_raw(resume);
    let mut seen = HashSet::new();
    analysis.recommendations.retain_mut(|recommendation| {
        let target_text = match recommendation.section {
            crate::features::ai::domain::AtsRecommendationSection::Profile => {
                Some(resume.resume.as_str())
            }
            crate::features::ai::domain::AtsRecommendationSection::Experience => recommendation
                .item_index
                .and_then(|index| resume.experiences.get(index))
                .map(|experience| experience.description.as_str()),
        };
        let Some(target_text) = target_text else {
            return false;
        };
        if target_text != recommendation.original_text
            || search_key(&recommendation.original_text)
                == search_key(&recommendation.proposed_text)
        {
            return false;
        }
        if score.missing.iter().any(|missing| {
            contains_term(&recommendation.proposed_text, missing)
                && !contains_term(&recommendation.original_text, missing)
        }) {
            return false;
        }
        recommendation.source_evidence = deduplicate_labels(&recommendation.source_evidence)
            .into_iter()
            .filter(|evidence| contains_term(&resume_source, evidence))
            .collect();
        if recommendation.source_evidence.is_empty() {
            recommendation
                .source_evidence
                .push(recommendation.original_text.clone());
        }
        if recommendation
            .target_requirement
            .as_deref()
            .is_some_and(|target| {
                !score
                    .evaluations
                    .iter()
                    .any(|evaluation| labels_overlap(&evaluation.requirement, target))
            })
        {
            recommendation.target_requirement = None;
        }
        if recommendation.reason.trim().is_empty() {
            recommendation.reason = recommendation.target_requirement.as_deref().map_or_else(
                || {
                    "Cette reformulation rend plus visible une preuve déjà présente dans le CV."
                        .to_owned()
                },
                |target| {
                    format!(
                        "L'offre met en avant « {target} » et cette preuve existe déjà dans le CV."
                    )
                },
            );
        }
        seen.insert(search_key(&recommendation.proposed_text))
    });
    analysis.recommendations.truncate(8);
}
