//! Cas de test isolé.

use super::*;

#[test]
fn pose_l_identite_et_l_objet() {
    let pdf = build_cover_letter(&profile(), &cover_letter());
    assert_eq!(pdf.first_name, "Alex");
    assert_eq!(pdf.last_name, "Exemple");
    assert_eq!(pdf.email, "alex@exemple.fr");
    assert_eq!(pdf.company.as_deref(), Some("Nova"));
    assert_eq!(pdf.job_title.as_deref(), Some("Développeur"));
    assert_eq!(pdf.recipient.as_deref(), Some("Service recrutement"));
    assert_eq!(pdf.job_reference.as_deref(), Some("FS-2026-114"));
}
