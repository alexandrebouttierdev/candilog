use crate::core::utils::validation::is_valid_email;

#[test]
fn test_is_valid_email_refuse_un_domaine_sans_point() {
    assert!(!is_valid_email("camille@localhost"));
}
