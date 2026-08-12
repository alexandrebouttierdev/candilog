//! Accès aux candidatures (base locale `SQLite`).

use crate::modules::candidatures::model::{Candidature, NouvelleCandidature, StatutCandidature};
use crate::modules::metriques::model::Page;
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{
    connexion, enum_depuis_texte, maintenant_iso, texte_depuis_enum, traduire_contrainte,
    traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use rusqlite::OptionalExtension;
use uuid::Uuid;

/// Critères appliqués par SQLite avant pagination.
#[derive(Debug, Clone, Default)]
pub struct CandidaturePageQuery {
    pub search: String,
    pub status: Option<StatutCandidature>,
    pub contract: Option<crate::modules::candidatures::model::TypeContrat>,
    pub company_id: Option<Uuid>,
    pub city: String,
    pub position: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort: String,
    pub descending: bool,
}

/// Agrégats globaux calculés sans charger les candidatures individuelles.
#[derive(Debug, Clone, Default)]
pub struct CandidatureStats {
    pub total: u64,
    pub pending: u64,
    pub followed_up: u64,
    pub interviews: u64,
    pub rejected: u64,
    pub linked_contacts: u64,
    /// Candidatures ayant déjà atteint l'étape entretien, même si leur statut a changé ensuite.
    pub converted_candidates: u64,
    /// Candidatures sans réponse depuis au moins sept jours.
    pub to_follow_up: u64,
    /// Comptes journaliers des 56 derniers jours, au format ISO.
    pub activity_by_day: Vec<(String, u64)>,
}

/// Contrat d'accès aux candidatures.
pub trait CandidatureRepository: Send + Sync {
    /// Crée une candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si l'entreprise liée est introuvable ; sinon `AppError::Database`.
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature>;
    /// Liste les candidatures (les plus récentes d'abord), avec le nom de l'entreprise liée.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Candidature>>;
    /// Liste une page après recherche, filtrage et tri dans SQLite.
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        query: &CandidaturePageQuery,
    ) -> AppResult<Page<Candidature>> {
        let mut items = self.list()?;
        let needle = query.search.trim().to_lowercase();
        items.retain(|item| {
            (needle.is_empty()
                || item.poste.to_lowercase().contains(&needle)
                || item
                    .entreprise_nom
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&needle))
                && query.status.is_none_or(|value| item.statut == value)
                && query
                    .contract
                    .is_none_or(|value| item.type_contrat == value)
                && query
                    .company_id
                    .is_none_or(|value| item.entreprise_id == value)
                && (query.position.trim().is_empty()
                    || item
                        .poste
                        .to_lowercase()
                        .contains(&query.position.trim().to_lowercase()))
                && query
                    .date_from
                    .as_deref()
                    .is_none_or(|value| item.date_envoi.as_str() >= value)
                && query
                    .date_to
                    .as_deref()
                    .is_none_or(|value| item.date_envoi.as_str() <= value)
        });
        let total = items.len() as u64;
        let start = page.saturating_sub(1).saturating_mul(page_size) as usize;
        Ok(Page::new(
            items
                .into_iter()
                .skip(start)
                .take(page_size as usize)
                .collect(),
            total,
            page,
            page_size,
        ))
    }
    /// Calcule les indicateurs globaux avec des agrégats SQL dans l'implémentation réelle.
    fn stats(&self) -> AppResult<CandidatureStats> {
        let items = self.list()?;
        let mut stats = CandidatureStats::default();
        for item in items {
            stats.total += 1;
            match item.statut {
                StatutCandidature::EnAttente => stats.pending += 1,
                StatutCandidature::Relancee => stats.followed_up += 1,
                StatutCandidature::Entretien => {
                    stats.interviews += 1;
                    stats.converted_candidates += 1;
                }
                StatutCandidature::Refus => stats.rejected += 1,
            }
            stats.linked_contacts += u64::from(item.contact_id.is_some());
        }
        Ok(stats)
    }
    /// Liste un nombre borné de candidatures sans réponse depuis la date limite donnée.
    fn list_to_follow_up(&self, before: &str, limit: u64) -> AppResult<Vec<Candidature>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|item| {
                item.statut == StatutCandidature::EnAttente && item.date_envoi.as_str() <= before
            })
            .take(limit as usize)
            .collect())
    }
    /// Met à jour tous les champs éditables d'une candidature.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; `AppError::Validation` si l'entreprise
    /// liée est introuvable.
    fn update(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature>;
    /// Met à jour uniquement le statut (cible du drag & drop).
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature>;
    /// Supprime une candidature par identifiant (et ses relances/entretiens en cascade).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt de candidatures.
pub struct SqliteCandidatureRepository {
    pool: SqlitePool,
}

impl SqliteCandidatureRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_candidature`], dans l'ordre (avec le nom d'entreprise joint).
const COLONNES: &str = "c.id, c.poste, c.entreprise_id, e.nom, c.contact_id, c.type_contrat,
    c.statut, c.date_envoi, c.lien_offre, c.notes, c.created_at, c.updated_at";

/// Source de la requête : jointure sur `entreprises` pour exposer `entreprise_nom`.
const DEPUIS: &str = "FROM candidatures c LEFT JOIN entreprises e ON e.id = c.entreprise_id";

/// Convertit une ligne `SQLite` (avec jointure) en candidature du domaine.
///
/// Renvoie `AppResult` (et non `rusqlite::Result`) car `type_contrat` et `statut` sont des enums
/// serde dont la conversion depuis le `TEXT` stocké peut échouer.
fn ligne_vers_candidature(row: &rusqlite::Row) -> AppResult<Candidature> {
    let type_contrat: String = row.get(5).map_err(|e| traduire_erreur(e, "candidature"))?;
    let statut: String = row.get(6).map_err(|e| traduire_erreur(e, "candidature"))?;
    Ok(Candidature {
        id: uuid_colonne(row, 0).map_err(|e| traduire_erreur(e, "candidature"))?,
        poste: row.get(1).map_err(|e| traduire_erreur(e, "candidature"))?,
        entreprise_id: uuid_colonne(row, 2).map_err(|e| traduire_erreur(e, "candidature"))?,
        entreprise_nom: row.get(3).map_err(|e| traduire_erreur(e, "candidature"))?,
        contact_id: uuid_colonne_opt(row, 4).map_err(|e| traduire_erreur(e, "candidature"))?,
        type_contrat: enum_depuis_texte(&type_contrat)?,
        statut: enum_depuis_texte(&statut)?,
        date_envoi: row.get(7).map_err(|e| traduire_erreur(e, "candidature"))?,
        lien_offre: row.get(8).map_err(|e| traduire_erreur(e, "candidature"))?,
        notes: row.get(9).map_err(|e| traduire_erreur(e, "candidature"))?,
        created_at: row.get(10).map_err(|e| traduire_erreur(e, "candidature"))?,
        updated_at: row.get(11).map_err(|e| traduire_erreur(e, "candidature"))?,
    })
}

/// Ajoute une étape à l'historique sans dépendre du statut final de la candidature.
fn enregistrer_statut(
    conn: &rusqlite::Connection,
    candidature_id: Uuid,
    statut: &str,
    changed_at: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO statut_history (id, candidature_id, statut, changed_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            Uuid::new_v4().to_string(),
            candidature_id.to_string(),
            statut,
            changed_at,
        ],
    )
    .map(|_| ())
    .map_err(|e| traduire_erreur(e, "historique du statut"))
}

impl CandidatureRepository for SqliteCandidatureRepository {
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature> {
        let mut conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
        let maintenant = maintenant_iso();
        let statut = texte_depuis_enum(&input.statut)?;
        let transaction = conn
            .transaction()
            .map_err(|e| traduire_erreur(e, "création de la candidature"))?;
        transaction
            .execute(
            "INSERT INTO candidatures (id, entreprise_id, contact_id, poste, type_contrat, statut,
                date_envoi, lien_offre, notes, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            rusqlite::params![
                id.to_string(),
                input.entreprise_id.to_string(),
                input.poste,
                texte_depuis_enum(&input.type_contrat)?,
                statut,
                input.date_envoi,
                input.lien_offre,
                input.notes,
                maintenant
            ],
            )
            .map_err(|e| {
            traduire_contrainte(
                e,
                "L'entreprise liée à cette candidature est introuvable",
                "candidature",
            )
            })?;
        enregistrer_statut(&transaction, id, &statut, &maintenant)?;
        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "création de la candidature"))?;
        conn.query_row(
            &format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_candidature(row)),
        )
        .map_err(|e| traduire_erreur(e, "candidature"))?
    }

    fn list(&self) -> AppResult<Vec<Candidature>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} {DEPUIS} ORDER BY c.date_envoi DESC, c.created_at DESC"
            ))
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut lignes = requete
            .query([])
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut candidatures = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|e| traduire_erreur(e, "candidatures"))?
        {
            candidatures.push(ligne_vers_candidature(row)?);
        }
        Ok(candidatures)
    }

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        query: &CandidaturePageQuery,
    ) -> AppResult<Page<Candidature>> {
        use rusqlite::types::Value;

        let conn = connexion(&self.pool)?;
        let page = page.max(1);
        let page_size = page_size.max(1);
        let mut clauses = Vec::<String>::new();
        let mut values = Vec::<Value>::new();
        fn push_clause(
            clauses: &mut Vec<String>,
            values: &mut Vec<Value>,
            clause: &str,
            value: Value,
        ) {
            values.push(value);
            clauses.push(clause.replace('?', &format!("?{}", values.len())));
        }

        if !query.search.trim().is_empty() {
            let needle = Value::Text(format!("%{}%", query.search.trim().to_lowercase()));
            values.push(needle.clone());
            let first = values.len();
            values.push(needle);
            let second = values.len();
            clauses.push(format!(
                "(lower(c.poste) LIKE ?{first} OR lower(coalesce(e.nom, '')) LIKE ?{second})"
            ));
        }
        if let Some(status) = query.status {
            push_clause(
                &mut clauses,
                &mut values,
                "c.statut = ?",
                Value::Text(texte_depuis_enum(&status)?),
            );
        }
        if let Some(contract) = query.contract {
            push_clause(
                &mut clauses,
                &mut values,
                "c.type_contrat = ?",
                Value::Text(texte_depuis_enum(&contract)?),
            );
        }
        if let Some(company_id) = query.company_id {
            push_clause(
                &mut clauses,
                &mut values,
                "c.entreprise_id = ?",
                Value::Text(company_id.to_string()),
            );
        }
        if !query.city.trim().is_empty() {
            push_clause(
                &mut clauses,
                &mut values,
                "lower(coalesce(e.ville, '')) LIKE ?",
                Value::Text(format!("%{}%", query.city.trim().to_lowercase())),
            );
        }
        if !query.position.trim().is_empty() {
            push_clause(
                &mut clauses,
                &mut values,
                "lower(c.poste) LIKE ?",
                Value::Text(format!("%{}%", query.position.trim().to_lowercase())),
            );
        }
        if let Some(value) = &query.date_from {
            push_clause(
                &mut clauses,
                &mut values,
                "c.date_envoi >= ?",
                Value::Text(value.clone()),
            );
        }
        if let Some(value) = &query.date_to {
            push_clause(
                &mut clauses,
                &mut values,
                "c.date_envoi <= ?",
                Value::Text(value.clone()),
            );
        }

        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) {DEPUIS}{where_sql}"),
                rusqlite::params_from_iter(values.iter()),
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "candidatures"))?;

        let order_column = match query.sort.as_str() {
            "poste" => "lower(c.poste)",
            "entreprise" => "lower(coalesce(e.nom, ''))",
            "statut" => "c.statut",
            _ => "c.date_envoi",
        };
        let direction = if query.descending { "DESC" } else { "ASC" };
        values.push(Value::Integer(i64::try_from(page_size).unwrap_or(i64::MAX)));
        let limit_index = values.len();
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        values.push(Value::Integer(i64::try_from(offset).unwrap_or(i64::MAX)));
        let offset_index = values.len();
        let sql = format!(
            "SELECT {COLONNES} {DEPUIS}{where_sql} ORDER BY {order_column} {direction}, c.created_at DESC LIMIT ?{limit_index} OFFSET ?{offset_index}"
        );
        let mut statement = conn
            .prepare(&sql)
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut rows = statement
            .query(rusqlite::params_from_iter(values.iter()))
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| traduire_erreur(e, "candidatures"))?
        {
            items.push(ligne_vers_candidature(row)?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn stats(&self) -> AppResult<CandidatureStats> {
        let conn = connexion(&self.pool)?;
        let statuses = [
            texte_depuis_enum(&StatutCandidature::EnAttente)?,
            texte_depuis_enum(&StatutCandidature::Relancee)?,
            texte_depuis_enum(&StatutCandidature::Entretien)?,
            texte_depuis_enum(&StatutCandidature::Refus)?,
        ];
        let pending_status = statuses[0].clone();
        let interview_status = statuses[2].clone();
        let (total, pending, followed_up, interviews, rejected, linked_contacts): (
            u64,
            u64,
            u64,
            u64,
            u64,
            u64,
        ) = conn
            .query_row(
                "SELECT count(*),
                    coalesce(sum(CASE WHEN statut = ?1 THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN statut = ?2 THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN statut = ?3 THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN statut = ?4 THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN contact_id IS NOT NULL THEN 1 ELSE 0 END), 0)
                 FROM candidatures",
                statuses,
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .map_err(|e| traduire_erreur(e, "statistiques candidatures"))?;
        let threshold = (chrono::Local::now().date_naive() - chrono::Duration::days(55))
            .format("%Y-%m-%d")
            .to_string();
        let mut statement = conn
            .prepare(
                "SELECT substr(date_envoi, 1, 10), count(*) FROM candidatures
                 WHERE date_envoi >= ?1 GROUP BY substr(date_envoi, 1, 10) ORDER BY 1 ASC",
            )
            .map_err(|e| traduire_erreur(e, "activité candidatures"))?;
        let rows = statement
            .query_map([threshold], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| traduire_erreur(e, "activité candidatures"))?;
        let mut activity_by_day = Vec::new();
        for row in rows {
            activity_by_day.push(row.map_err(|e| traduire_erreur(e, "activité candidatures"))?);
        }
        let converted_candidates: u64 = conn
            .query_row(
                "SELECT count(*) FROM candidatures c
                 WHERE c.statut = ?1
                    OR EXISTS (
                        SELECT 1 FROM statut_history h
                        WHERE h.candidature_id = c.id AND h.statut = ?1
                    )
                    OR EXISTS (
                        SELECT 1 FROM entretiens e WHERE e.candidature_id = c.id
                    )",
                [interview_status],
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "conversions en entretien"))?;
        let follow_up_before = (chrono::Local::now().date_naive() - chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        let to_follow_up: u64 = conn
            .query_row(
                "SELECT count(*) FROM candidatures
                 WHERE statut = ?1 AND substr(date_envoi, 1, 10) <= ?2",
                rusqlite::params![pending_status, follow_up_before],
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "candidatures à relancer"))?;
        Ok(CandidatureStats {
            total,
            pending,
            followed_up,
            interviews,
            rejected,
            linked_contacts,
            converted_candidates,
            to_follow_up,
            activity_by_day,
        })
    }

    fn list_to_follow_up(&self, before: &str, limit: u64) -> AppResult<Vec<Candidature>> {
        let conn = connexion(&self.pool)?;
        let status = texte_depuis_enum(&StatutCandidature::EnAttente)?;
        let mut statement = conn
            .prepare(&format!(
                "SELECT {COLONNES} {DEPUIS}
                 WHERE c.statut = ?1 AND substr(c.date_envoi, 1, 10) <= ?2
                 ORDER BY c.date_envoi ASC LIMIT ?3"
            ))
            .map_err(|e| traduire_erreur(e, "candidatures à relancer"))?;
        let mut rows = statement
            .query(rusqlite::params![status, before, limit.max(1)])
            .map_err(|e| traduire_erreur(e, "candidatures à relancer"))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| traduire_erreur(e, "candidatures à relancer"))?
        {
            items.push(ligne_vers_candidature(row)?);
        }
        Ok(items)
    }

    fn update(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        let mut conn = connexion(&self.pool)?;
        let ancien_statut: Option<String> = conn
            .query_row(
                "SELECT statut FROM candidatures WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        let Some(ancien_statut) = ancien_statut else {
            return Err(AppError::NotFound(format!("candidature {id}")));
        };
        let nouveau_statut = texte_depuis_enum(&input.statut)?;
        let maintenant = maintenant_iso();
        let transaction = conn
            .transaction()
            .map_err(|e| traduire_erreur(e, "modification de la candidature"))?;
        let modifiees = transaction
            .execute(
                "UPDATE candidatures SET entreprise_id = ?2, poste = ?3, type_contrat = ?4, statut = ?5,
                    date_envoi = ?6, lien_offre = ?7, notes = ?8, updated_at = ?9
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.entreprise_id.to_string(),
                    input.poste,
                    texte_depuis_enum(&input.type_contrat)?,
                    nouveau_statut,
                    input.date_envoi,
                    input.lien_offre,
                    input.notes,
                    maintenant
                ],
            )
            .map_err(|e| {
                traduire_contrainte(
                    e,
                    "L'entreprise liée à cette candidature est introuvable",
                    "candidature",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("candidature {id}")));
        }
        if ancien_statut != nouveau_statut {
            enregistrer_statut(&transaction, id, &nouveau_statut, &maintenant)?;
        }
        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "modification de la candidature"))?;
        conn.query_row(
            &format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_candidature(row)),
        )
        .map_err(|e| traduire_erreur(e, "candidature"))?
    }

    fn update_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature> {
        let mut conn = connexion(&self.pool)?;
        let ancien_statut: Option<String> = conn
            .query_row(
                "SELECT statut FROM candidatures WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        let Some(ancien_statut) = ancien_statut else {
            return Err(AppError::NotFound(format!("candidature {id}")));
        };
        let nouveau_statut = texte_depuis_enum(&statut)?;
        let maintenant = maintenant_iso();
        let transaction = conn
            .transaction()
            .map_err(|e| traduire_erreur(e, "changement du statut"))?;
        let modifiees = transaction
            .execute(
                "UPDATE candidatures SET statut = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id.to_string(), nouveau_statut, maintenant],
            )
            .map_err(|e| traduire_erreur(e, "candidature invalide"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("candidature {id}")));
        }
        if ancien_statut != nouveau_statut {
            enregistrer_statut(&transaction, id, &nouveau_statut, &maintenant)?;
        }
        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "changement du statut"))?;
        conn.query_row(
            &format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_candidature(row)),
        )
        .map_err(|e| traduire_erreur(e, "candidature"))?
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM candidatures WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
