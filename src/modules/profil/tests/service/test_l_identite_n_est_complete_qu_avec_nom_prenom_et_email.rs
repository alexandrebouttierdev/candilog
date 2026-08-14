use super::*;

#[test]
fn l_identite_n_est_complete_qu_avec_nom_prenom_et_email() {
    let mut profile = profile_avec_sections(0);
    profile.personal.email = "alice@dupont.fr".into();
    assert_eq!(completion_score(&profile), 0);

    profile.personal.first_name = "Alice".into();
    assert_eq!(completion_score(&profile), 0);

    profile.personal.last_name = "Dupont".into();
    assert_eq!(completion_score(&profile), 14);
}
