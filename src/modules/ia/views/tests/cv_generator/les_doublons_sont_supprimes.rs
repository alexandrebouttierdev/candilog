//! Cas de test isolé.

use super::*;

#[test]
fn les_doublons_sont_supprimes() {
    let analysis = score(&["Rust", "Rust", "Go"], &["SQL", "SQL"]);
    assert_eq!(present_skills(&analysis), owned(&["Rust", "Go"]));
    assert_eq!(missing_skills(&analysis), owned(&["SQL"]));
}
