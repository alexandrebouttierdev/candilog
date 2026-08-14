use super::*;

#[test]
fn un_profil_a_moitie_score_proche_de_cinquante() {
    // Pondération 1/7 : les scores entiers les plus proches de 50 sont
    // 3/7 ≈ 43 et 4/7 ≈ 57, arrondis au plus proche.
    assert_eq!(completion_score(&profile_avec_sections(3)), 43);
    assert_eq!(completion_score(&profile_avec_sections(4)), 57);
}
