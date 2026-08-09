//! Cas de test isolé.

use super::*;

#[test]
fn test_ignore_les_fragments_trop_courts_pour_un_telephone() {
    // « 2019-2021 » n'a que 8 chiffres → pas un téléphone.
    let c = extract_contacts("Expérience 2019-2021 chez ACME");
    assert_eq!(c.phone, None);
}
