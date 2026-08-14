//! Regroupement des propositions extraites d'un CV.

use crate::shared::profile::Profile;
use crate::ui::components::icon::Icon;

pub(super) struct ImportProposal {
    pub(super) label: String,
    pub(super) value: String,
    pub(super) meta: Option<String>,
    pub(super) key: String,
}

pub(super) struct ImportGroup {
    pub(super) title: &'static str,
    pub(super) kind: Icon,
    pub(super) items: Vec<ImportProposal>,
}

pub(super) fn import_groups(profile: &Profile) -> Vec<ImportGroup> {
    let mut groups = Vec::new();
    let personal = [
        (
            "first_name",
            "Prénom",
            Some(profile.personal.first_name.as_str()),
        ),
        (
            "last_name",
            "Nom",
            Some(profile.personal.last_name.as_str()),
        ),
        ("email", "E-mail", Some(profile.personal.email.as_str())),
        ("phone", "Téléphone", profile.personal.phone.as_deref()),
        ("city", "Ville", profile.personal.city.as_deref()),
        (
            "headline",
            "Titre professionnel",
            profile.personal.headline.as_deref(),
        ),
        ("summary", "Résumé", profile.personal.summary.as_deref()),
        ("linkedin", "LinkedIn", profile.personal.linkedin.as_deref()),
        ("github", "GitHub", profile.personal.github.as_deref()),
        (
            "website",
            "Site / portfolio",
            profile.personal.website.as_deref(),
        ),
    ]
    .into_iter()
    .filter_map(|(field, label, value)| {
        value
            .filter(|value| !value.trim().is_empty())
            .map(|value| ImportProposal {
                label: label.into(),
                value: value.into(),
                meta: None,
                key: format!("personal.{field}:0"),
            })
    })
    .collect::<Vec<_>>();
    push_import_group(&mut groups, "Coordonnées & liens", Icon::Profile, personal);

    push_import_group(
        &mut groups,
        "Expériences",
        Icon::Building,
        profile
            .experiences
            .iter()
            .enumerate()
            .map(|(index, item)| ImportProposal {
                label: item.company.clone(),
                value: item.title.clone(),
                meta: Some(format_import_period(
                    &item.start_date,
                    item.end_date.as_deref(),
                    item.current,
                )),
                key: crate::app::profile_edit::import_item_key("experiences", index),
            })
            .collect(),
    );
    push_import_group(
        &mut groups,
        "Compétences",
        Icon::Sparkles,
        simple_import_items(
            "skills",
            profile.skills.iter().map(|item| item.name.clone()),
        ),
    );
    push_import_group(
        &mut groups,
        "Formations",
        Icon::Document,
        profile
            .education
            .iter()
            .enumerate()
            .map(|(index, item)| ImportProposal {
                label: item.school.clone(),
                value: item.degree.clone(),
                meta: None,
                key: crate::app::profile_edit::import_item_key("education", index),
            })
            .collect(),
    );
    push_import_group(
        &mut groups,
        "Langues",
        Icon::Network,
        profile
            .languages
            .iter()
            .enumerate()
            .map(|(index, item)| ImportProposal {
                label: item.level.clone(),
                value: item.name.clone(),
                meta: None,
                key: crate::app::profile_edit::import_item_key("languages", index),
            })
            .collect(),
    );
    push_import_group(
        &mut groups,
        "Projets",
        Icon::Document,
        simple_import_items(
            "projects",
            profile.projects.iter().map(|item| item.name.clone()),
        ),
    );
    push_import_group(
        &mut groups,
        "Certifications",
        Icon::Check,
        simple_import_items(
            "certifications",
            profile.certifications.iter().map(|item| item.name.clone()),
        ),
    );
    groups
}

fn simple_import_items(
    category: &'static str,
    values: impl Iterator<Item = String>,
) -> Vec<ImportProposal> {
    values
        .enumerate()
        .map(|(index, value)| ImportProposal {
            label: String::new(),
            value,
            meta: None,
            key: crate::app::profile_edit::import_item_key(category, index),
        })
        .collect()
}

fn push_import_group(
    groups: &mut Vec<ImportGroup>,
    title: &'static str,
    kind: Icon,
    items: Vec<ImportProposal>,
) {
    if !items.is_empty() {
        groups.push(ImportGroup { title, kind, items });
    }
}

fn format_import_period(start: &str, end: Option<&str>, current: bool) -> String {
    let end = if current {
        "Aujourd’hui"
    } else {
        end.unwrap_or("Date de fin non précisée")
    };
    if start.trim().is_empty() {
        end.to_owned()
    } else {
        format!("{start} — {end}")
    }
}
