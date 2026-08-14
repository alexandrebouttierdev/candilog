use super::*;

#[test]
fn un_profil_complet_score_cent() {
    assert_eq!(completion_score(&profile_avec_sections(7)), 100);
}
