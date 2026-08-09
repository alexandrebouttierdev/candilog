//! Cas de test isolé.

use super::*;

#[test]
fn test_website_exclut_linkedin_et_github() {
    let cv = "linkedin.com/in/ada https://github.com/ada https://portfolio.ada.io";
    let c = extract_contacts(cv);
    assert_eq!(c.website.as_deref(), Some("https://portfolio.ada.io"));
}
