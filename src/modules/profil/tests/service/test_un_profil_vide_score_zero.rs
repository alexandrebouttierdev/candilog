use super::*;

#[test]
fn un_profil_vide_score_zero() {
    assert_eq!(completion_score(&Profile::default()), 0);
}
