//! Cas de test isolé.

use super::*;

#[test]
fn la_casse_est_ignoree() {
    let analysis = score(&["Rust", "rust"], &["Go", "GO", "go"]);
    assert_eq!(present_skills(&analysis), owned(&["Rust"]));
    assert_eq!(missing_skills(&analysis), owned(&["Go"]));
}
