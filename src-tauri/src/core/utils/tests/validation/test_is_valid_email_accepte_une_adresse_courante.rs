use crate::core::utils::validation::is_valid_email;

#[test]
fn test_is_valid_email_accepte_une_adresse_courante() {
    assert!(is_valid_email("camille@example.fr"));
}
