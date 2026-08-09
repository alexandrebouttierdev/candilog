//! Cas de test isolé.

use super::*;

#[test]
fn test_normalise_les_dates_mm_slash_yyyy() {
    assert_eq!(normalize_date("06/2023").as_deref(), Some("2023-06"));
    assert_eq!(normalize_date("mars 2021").as_deref(), Some("2021-03"));
    assert_eq!(normalize_date("2019").as_deref(), Some("2019"));
    assert_eq!(
        normalize_date("depuis 2020 (en cours)").as_deref(),
        Some("Présent")
    );
    assert_eq!(normalize_date("  ").as_deref(), None);
}
