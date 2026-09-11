//! Lettre de motivation assemblée uniquement depuis un catalogue de faits vérifiés.

use super::normalization::contains_search_term;
use super::{search_key, CoverLetterRequest, ValidateAiOutput, MAX_ITEMS, MAX_ITEM_CHARS};
use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::elider;
use crate::features::profile::domain::Profile;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundedFactKind {
    Summary,
    Experience,
    Skill,
    Education,
    Project,
    Certification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroundedFact {
    pub id: String,
    pub kind: GroundedFactKind,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverLetterPlan {
    #[serde(default, alias = "fact_ids", alias = "selectedFactIds")]
    pub selected_fact_ids: Vec<String>,
    #[serde(default, alias = "motivationKeywords")]
    pub motivation_keywords: Vec<String>,
}

impl ValidateAiOutput for CoverLetterPlan {
    fn validate_ai_output(&self) -> AppResult<()> {
        if self.selected_fact_ids.len() > MAX_ITEMS || self.motivation_keywords.len() > MAX_ITEMS {
            return Err(AppError::Provider(
                "La sélection de faits proposée par l'IA est trop longue.".into(),
            ));
        }
        if self
            .selected_fact_ids
            .iter()
            .chain(&self.motivation_keywords)
            .any(|value| value.chars().count() > MAX_ITEM_CHARS)
        {
            return Err(AppError::Provider(
                "La sélection de faits proposée par l'IA contient un champ trop long.".into(),
            ));
        }
        super::validate_structured_size(self)
    }
}

#[must_use]
pub fn build_fact_catalog(profile: &Profile) -> Vec<GroundedFact> {
    let mut facts = Vec::new();
    if let Some(summary) = profile
        .identity
        .resume
        .as_deref()
        .filter(|summary| !summary.trim().is_empty())
    {
        facts.push(GroundedFact {
            id: "summary:0".into(),
            kind: GroundedFactKind::Summary,
            text: summary.trim().to_owned(),
        });
    }
    facts.extend(
        profile
            .experiences
            .iter()
            .enumerate()
            .map(|(index, experience)| GroundedFact {
                id: format!("experience:{index}"),
                kind: GroundedFactKind::Experience,
                text: match experience
                    .description
                    .as_deref()
                    .filter(|text| !text.trim().is_empty())
                {
                    Some(description) => format!(
                        "{} chez {} : {}",
                        experience.title.trim(),
                        experience.company.trim(),
                        description.trim()
                    ),
                    None => format!(
                        "{} chez {}",
                        experience.title.trim(),
                        experience.company.trim()
                    ),
                },
            }),
    );
    facts.extend(
        profile
            .skills
            .iter()
            .enumerate()
            .filter(|(_, skill)| !skill.name.trim().is_empty())
            .map(|(index, skill)| GroundedFact {
                id: format!("skill:{index}"),
                kind: GroundedFactKind::Skill,
                text: skill.name.trim().to_owned(),
            }),
    );
    facts.extend(
        profile
            .education
            .iter()
            .enumerate()
            .map(|(index, education)| GroundedFact {
                id: format!("education:{index}"),
                kind: GroundedFactKind::Education,
                text: format!("{} à {}", education.degree.trim(), education.school.trim()),
            }),
    );
    facts.extend(profile.projects.iter().enumerate().map(|(index, project)| {
        GroundedFact {
            id: format!("project:{index}"),
            kind: GroundedFactKind::Project,
            text: match project
                .description
                .as_deref()
                .filter(|text| !text.trim().is_empty())
            {
                Some(description) => {
                    format!("{} : {}", project.name.trim(), description.trim())
                }
                None => project.name.trim().to_owned(),
            },
        }
    }));
    facts.extend(
        profile
            .certifications
            .iter()
            .enumerate()
            .map(|(index, certification)| GroundedFact {
                id: format!("certification:{index}"),
                kind: GroundedFactKind::Certification,
                text: match certification
                    .issuer
                    .as_deref()
                    .filter(|text| !text.trim().is_empty())
                {
                    Some(issuer) => {
                        format!(
                            "{} délivrée par {}",
                            certification.name.trim(),
                            issuer.trim()
                        )
                    }
                    None => certification.name.trim().to_owned(),
                },
            }),
    );
    facts.retain(|fact| !fact.text.trim().is_empty());
    facts
}

/// Assemble une lettre depuis des références vérifiées, sans prose factuelle produite par l'IA.
///
/// # Errors
/// Refuse un identifiant de fait inconnu ou une option non prise en charge. Les mots-clés
/// absents du brief sont écartés silencieusement.
pub fn render_grounded_letter(
    catalog: &[GroundedFact],
    plan: &CoverLetterPlan,
    request: &CoverLetterRequest,
) -> AppResult<String> {
    let tone = request.tone.as_deref().unwrap_or("formal");
    if !matches!(tone, "formal" | "casual" | "creative") {
        return Err(AppError::Validation(
            "Le ton de la lettre n'est pas pris en charge.".into(),
        ));
    }
    let fact_limit = match request.length.as_deref().unwrap_or("medium") {
        "short" => 1,
        "medium" => 2,
        "long" => 3,
        _ => {
            return Err(AppError::Validation(
                "La longueur de lettre demandée n'est pas prise en charge.".into(),
            ));
        }
    };
    let by_id: HashMap<&str, &GroundedFact> = catalog
        .iter()
        .map(|fact| (fact.id.as_str(), fact))
        .collect();
    let mut selected = Vec::new();
    let mut seen_ids = HashSet::new();
    for id in &plan.selected_fact_ids {
        let fact = by_id.get(id.as_str()).ok_or_else(|| {
            AppError::Provider("La réponse IA référence un fait inconnu du profil.".into())
        })?;
        if seen_ids.insert(id.as_str()) && selected.len() < fact_limit {
            selected.push(*fact);
        }
    }
    // Un petit modèle renvoie parfois un plan vide ou trop court : on complète alors avec
    // les faits du catalogue dans l'ordre métier, pour éviter une lettre réduite à l'ouverture.
    fill_selected_facts(&mut selected, &mut seen_ids, catalog, fact_limit);
    selected.sort_by_key(|fact| fact_kind_priority(fact.kind));

    let brief = [
        request.company.as_deref().unwrap_or_default(),
        request.job_title.as_deref().unwrap_or_default(),
        request.context.as_deref().unwrap_or_default(),
        request.instruction.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    let mut keywords = Vec::new();
    let mut seen_keywords = HashSet::new();
    for keyword in &plan.motivation_keywords {
        let key = search_key(keyword);
        // Un mot-clé reformulé par le modèle (« innovation » pour « innovant ») est écarté,
        // pas fatal : la lettre ne cite que le brief, et une paraphrase ne doit pas faire
        // échouer toute la rédaction.
        if key.is_empty() || !contains_search_term(&brief, keyword) {
            continue;
        }
        if seen_keywords.insert(key) && keywords.len() < 3 {
            keywords.push(keyword.trim());
        }
    }

    let company = request
        .company
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("votre entreprise");
    let job = request
        .job_title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("ce poste");
    let opening = match tone {
        "casual" => format!(
            "Bonjour,\n\nJe candidate au poste {} chez {}.",
            elider("de", job),
            company
        ),
        "creative" => format!(
            "Madame, Monsieur,\n\nLe poste {} au sein {} m'intéresse vivement : voici pourquoi mon parcours y répond.",
            elider("de", job),
            elider("de", company)
        ),
        _ => format!(
            "Madame, Monsieur,\n\nJe me permets de vous adresser ma candidature pour le poste {} au sein {}.",
            elider("de", job),
            elider("de", company)
        ),
    };
    let mut paragraphs = vec![opening];
    paragraphs.extend(
        selected
            .into_iter()
            .map(|fact| fact_sentence(fact, tone, job, company)),
    );
    if !keywords.is_empty() {
        paragraphs.push(match tone {
            "casual" => format!(
                "Les enjeux autour {} me parlent particulièrement et renforcent mon envie de rejoindre {}.",
                elider("de", &join_french(&keywords)),
                company
            ),
            "creative" => format!(
                "C'est surtout autour {} que je souhaite m'investir auprès de {}.",
                elider("de", &join_french(&keywords)),
                company
            ),
            _ => format!(
                "Votre besoin autour {} motive particulièrement ma candidature auprès de {}.",
                elider("de", &join_french(&keywords)),
                company
            ),
        });
    }
    paragraphs.push(match tone {
        "casual" => "Je serais ravi d'échanger pour vous présenter mon parcours plus en détail.\n\nCordialement,".into(),
        "creative" => "Je serais heureux de poursuivre cette candidature autour d'un échange concret avec votre équipe.\n\nCordialement,".into(),
        _ => "Je reste à votre disposition pour un entretien afin de détailler ma candidature.\n\nVeuillez agréer, Madame, Monsieur, l'expression de mes salutations distinguées.".into(),
    });
    Ok(paragraphs.join("\n\n"))
}

fn fact_kind_priority(kind: GroundedFactKind) -> u8 {
    match kind {
        GroundedFactKind::Experience => 0,
        GroundedFactKind::Summary => 1,
        GroundedFactKind::Skill => 2,
        GroundedFactKind::Project => 3,
        GroundedFactKind::Education => 4,
        GroundedFactKind::Certification => 5,
    }
}

fn fill_selected_facts<'a>(
    selected: &mut Vec<&'a GroundedFact>,
    seen_ids: &mut HashSet<&'a str>,
    catalog: &'a [GroundedFact],
    fact_limit: usize,
) {
    if selected.len() >= fact_limit || catalog.is_empty() {
        return;
    }
    let mut ranked: Vec<&GroundedFact> = catalog.iter().collect();
    ranked.sort_by_key(|fact| fact_kind_priority(fact.kind));
    for fact in ranked {
        if selected.len() >= fact_limit {
            break;
        }
        if seen_ids.insert(fact.id.as_str()) {
            selected.push(fact);
        }
    }
}

fn fact_sentence(fact: &GroundedFact, tone: &str, job: &str, company: &str) -> String {
    let text = phrase(&fact.text);
    match (fact.kind, tone) {
        (GroundedFactKind::Summary, "casual" | "creative") => {
            format!("En quelques mots, mon orientation : {text}")
        }
        (GroundedFactKind::Summary, _) => {
            format!("Mon projet professionnel s'inscrit dans cette direction : {text}")
        }
        (GroundedFactKind::Experience, "casual") => {
            format!("De mon côté, j'ai notamment mené {text} — une base utile pour {company}.")
        }
        (GroundedFactKind::Experience, "creative") => {
            format!(
                "Une expérience me paraît particulièrement pertinente pour le poste {} : {text}",
                elider("de", job)
            )
        }
        (GroundedFactKind::Experience, _) => {
            format!(
                "Pour le poste {} chez {}, je peux notamment m'appuyer sur l'expérience suivante : {text}",
                elider("de", job),
                company
            )
        }
        (GroundedFactKind::Skill, "casual" | "creative") => {
            format!("Je mobilise aussi couramment {text}")
        }
        (GroundedFactKind::Skill, _) => {
            format!("Parmi les compétences utiles à ce poste, je maîtrise notamment {text}")
        }
        (GroundedFactKind::Education, _) => {
            format!("Sur le plan de la formation, j'ai suivi {text}")
        }
        (GroundedFactKind::Project, "casual" | "creative") => {
            format!("J'ai également porté le projet {text}")
        }
        (GroundedFactKind::Project, _) => {
            format!("J'ai également conduit le projet {text}")
        }
        (GroundedFactKind::Certification, _) => {
            format!("Je dispose aussi de la certification {text}")
        }
    }
}

/// Ramène un fait du catalogue à une fin de phrase lisible.
///
/// Un fait reprend le texte du profil tel quel : une description d'expérience y arrive avec
/// ses retours à la ligne et sa ponctuation. Insérée telle quelle, elle coupait la phrase de
/// la lettre en plein milieu et lui ajoutait un second point (« … courantes.. »).
fn phrase(text: &str) -> String {
    let mut rendu = text
        .lines()
        .map(str::trim)
        .filter(|ligne| !ligne.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    while rendu.ends_with([';', ',', '.', ' ']) {
        rendu.pop();
    }
    rendu.push('.');
    rendu
}

fn join_french(values: &[&str]) -> String {
    match values {
        [] => String::new(),
        [only] => (*only).to_owned(),
        [first, second] => format!("{first} et {second}"),
        _ => format!(
            "{} et {}",
            values[..values.len() - 1].join(", "),
            values[values.len() - 1]
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::AppError;
    use crate::features::ai::domain::CoverLetterRequest;

    fn catalog() -> Vec<GroundedFact> {
        vec![GroundedFact {
            id: "experience:0".into(),
            kind: GroundedFactKind::Experience,
            text: "Ingénieure chez Nova — APIs Rust".into(),
        }]
    }

    fn request() -> CoverLetterRequest {
        CoverLetterRequest {
            generation_id: "test".into(),
            company: Some("Acme".into()),
            job_title: Some("Développeuse Rust".into()),
            tone: Some("formal".into()),
            length: Some("medium".into()),
            context: Some("Acme recherche une développeuse Rust pour ses APIs".into()),
            previous_cover_letter: None,
            instruction: None,
        }
    }

    #[test]
    fn une_reference_de_fait_inconnue_est_refusee() {
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:inconnue".into()],
            motivation_keywords: vec![],
        };

        assert!(matches!(
            render_grounded_letter(&catalog(), &plan, &request()),
            Err(AppError::Provider(_))
        ));
    }

    #[test]
    fn la_lettre_rendue_ne_contient_que_les_faits_du_catalogue() {
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:0".into()],
            motivation_keywords: vec!["APIs".into()],
        };

        let text = render_grounded_letter(&catalog(), &plan, &request()).unwrap();

        assert!(text.contains("Nova"));
        assert!(text.contains("APIs"));
        assert!(!text.contains("Google"));
    }

    #[test]
    fn un_mot_cle_absent_du_brief_est_ecarte_sans_faire_echouer_la_lettre() {
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:0".into()],
            motivation_keywords: vec!["Kubernetes".into(), "APIs".into()],
        };

        let text = render_grounded_letter(&catalog(), &plan, &request()).unwrap();

        assert!(!text.contains("Kubernetes"));
        assert!(text.contains("Votre besoin autour d\u{2019}APIs"));
        assert!(text.contains("auprès de Acme"));
    }

    /// Un plan vide (petit modèle local) ne doit plus produire une lettre sans faits.
    #[test]
    fn un_plan_vide_est_complete_avec_les_faits_du_catalogue() {
        let plan = CoverLetterPlan::default();
        let text = render_grounded_letter(&catalog(), &plan, &request()).unwrap();

        assert!(text.contains("Nova"), "{text}");
        assert!(
            text.contains("Pour le poste") || text.contains("De mon côté"),
            "{text}"
        );
    }

    /// « pour le poste de Administrateur », « au sein de Astek » : la lettre composait ses
    /// phrases autour de valeurs saisies sans jamais élider la préposition.
    #[test]
    fn la_lettre_elide_la_preposition_devant_une_voyelle() {
        let mut request = request();
        request.company = Some("Astek".into());
        request.job_title = Some("Administrateur syst\u{e8}me".into());
        let plan = CoverLetterPlan::default();

        let text = render_grounded_letter(&catalog(), &plan, &request).unwrap();

        assert!(
            text.contains(
                "pour le poste d\u{2019}Administrateur syst\u{e8}me au sein d\u{2019}Astek."
            ),
            "{text}"
        );
        assert!(
            text.contains("Je me permets de vous adresser ma candidature"),
            "{text}"
        );
    }

    /// Un fait reprend le texte du profil : ses retours \u{e0} la ligne coupaient la phrase en
    /// plein milieu, et sa ponctuation finale ajoutait un second point.
    #[test]
    fn un_fait_multiligne_devient_une_phrase_unique() {
        let catalog = vec![GroundedFact {
            id: "experience:0".into(),
            kind: GroundedFactKind::Experience,
            text: "Technicienne chez Nova : Support N1.\nD\u{e9}ploiement de postes.".into(),
        }];
        let plan = CoverLetterPlan {
            selected_fact_ids: vec!["experience:0".into()],
            motivation_keywords: vec![],
        };

        let text = render_grounded_letter(&catalog, &plan, &request()).unwrap();

        assert!(!text.contains(".."), "point doubl\u{e9} : {text}");
        let phrase = text
            .split("\n\n")
            .find(|bloc| bloc.contains("Technicienne chez Nova"))
            .unwrap();
        assert!(!phrase.contains('\n'), "phrase coupée : {phrase}");
        assert!(phrase.ends_with("Déploiement de postes."), "{phrase}");
        assert!(
            phrase.starts_with("Pour le poste")
                || phrase.starts_with("De mon côté")
                || phrase.starts_with("Une expérience"),
            "{phrase}"
        );
    }

    #[test]
    fn un_fragment_de_mot_n_est_pas_un_mot_cle_du_brief() {
        let mut request = request();
        request.context = Some("Un projet ongoing".into());
        let plan = CoverLetterPlan {
            selected_fact_ids: vec![],
            motivation_keywords: vec!["go".into()],
        };

        let text = render_grounded_letter(&catalog(), &plan, &request).unwrap();

        assert!(!text.contains("Votre besoin autour de"));
    }
}
