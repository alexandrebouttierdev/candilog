//! Cas de test isolé.

use super::*;

#[test]
fn les_deux_natures_d_evenement_sont_distinguables() {
    assert_ne!(EventKind::Interview.tone(), EventKind::Reminder.tone());
    assert_ne!(EventKind::Interview.label(), EventKind::Reminder.label());
    assert_eq!(EventKind::Interview.tone(), Tone::Success);
    assert_eq!(EventKind::Reminder.tone(), Tone::Warning);
}
