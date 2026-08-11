//! Cas de test isolé.

use super::*;

/// Le seul mécanisme de suppression était `reset()`, déclenché exclusivement par l'action
/// manuelle « Vider le cache IA » des Paramètres : ni expiration, ni éviction par taille, ni
/// limite de lignes. Chaque analyse d'offre ou de CV pesant plusieurs kilo-octets, le fichier
/// de base croissait indéfiniment — et le poids était recopié à chaque export de backup.
#[test]
fn test_le_cache_est_borne_en_nombre_d_entrees() {
    let depot = repo();
    let total = MAX_ENTREES + 20;
    for index in 0..total {
        let mut entree = entry(&format!("cle-{index:05}"), "valeur");
        // Horodatage croissant : les entrées les plus anciennes sont écrites en premier, ce
        // qui rend l'ordre d'éviction observable.
        entree.cree_le = format!("2026-01-01T00:00:00.{index:05}Z");
        depot.put(&entree).unwrap();
    }

    let restantes = depot.compter().unwrap();
    assert!(
        restantes <= MAX_ENTREES,
        "le cache doit rester borné, il contient {restantes} entrées"
    );

    // L'entrée qu'on vient d'écrire doit toujours être servie : l'évincer rendrait le cache
    // inutile pour l'opération en cours.
    assert_eq!(
        depot.get(&format!("cle-{:05}", total - 1)).unwrap(),
        Some("valeur".to_string()),
        "la dernière entrée écrite a été évincée"
    );
    // Et la plus ancienne doit avoir cédé la place.
    assert_eq!(
        depot.get("cle-00000").unwrap(),
        None,
        "l'éviction doit commencer par les entrées les plus anciennes"
    );
}
