//! Deux catalogues, deux usages : le secteur qualifie l'entreprise, le domaine le poste.

use super::*;

#[test]
fn les_secteurs_sont_semes_avec_des_identifiants_stables() {
    let sectors = repo().load().unwrap().sectors;

    assert_eq!(sectors.len(), 23);
    assert_eq!(
        sectors.first().map(|item| item.name.as_str()),
        Some("Achats / Comptabilité / Gestion")
    );
    // Les identifiants sont figés par `init_schema.sql` : une sauvegarde reste donc
    // lisible sur une autre installation.
    let informatique = sectors
        .iter()
        .find(|item| item.name == "Informatique / Télécommunication")
        .unwrap();
    assert_eq!(
        informatique.id.to_string(),
        "5ec70000-0000-4000-8000-00000000000d"
    );
}

#[test]
fn les_deux_catalogues_ne_partagent_aucune_cle() {
    let referentials = repo().load().unwrap();

    // Un secteur est identifié par un UUID, un domaine par son code métier : rien ne
    // permet d'utiliser l'un là où l'autre est attendu.
    let sector_ids: Vec<String> = referentials
        .sectors
        .iter()
        .map(|item| item.id.to_string())
        .collect();
    for domain in &referentials.professional_domains {
        assert!(
            !sector_ids.contains(&domain.code),
            "le domaine {} porte une clé de secteur",
            domain.code
        );
    }
}
