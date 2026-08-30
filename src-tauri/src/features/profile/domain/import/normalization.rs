use super::super::{Certification, Education, Experience, Language, Project, Skill};

pub(super) fn empty_to_none(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

pub(super) fn same_text(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

pub(super) fn experience_key(item: &Experience) -> String {
    normalize(&format!(
        "{}|{}|{}",
        item.title, item.company, item.start_date
    ))
}

pub(super) fn skill_key(item: &Skill) -> String {
    normalize(&item.name)
}

pub(super) fn education_key(item: &Education) -> String {
    normalize(&format!("{}|{}", item.degree, item.school))
}

pub(super) fn language_key(item: &Language) -> String {
    normalize(&item.name)
}

pub(super) fn project_key(item: &Project) -> String {
    normalize(&item.name)
}

pub(super) fn certification_key(item: &Certification) -> String {
    normalize(&item.name)
}

pub(super) fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}
