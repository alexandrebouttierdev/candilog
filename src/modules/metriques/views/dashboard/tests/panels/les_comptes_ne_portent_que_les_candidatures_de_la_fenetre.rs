//! Cas de test isolé.

use super::*;

#[test]
fn les_comptes_ne_portent_que_les_candidatures_de_la_fenetre() {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let activity = vec![(today, 2), ("2020-01-01".into(), 1)];
    let days = last_seven_days(&activity);
    assert_eq!(days.last().unwrap().1, 2, "les envois du jour");
    assert_eq!(days.first().unwrap().1, 0, "hors fenêtre");
    assert_eq!(
        days.iter().map(|(_, count)| count).sum::<usize>(),
        2,
        "aucune candidature hors fenêtre ne compte"
    );
}
