//! Dépôt `SQLite` des contacts.

use crate::core::database::helpers::{
    connexion, maintenant_iso, traduire_contrainte, traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::features::contacts::domain::{Contact, ContactRepository, NouveauContact};

/// Implémentation `SQLite` du dépôt de contacts.
pub struct SqliteContactRepository {
    pool: SqlitePool,
}

impl SqliteContactRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_contact`], dans l'ordre, avec la jointure entreprise.
const COLONNES: &str = "c.id, c.entreprise_id, e.nom, c.prenom, c.nom, c.poste, c.role_suivi, \
                        c.email, c.telephone, c.linkedin, c.notes, c.created_at, c.updated_at";

/// Source des colonnes : `LEFT JOIN` et non jointure interne, un contact pouvant n'être
/// rattaché à aucune entreprise.
const SOURCE: &str = "contacts c LEFT JOIN entreprises e ON e.id = c.entreprise_id";

/// Ordre d'affichage du réseau.
const ORDRE: &str = "ORDER BY c.nom COLLATE NOCASE ASC, c.prenom COLLATE NOCASE ASC";

/// Convertit une ligne `SQLite` en contact du domaine.
fn ligne_vers_contact(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: uuid_colonne(row, 0)?,
        entreprise_id: uuid_colonne_opt(row, 1)?,
        entreprise_nom: row.get(2)?,
        prenom: row.get(3)?,
        nom: row.get(4)?,
        poste: row.get(5)?,
        role_suivi: row.get(6)?,
        email: row.get(7)?,
        telephone: row.get(8)?,
        linkedin: row.get(9)?,
        notes: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

impl ContactRepository for SqliteContactRepository {
    fn list(&self) -> AppResult<Vec<Contact>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!("SELECT {COLONNES} FROM {SOURCE} {ORDRE}"))
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

    fn get(&self, id: uuid::Uuid) -> AppResult<Contact> {
        let conn = connexion(&self.pool)?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM {SOURCE} WHERE c.id = ?1"),
            [id.to_string()],
            ligne_vers_contact,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("contact {id}")),
            other => traduire_erreur(other, "contact"),
        })
    }

    fn list_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<Contact>> {
        let conn = connexion(&self.pool)?;
        let page = page.max(1);
        let page_size = page_size.max(1);
        let needle = format!("%{}%", search.trim().to_lowercase());
        // `?1 = '%%'` court-circuite le filtre lorsque la recherche est vide : sans ce test,
        // les contacts dont le poste ou l'e-mail est NULL seraient exclus par les `LIKE`.
        let filtre = "WHERE ?1 = '%%' \
                      OR lower(c.prenom) LIKE ?1 \
                      OR lower(c.nom) LIKE ?1 \
                      OR lower(coalesce(c.poste, '')) LIKE ?1 \
                      OR lower(coalesce(c.role_suivi, '')) LIKE ?1 \
                      OR lower(coalesce(c.email, '')) LIKE ?1 \
                      OR lower(coalesce(e.nom, '')) LIKE ?1";
        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) FROM {SOURCE} {filtre}"),
                [&needle],
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "contacts"))?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM {SOURCE} {filtre} {ORDRE} LIMIT ?2 OFFSET ?3"
            ))
            .map_err(|e| traduire_erreur(e, "contacts"))?;
        let lignes = requete
            .query_map(
                rusqlite::params![needle, page_size, Page::<Contact>::offset(page, page_size)],
                ligne_vers_contact,
            )
            .map_err(|e| traduire_erreur(e, "contacts"))?;
        let mut items = Vec::new();
        for ligne in lignes {
            items.push(ligne.map_err(|e| traduire_erreur(e, "contacts"))?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn create(&self, input: &NouveauContact) -> AppResult<Contact> {
        let conn = connexion(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let maintenant = maintenant_iso();
        conn.execute(
            "INSERT INTO contacts (id, entreprise_id, prenom, nom, poste, role_suivi, email,
                telephone, linkedin, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            rusqlite::params![
                id.to_string(),
                input.entreprise_id.map(|id| id.to_string()),
                input.prenom,
                input.nom,
                input.poste,
                input.role_suivi,
                input.email,
                input.telephone,
                input.linkedin,
                input.notes,
                maintenant
            ],
        )
        .map_err(|e| {
            traduire_contrainte(
                e,
                "L'entreprise liée à ce contact est introuvable",
                "contact",
            )
        })?;
        self.get(id)
    }

    fn update(&self, id: uuid::Uuid, input: &NouveauContact) -> AppResult<Contact> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE contacts SET entreprise_id = ?2, prenom = ?3, nom = ?4, poste = ?5,
                    role_suivi = ?6, email = ?7, telephone = ?8, linkedin = ?9, notes = ?10,
                    updated_at = ?11
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.entreprise_id.map(|id| id.to_string()),
                    input.prenom,
                    input.nom,
                    input.poste,
                    input.role_suivi,
                    input.email,
                    input.telephone,
                    input.linkedin,
                    input.notes,
                    maintenant_iso()
                ],
            )
            .map_err(|e| {
                traduire_contrainte(
                    e,
                    "L'entreprise liée à ce contact est introuvable",
                    "contact",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("contact {id}")));
        }
        self.get(id)
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let mut conn = connexion(&self.pool)?;
        // Le schéma détache (`ON DELETE SET NULL`) au lieu de refuser : sans ce garde-fou, une
        // candidature ou un entretien perdrait son interlocuteur en silence. Requête SQL
        // directe : la feature `contacts` n'importe ni `candidatures` ni `entretiens`.
        //
        // Transaction `IMMEDIATE` : le comptage et la suppression doivent être atomiques, sinon
        // une candidature créée entre les deux passerait le garde-fou et serait détachée. Une
        // transaction différée prendrait d'abord un verrou de lecture, et son passage en
        // écriture pourrait échouer en `SQLITE_BUSY` — on prend donc le verrou d'écriture
        // d'entrée de jeu.
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
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
