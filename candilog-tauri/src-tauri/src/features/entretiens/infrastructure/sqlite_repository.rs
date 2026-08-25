//! Dépôt `SQLite` des entretiens.

use crate::core::database::helpers::{
    connexion, enum_depuis_texte, maintenant_iso, texte_depuis_enum, traduire_erreur, uuid_colonne,
    uuid_colonne_opt,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::entretiens::domain::{
    AnalyseEntretien, Entretien, EntretienRepository, NouvelEntretien,
};
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt d'entretiens.
pub struct SqliteEntretienRepository {
    pool: SqlitePool,
}

impl SqliteEntretienRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_entretien`], dans l'ordre.
const COLONNES: &str = "e.id, e.candidature_id, c.poste, ent.nom, e.contact_id, \
                        ct.prenom || ' ' || ct.nom, e.date_entretien, e.type, e.lieu, e.notes, \
                        e.compte_rendu, e.analyse_ia, e.created_at, e.updated_at";

/// Source des colonnes : trois jointures gauches, la candidature étant la seule obligatoire
/// en base — l'entreprise et le contact peuvent manquer.
const DEPUIS: &str = "FROM entretiens e \
                      LEFT JOIN candidatures c ON c.id = e.candidature_id \
                      LEFT JOIN entreprises ent ON ent.id = c.entreprise_id \
                      LEFT JOIN contacts ct ON ct.id = e.contact_id";

/// Convertit une ligne `SQLite` en entretien du domaine.
fn ligne_vers_entretien(row: &rusqlite::Row) -> AppResult<Entretien> {
    let erreur = |e: rusqlite::Error| traduire_erreur(e, "entretien");
    let type_texte: String = row.get(7).map_err(erreur)?;
    let analyse_json: Option<String> = row.get(11).map_err(erreur)?;
    let analyse_ia = match analyse_json {
        None => None,
        Some(json) => {
            Some(serde_json::from_str(&json).map_err(|e| AppError::Serialization(e.to_string()))?)
        }
    };
    Ok(Entretien {
        id: uuid_colonne(row, 0).map_err(erreur)?,
        candidature_id: uuid_colonne(row, 1).map_err(erreur)?,
        candidature_poste: row.get(2).map_err(erreur)?,
        entreprise_nom: row.get(3).map_err(erreur)?,
        contact_id: uuid_colonne_opt(row, 4).map_err(erreur)?,
        contact_nom: row.get(5).map_err(erreur)?,
        date_entretien: row.get(6).map_err(erreur)?,
        type_entretien: enum_depuis_texte(&type_texte)?,
        lieu: row.get(8).map_err(erreur)?,
        notes: row.get(9).map_err(erreur)?,
        compte_rendu: row.get(10).map_err(erreur)?,
        analyse_ia,
        created_at: row.get(12).map_err(erreur)?,
        updated_at: row.get(13).map_err(erreur)?,
    })
}

/// Lit une requête d'entretiens jusqu'au bout.
fn collecter(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> AppResult<Vec<Entretien>> {
    let mut requete = conn
        .prepare(sql)
        .map_err(|e| traduire_erreur(e, "entretiens"))?;
    let mut lignes = requete
        .query(params)
        .map_err(|e| traduire_erreur(e, "entretiens"))?;
    let mut items = Vec::new();
    while let Some(row) = lignes
        .next()
        .map_err(|e| traduire_erreur(e, "entretiens"))?
    {
        items.push(ligne_vers_entretien(row)?);
    }
    Ok(items)
}

impl EntretienRepository for SqliteEntretienRepository {
    fn list(&self) -> AppResult<Vec<Entretien>> {
        let conn = connexion(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLONNES} {DEPUIS} ORDER BY e.date_entretien ASC"),
            &[],
        )
    }

    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Entretien>> {
        let conn = connexion(&self.pool)?;
        collecter(
            &conn,
            &format!(
                "SELECT {COLONNES} {DEPUIS} \
                 WHERE e.date_entretien >= ?1 AND e.date_entretien <= ?2 \
                 ORDER BY e.date_entretien ASC"
            ),
            &[&from, &to],
        )
    }

    fn get(&self, id: Uuid) -> AppResult<Entretien> {
        let conn = connexion(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLONNES} {DEPUIS} WHERE e.id = ?1"),
            &[&id.to_string()],
        )?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("entretien {id}")))
    }

    fn save_and_mark_candidate(
        &self,
        id: Option<Uuid>,
        input: &NouvelEntretien,
    ) -> AppResult<Entretien> {
        let mut conn = connexion(&self.pool)?;
        let entretien_id = id.unwrap_or_else(Uuid::new_v4);
        let maintenant = maintenant_iso();
        let type_entretien = texte_depuis_enum(&input.type_entretien)?;
        let transaction = conn
            .transaction()
            .map_err(|e| traduire_erreur(e, "enregistrement de l'entretien"))?;

        // Le statut de la candidature est lu **avant** l'écriture de l'entretien : c'est ce
        // qui permet de n'historiser que les passages réels à l'étape entretien. La lecture
        // vaut aussi contrôle d'existence — la clé étrangère la refuserait plus loin, mais
        // avec un message technique.
        let statut_precedent: String = transaction
            .query_row(
                "SELECT statut FROM candidatures WHERE id = ?1",
                [input.candidature_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::Validation(
                    "La candidature liée à cet entretien est introuvable".into(),
                ),
                other => traduire_erreur(other, "candidature"),
            })?;

        let parametres = rusqlite::params![
            entretien_id.to_string(),
            input.candidature_id.to_string(),
            input.contact_id.map(|value| value.to_string()),
            input.date_entretien,
            type_entretien,
            input.lieu,
            input.notes,
            input.compte_rendu,
            maintenant,
        ];
        let modifiees = if id.is_some() {
            transaction.execute(
                "UPDATE entretiens SET candidature_id = ?2, contact_id = ?3, date_entretien = ?4,
                    type = ?5, lieu = ?6, notes = ?7, compte_rendu = ?8, updated_at = ?9
                 WHERE id = ?1",
                parametres,
            )
        } else {
            transaction.execute(
                "INSERT INTO entretiens (id, candidature_id, contact_id, date_entretien, type,
                    lieu, notes, compte_rendu, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                parametres,
            )
        }
        .map_err(|e| traduire_erreur(e, "entretien"))?;

        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entretien {entretien_id}")));
        }

        // Planifier un entretien fait avancer la candidature : c'est la règle métier que
        // l'utilisateur attend, et l'oublier laisserait des candidatures « en attente »
        // alors qu'un entretien est déjà au calendrier.
        let statut_entretien = "ENTRETIEN";
        transaction
            .execute(
                "UPDATE candidatures SET statut = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![
                    input.candidature_id.to_string(),
                    statut_entretien,
                    maintenant
                ],
            )
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        if statut_precedent != statut_entretien {
            transaction
                .execute(
                    "INSERT INTO statut_history (id, candidature_id, statut, changed_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        Uuid::new_v4().to_string(),
                        input.candidature_id.to_string(),
                        statut_entretien,
                        maintenant,
                    ],
                )
                .map_err(|e| traduire_erreur(e, "historique du statut"))?;
        }

        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "enregistrement de l'entretien"))?;
        self.get(entretien_id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        // La candidature garde son statut : supprimer un entretien annulé ne veut pas dire
        // que la candidature n'a jamais atteint cette étape.
        conn.execute("DELETE FROM entretiens WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "entretien"))?;
        Ok(())
    }

    fn enregistrer_analyse(&self, id: Uuid, analyse: &AnalyseEntretien) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        let json = serde_json::to_string(analyse)?;
        let modifiees = conn
            .execute(
                "UPDATE entretiens SET analyse_ia = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id.to_string(), json, maintenant_iso()],
            )
            .map_err(|e| traduire_erreur(e, "entretien"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entretien {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
