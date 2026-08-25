//! Dépôt `SQLite` des candidatures.

use crate::core::database::helpers::{
    connexion, enum_depuis_texte, maintenant_iso, texte_depuis_enum, traduire_contrainte,
    traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::features::candidatures::domain::{
    Candidature, CandidatureRepository, FiltreCandidatures, NouvelleCandidature,
    RepartitionPipeline, StatutCandidature, TriCandidature,
};
use rusqlite::types::Value;
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt de candidatures.
pub struct SqliteCandidatureRepository {
    pool: SqlitePool,
}

impl SqliteCandidatureRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_candidature`], dans l'ordre.
const COLONNES: &str = "c.id, c.poste, c.entreprise_id, e.nom, e.ville, c.contact_id, \
                        c.type_contrat, c.statut, c.date_envoi, c.lien_offre, c.notes, \
                        c.created_at, c.updated_at";

/// Source des colonnes : `LEFT JOIN` pour exposer le nom et la ville de l'entreprise.
const DEPUIS: &str = "FROM candidatures c LEFT JOIN entreprises e ON e.id = c.entreprise_id";

/// Convertit une ligne `SQLite` en candidature du domaine.
///
/// Renvoie `AppResult` et non `rusqlite::Result` : `type_contrat` et `statut` sont des enums
/// serde, dont la conversion depuis le `TEXT` stocké peut échouer sur une valeur héritée
/// qu'aucune migration n'aurait normalisée.
fn ligne_vers_candidature(row: &rusqlite::Row) -> AppResult<Candidature> {
    let lire = |index: usize| -> AppResult<String> {
        row.get(index)
            .map_err(|e| traduire_erreur(e, "candidature"))
    };
    let type_contrat = lire(6)?;
    let statut = lire(7)?;
    Ok(Candidature {
        id: uuid_colonne(row, 0).map_err(|e| traduire_erreur(e, "candidature"))?,
        poste: lire(1)?,
        entreprise_id: uuid_colonne(row, 2).map_err(|e| traduire_erreur(e, "candidature"))?,
        entreprise_nom: row.get(3).map_err(|e| traduire_erreur(e, "candidature"))?,
        entreprise_ville: row.get(4).map_err(|e| traduire_erreur(e, "candidature"))?,
        contact_id: uuid_colonne_opt(row, 5).map_err(|e| traduire_erreur(e, "candidature"))?,
        type_contrat: enum_depuis_texte(&type_contrat)?,
        statut: enum_depuis_texte(&statut)?,
        date_envoi: lire(8)?,
        lien_offre: row.get(9).map_err(|e| traduire_erreur(e, "candidature"))?,
        notes: row.get(10).map_err(|e| traduire_erreur(e, "candidature"))?,
        created_at: lire(11)?,
        updated_at: lire(12)?,
    })
}

/// Ajoute une étape à l'historique de statut.
///
/// L'historique est ce qui permet de compter les candidatures **passées** par l'entretien,
/// y compris celles qui ont ensuite été refusées : le statut courant seul les perdrait, et
/// l'entonnoir de conversion des analyses afficherait un taux faux.
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

/// Clauses `WHERE` et paramètres liés correspondant à un filtre.
///
/// Chaque critère est un paramètre lié, jamais une valeur interpolée : le poste, la ville et
/// la recherche libre viennent de champs de saisie, et les concaténer au SQL ouvrirait une
/// injection.
fn clauses(filtre: &FiltreCandidatures) -> AppResult<(String, Vec<Value>)> {
    let mut clauses = Vec::<String>::new();
    let mut valeurs = Vec::<Value>::new();

    let ajouter =
        |clause: &str, valeur: Value, valeurs: &mut Vec<Value>, clauses: &mut Vec<String>| {
            valeurs.push(valeur);
            clauses.push(clause.replace('?', &format!("?{}", valeurs.len())));
        };

    let motif = |texte: &str| Value::Text(format!("%{}%", texte.trim().to_lowercase()));

    if !filtre.search.trim().is_empty() {
        valeurs.push(motif(&filtre.search));
        let premier = valeurs.len();
        valeurs.push(motif(&filtre.search));
        let second = valeurs.len();
        clauses.push(format!(
            "(lower(c.poste) LIKE ?{premier} OR lower(coalesce(e.nom, '')) LIKE ?{second})"
        ));
    }
    if let Some(statut) = filtre.statut {
        ajouter(
            "c.statut = ?",
            Value::Text(texte_depuis_enum(&statut)?),
            &mut valeurs,
            &mut clauses,
        );
    }
    if let Some(contrat) = filtre.contrat {
        ajouter(
            "c.type_contrat = ?",
            Value::Text(texte_depuis_enum(&contrat)?),
            &mut valeurs,
            &mut clauses,
        );
    }
    if let Some(entreprise_id) = filtre.entreprise_id {
        ajouter(
            "c.entreprise_id = ?",
            Value::Text(entreprise_id.to_string()),
            &mut valeurs,
            &mut clauses,
        );
    }
    if !filtre.ville.trim().is_empty() {
        ajouter(
            "lower(coalesce(e.ville, '')) LIKE ?",
            motif(&filtre.ville),
            &mut valeurs,
            &mut clauses,
        );
    }
    if !filtre.poste.trim().is_empty() {
        ajouter(
            "lower(c.poste) LIKE ?",
            motif(&filtre.poste),
            &mut valeurs,
            &mut clauses,
        );
    }
    if let Some(debut) = &filtre.date_debut {
        ajouter(
            "c.date_envoi >= ?",
            Value::Text(debut.clone()),
            &mut valeurs,
            &mut clauses,
        );
    }
    if let Some(fin) = &filtre.date_fin {
        ajouter(
            "c.date_envoi <= ?",
            Value::Text(fin.clone()),
            &mut valeurs,
            &mut clauses,
        );
    }

    let sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    Ok((sql, valeurs))
}

/// Expression `ORDER BY` correspondant à la colonne de tri.
///
/// Le jeu de valeurs est fermé par l'enum : rien de ce qui vient de l'IPC n'atteint le SQL.
const fn colonne_de_tri(tri: TriCandidature) -> &'static str {
    match tri {
        TriCandidature::Poste => "lower(c.poste)",
        TriCandidature::Entreprise => "lower(coalesce(e.nom, ''))",
        TriCandidature::Statut => "c.statut",
        TriCandidature::Date => "c.date_envoi",
    }
}

impl CandidatureRepository for SqliteCandidatureRepository {
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

    fn get(&self, id: Uuid) -> AppResult<Candidature> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"))
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        let mut lignes = requete
            .query([id.to_string()])
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        match lignes
            .next()
            .map_err(|e| traduire_erreur(e, "candidature"))?
        {
            Some(row) => ligne_vers_candidature(row),
            None => Err(AppError::NotFound(format!("candidature {id}"))),
        }
    }

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filtre: &FiltreCandidatures,
    ) -> AppResult<Page<Candidature>> {
        let conn = connexion(&self.pool)?;
        let page = page.max(1);
        let page_size = page_size.max(1);
        let (where_sql, mut valeurs) = clauses(filtre)?;

        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) {DEPUIS}{where_sql}"),
                rusqlite::params_from_iter(valeurs.iter()),
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "candidatures"))?;

        let direction = if filtre.descendant { "DESC" } else { "ASC" };
        valeurs.push(Value::Integer(i64::try_from(page_size).unwrap_or(i64::MAX)));
        let index_limite = valeurs.len();
        valeurs.push(Value::Integer(
            i64::try_from(Page::<Candidature>::offset(page, page_size)).unwrap_or(i64::MAX),
        ));
        let index_offset = valeurs.len();

        // `c.created_at DESC` en second critère : sans lui, deux candidatures de même date
        // d'envoi changeraient d'ordre d'une page à l'autre, et une ligne pourrait
        // apparaître deux fois ou pas du tout à la pagination.
        let sql = format!(
            "SELECT {COLONNES} {DEPUIS}{where_sql} ORDER BY {} {direction}, c.created_at DESC \
             LIMIT ?{index_limite} OFFSET ?{index_offset}",
            colonne_de_tri(filtre.tri)
        );
        let mut requete = conn
            .prepare(&sql)
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut lignes = requete
            .query(rusqlite::params_from_iter(valeurs.iter()))
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut items = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|e| traduire_erreur(e, "candidatures"))?
        {
            items.push(ligne_vers_candidature(row)?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn repartition(&self, filtre: &FiltreCandidatures) -> AppResult<RepartitionPipeline> {
        let conn = connexion(&self.pool)?;
        // La répartition ignore le filtre de statut : le Kanban affiche les quatre colonnes,
        // et n'en compter qu'une viderait les trois autres.
        let mut sans_statut = filtre.clone();
        sans_statut.statut = None;
        let (where_sql, mut valeurs) = clauses(&sans_statut)?;

        let statuts = [
            texte_depuis_enum(&StatutCandidature::EnAttente)?,
            texte_depuis_enum(&StatutCandidature::Relancee)?,
            texte_depuis_enum(&StatutCandidature::Entretien)?,
            texte_depuis_enum(&StatutCandidature::Refus)?,
        ];
        let base = valeurs.len();
        for statut in &statuts {
            valeurs.push(Value::Text(statut.clone()));
        }

        let sql = format!(
            "SELECT coalesce(sum(CASE WHEN c.statut = ?{} THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.statut = ?{} THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.statut = ?{} THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.statut = ?{} THEN 1 ELSE 0 END), 0)
             {DEPUIS}{where_sql}",
            base + 1,
            base + 2,
            base + 3,
            base + 4,
        );
        conn.query_row(&sql, rusqlite::params_from_iter(valeurs.iter()), |row| {
            Ok(RepartitionPipeline {
                en_attente: row.get(0)?,
                relancee: row.get(1)?,
                entretien: row.get(2)?,
                refus: row.get(3)?,
            })
        })
        .map_err(|e| traduire_erreur(e, "répartition du pipeline"))
    }

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
                "INSERT INTO candidatures (id, entreprise_id, contact_id, poste, type_contrat,
                    statut, date_envoi, lien_offre, notes, created_at, updated_at)
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
        // Même transaction que l'insertion : une candidature sans étape initiale serait
        // invisible de l'entonnoir de conversion.
        enregistrer_statut(&transaction, id, &statut, &maintenant)?;
        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "création de la candidature"))?;
        self.get(id)
    }

    fn update(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        let mut conn = connexion(&self.pool)?;
        let maintenant = maintenant_iso();
        let statut = texte_depuis_enum(&input.statut)?;
        let transaction = conn
            .transaction()
            .map_err(|e| traduire_erreur(e, "modification de la candidature"))?;

        let ancien_statut: Option<String> = transaction
            .query_row(
                "SELECT statut FROM candidatures WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .ok();
        let Some(ancien_statut) = ancien_statut else {
            return Err(AppError::NotFound(format!("candidature {id}")));
        };

        transaction
            .execute(
                "UPDATE candidatures SET entreprise_id = ?2, poste = ?3, type_contrat = ?4,
                    statut = ?5, date_envoi = ?6, lien_offre = ?7, notes = ?8, updated_at = ?9
                 WHERE id = ?1",
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
        // L'historique n'enregistre que les changements réels : réenregistrer le poste sans
        // toucher au statut ajouterait une étape fictive à l'entonnoir.
        if ancien_statut != statut {
            enregistrer_statut(&transaction, id, &statut, &maintenant)?;
        }
        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "modification de la candidature"))?;
        self.get(id)
    }

    fn update_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature> {
        let mut conn = connexion(&self.pool)?;
        let maintenant = maintenant_iso();
        let statut = texte_depuis_enum(&statut)?;
        let transaction = conn
            .transaction()
            .map_err(|e| traduire_erreur(e, "changement de statut"))?;
        let modifiees = transaction
            .execute(
                "UPDATE candidatures SET statut = ?2, updated_at = ?3
                 WHERE id = ?1 AND statut <> ?2",
                rusqlite::params![id.to_string(), statut, maintenant],
            )
            .map_err(|e| traduire_erreur(e, "changement de statut"))?;
        if modifiees > 0 {
            enregistrer_statut(&transaction, id, &statut, &maintenant)?;
        }
        transaction
            .commit()
            .map_err(|e| traduire_erreur(e, "changement de statut"))?;
        // `modifiees == 0` couvre deux cas : identifiant inconnu, ou statut déjà à la valeur
        // demandée — un glisser-déposer reposant la carte dans sa colonne d'origine. `get`
        // distingue les deux en renvoyant `NotFound` pour le premier.
        self.get(id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        // Relances, entretiens et historique de statut partent en cascade (`ON DELETE
        // CASCADE` du schéma) ; l'entreprise et le contact sont conservés.
        conn.execute("DELETE FROM candidatures WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
