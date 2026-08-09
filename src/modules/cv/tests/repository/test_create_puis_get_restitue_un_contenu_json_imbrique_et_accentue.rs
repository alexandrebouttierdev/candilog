//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_get_restitue_un_contenu_json_imbrique_et_accentue() {
    let repo = repo();
    let contenu = serde_json::json!({
        "personal": {"nom": "Béatrice Éloïse", "ville": "Montréal"},
        "experiences": [
            {"poste": "Développeuse", "entreprise": "Société Générale", "notes": "Œuvré sur l'API"},
            {"poste": "Chargée d'études", "entreprise": "L'Étude & Cie", "notes": "Rédaction de synthèses"}
        ],
        "skills": ["Rust", "Résilience", "Créativité"],
        "score": 12.5,
        "actif": true,
        "commentaire": null
    });
    let creee = repo.create("CV Complet", &contenu).unwrap();
    let relue = repo.get(creee.id).unwrap();
    assert_eq!(relue.content, contenu);
}
