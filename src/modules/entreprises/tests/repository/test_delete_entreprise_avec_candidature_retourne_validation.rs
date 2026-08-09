//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_entreprise_avec_candidature_retourne_validation() {
    let repo = repo();
    let creee = repo.create(&entree("ACME")).unwrap();
    {
        let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
        conn.execute(
                "INSERT INTO candidatures (id, entreprise_id, poste, date_envoi, created_at, updated_at)
                 VALUES (?1, ?2, 'Dev', '2026-01-01', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                rusqlite::params![uuid::Uuid::new_v4().to_string(), creee.id.to_string()],
            )
            .unwrap();
    }
    // Le message est lu tel quel par l'utilisateur : on vérifie la phrase, pas seulement
    // la variante d'erreur.
    match repo.delete(creee.id) {
        Err(AppError::Validation(message)) => {
            assert_eq!(
                message,
                "Suppression impossible : des candidatures sont liées à cette entreprise"
            );
        }
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
}
