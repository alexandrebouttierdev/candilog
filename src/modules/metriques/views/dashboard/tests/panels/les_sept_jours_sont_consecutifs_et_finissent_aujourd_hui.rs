//! Cas de test isolé.

use super::*;

#[test]
fn les_sept_jours_sont_consecutifs_et_finissent_aujourd_hui() {
    let days = last_seven_days(&[]);
    assert_eq!(days.len(), 7);
    for pair in days.windows(2) {
        assert_eq!(
            pair[0].0.succ_opt(),
            Some(pair[1].0),
            "jours non consécutifs"
        );
    }
    assert_eq!(days.last().unwrap().0, chrono::Local::now().date_naive());
}
