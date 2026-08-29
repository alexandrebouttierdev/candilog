//! Cas de test isolé.

use super::*;

#[test]
fn test_validate_user_file_path_refuse_la_traversee() {
    assert!(validate_user_file_path("../../etc/passwd").is_err());
    assert!(validate_user_file_path("").is_err());
    let ok = validate_user_file_path("/home/alex/cv.pdf").unwrap();
    assert_eq!(ok, std::path::PathBuf::from("/home/alex/cv.pdf"));
}
