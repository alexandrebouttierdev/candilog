//! Cas de test isolé.

use super::*;

#[test]
fn le_compteur_est_global_et_ne_depend_pas_de_la_selection() {
    // Le défaut observé : la tuile affichait 0 à l'arrivée sur l'écran, alors que la
    // liste juste en dessous totalisait 37 candidatures liées.
    let premiere = Uuid::new_v4();
    let seconde = Uuid::new_v4();
    let candidates = vec![
        candidature(premiere),
        candidature(premiere),
        candidature(seconde),
    ];
    let companies = vec![entreprise(premiere), entreprise(seconde)];
    assert_eq!(total_candidatures(&candidates, &companies), 3);
}
