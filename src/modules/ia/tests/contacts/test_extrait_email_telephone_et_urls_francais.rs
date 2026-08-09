//! Cas de test isolé.

use super::*;

#[test]
fn test_extrait_email_telephone_et_urls_francais() {
    let cv = "Ada Lovelace\nada.lovelace@example.com — +33 6 12 34 56 78\n\
                  https://www.linkedin.com/in/ada-lovelace\ngithub.com/ada\nSite : https://ada.dev";
    let c = extract_contacts(cv);
    assert_eq!(c.email.as_deref(), Some("ada.lovelace@example.com"));
    assert_eq!(c.phone.as_deref(), Some("+33 6 12 34 56 78"));
    assert_eq!(
        c.linkedin.as_deref(),
        Some("https://www.linkedin.com/in/ada-lovelace")
    );
    assert_eq!(c.github.as_deref(), Some("github.com/ada"));
    assert_eq!(c.website.as_deref(), Some("https://ada.dev"));
}
