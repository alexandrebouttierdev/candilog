//! Accès aux contacts (base locale `SQLite`).

use crate::modules::contacts::model::{Contact, NouveauContact};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{
    connexion, maintenant_iso, traduire_contrainte, traduire_erreur, uuid_colonne, uuid_colonne_opt,
};

/// Contrat d'accès aux contacts.
pub trait ContactRepository: Send + Sync {
    /// Liste les contacts (triés par nom puis prénom).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Contact>>;
    /// Crée un contact.
    ///
    /// # Errors
    /// `AppError::Validation` si l'entreprise liée est introuvable ;
    /// `AppError::Database` si l'insertion échoue.
    fn create(&self, input: &NouveauContact) -> AppResult<Contact>;
    /// Met à jour un contact.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ;
    /// `AppError::Validation` si l'entreprise liée est introuvable.
    fn update(&self, id: uuid::Uuid, input: &NouveauContact) -> AppResult<Contact>;
    /// Supprime un contact.
    ///
    /// # Errors
    /// `AppError::Validation` si le contact est encore lié à des candidatures ou des entretiens ;
    /// `AppError::Database` si la suppression échoue.
    fn delete(&self, id: uuid::Uuid) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt de contacts.
pub struct SqliteContactRepository {
    pool: SqlitePool,
}

impl SqliteContactRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_contact`], dans l'ordre.
const COLONNES: &str =
    "id, entreprise_id, prenom, nom, poste, email, telephone, linkedin, notes, created_at, updated_at";

/// Convertit une ligne `SQLite` en contact du domaine.
fn ligne_vers_contact(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: uuid_colonne(row, 0)?,
        entreprise_id: uuid_colonne_opt(row, 1)?,
        prenom: row.get(2)?,
        nom: row.get(3)?,
        poste: row.get(4)?,
        email: row.get(5)?,
        telephone: row.get(6)?,
        linkedin: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

impl ContactRepository for SqliteContactRepository {
    fn list(&self) -> AppResult<Vec<Contact>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM contacts ORDER BY nom COLLATE NOCASE ASC, prenom COLLATE NOCASE ASC"
            ))
            .map_err(|e| traduire_erreur(e, "contacts"))?;
        let lignes = requete
            .query_map([], ligne_vers_contact)
            .map_err(|e| traduire_erreur(e, "contacts"))?;
        let mut contacts = Vec::new();
        for ligne in lignes {
            contacts.push(ligne.map_err(|e| traduire_erreur(e, "contacts"))?);
        }
        Ok(contacts)
    }

    fn create(&self, input: &NouveauContact) -> AppResult<Contact> {
        let conn = connexion(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let maintenant = maintenant_iso();
        conn.execute(
            "INSERT INTO contacts (id, entreprise_id, prenom, nom, poste, email, telephone, linkedin, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            rusqlite::params![
                id.to_string(),
                input.entreprise_id.map(|id| id.to_string()),
                input.prenom, input.nom, input.poste, input.email,
                input.telephone, input.linkedin, input.notes, maintenant
            ],
        )
        .map_err(|e| {
            traduire_contrainte(e, "L'entreprise liée à ce contact est introuvable", "contact")
        })?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM contacts WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_contact,
        )
        .map_err(|e| traduire_erreur(e, "contact"))
    }

    fn update(&self, id: uuid::Uuid, input: &NouveauContact) -> AppResult<Contact> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE contacts SET entreprise_id = ?2, prenom = ?3, nom = ?4, poste = ?5, email = ?6,
                    telephone = ?7, linkedin = ?8, notes = ?9, updated_at = ?10
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.entreprise_id.map(|id| id.to_string()),
                    input.prenom, input.nom, input.poste, input.email,
                    input.telephone, input.linkedin, input.notes, maintenant_iso()
                ],
            )
            .map_err(|e| {
                traduire_contrainte(e, "L'entreprise liée à ce contact est introuvable", "contact")
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("contact {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLONNES} FROM contacts WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_contact,
        )
        .map_err(|e| traduire_erreur(e, "contact"))
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let mut conn = connexion(&self.pool)?;
        // Le schéma détache (`ON DELETE SET NULL`) au lieu de refuser : sans ce garde-fou, une
        // candidature ou un entretien perdrait son interlocuteur en silence. On rétablit donc le
        // refus qu'appliquait `PostGREST`. Requête SQL directe : le module `contacts` n'importe
        // ni `candidatures` ni `entretiens`.
        //
        // Transaction `IMMEDIATE` : le comptage et la suppression doivent être atomiques, sinon
        // une candidature créée entre les deux passerait le garde-fou et serait détachée. Une
        // transaction différée prendrait d'abord un verrou de lecture, et son passage en écriture
        // pourrait échouer en `SQLITE_BUSY` — on prend donc le verrou d'écriture d'entrée de jeu.
        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| traduire_erreur(e, "contact"))?;
        let references: i64 = tx
            .query_row(
                "SELECT (SELECT count(*) FROM candidatures WHERE contact_id = ?1)
                      + (SELECT count(*) FROM entretiens WHERE contact_id = ?1)",
                [id.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "contact"))?;
        if references > 0 {
            return Err(AppError::Validation(
                "Suppression impossible : ce contact est lié à des candidatures ou des entretiens"
                    .to_owned(),
            ));
        }
        tx.execute("DELETE FROM contacts WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "contact"))?;
        tx.commit().map_err(|e| traduire_erreur(e, "contact"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
