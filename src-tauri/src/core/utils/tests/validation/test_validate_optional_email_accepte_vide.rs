use crate::core::utils::validation::validate_optional_email;

#[test]
fn test_validate_optional_email_accepte_vide() {
    assert!(validate_optional_email(None, "L'e-mail").is_ok());
    assert!(validate_optional_email(Some("  "), "L'e-mail").is_ok());
}
