use super::ImportResolution;

pub(super) struct ListChoice<T> {
    pub(super) selected: bool,
    pub(super) value: T,
    pub(super) existing_index: Option<u32>,
    pub(super) resolution: ImportResolution,
}

pub(super) struct ListMatch<T> {
    pub(super) id: String,
    pub(super) proposed: T,
    pub(super) existing: Option<T>,
    pub(super) existing_index: Option<u32>,
    pub(super) has_conflict: bool,
}

pub(super) fn apply_list<T: Clone>(
    target: &mut Vec<T>,
    decisions: impl IntoIterator<Item = ListChoice<T>>,
    added: &mut u32,
    replaced: &mut u32,
    skipped: &mut u32,
) {
    for decision in decisions {
        if !decision.selected || decision.resolution == ImportResolution::KeepExisting {
            *skipped += 1;
            continue;
        }
        if decision.resolution == ImportResolution::Replace {
            if let Some(index) = decision.existing_index {
                if let Some(slot) = target.get_mut(index as usize) {
                    *slot = decision.value;
                    *replaced += 1;
                    continue;
                }
            }
        }
        target.push(decision.value);
        *added += 1;
    }
}

pub(super) fn list_items<T: Clone>(
    prefix: &str,
    extracted: &[T],
    current: &[T],
    key: impl Fn(&T) -> String,
) -> Vec<ListMatch<T>> {
    let mut used = vec![false; current.len()];
    extracted
        .iter()
        .enumerate()
        .map(|(index, proposed)| {
            let proposed_key = key(proposed);
            let match_index = current.iter().enumerate().find_map(|(i, item)| {
                if used[i] || key(item) != proposed_key || proposed_key.is_empty() {
                    None
                } else {
                    Some(i)
                }
            });
            if let Some(i) = match_index {
                used[i] = true;
            }
            ListMatch {
                id: format!("{prefix}-{index}"),
                proposed: proposed.clone(),
                existing: match_index.map(|i| current[i].clone()),
                existing_index: match_index.map(|i| i as u32),
                has_conflict: match_index.is_some(),
            }
        })
        .collect()
}
