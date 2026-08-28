//! Agrégats d'analyse calculés par `SQLite`.

use crate::core::database::helpers::{
    connexion, enum_depuis_texte, traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use crate::core::database::SqlitePool;
use crate::core::errors::AppResult;
use crate::features::analyses::domain::{
    ARelancer, AnalysesRepository, Echeance, Etape, Indicateurs, Performance, SemaineActivite,
};
use crate::features::candidatures::domain::{Candidature, StatutCandidature, TypeContrat};

/// Dépôt des analyses sur la base locale.
pub struct SqliteAnalysesRepository {
    pool: SqlitePool,
}

impl SqliteAnalysesRepository {
    /// Construit le dépôt à partir du pool partagé.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn pourcentage(part: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part.min(total) as f64 / total as f64) * 100.0).round() as u8
}

impl AnalysesRepository for SqliteAnalysesRepository {
    fn indicateurs(&self, depuis: Option<&str>) -> AppResult<Indicateurs> {
        let conn = connexion(&self.pool)?;
        let (candidatures, entretiens, reponses, refus, en_attente, relancees) = conn
            .query_row(
                "SELECT count(*),
                    coalesce(sum(CASE WHEN
                        c.statut = 'ENTRETIEN'
                        OR EXISTS (SELECT 1 FROM statut_history h
                                   WHERE h.candidature_id = c.id AND h.statut = 'ENTRETIEN')
                        OR EXISTS (SELECT 1 FROM entretiens e WHERE e.candidature_id = c.id)
                    THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN
                        c.statut IN ('ENTRETIEN', 'REFUS')
                        OR EXISTS (SELECT 1 FROM statut_history h
                                   WHERE h.candidature_id = c.id
                                     AND h.statut IN ('ENTRETIEN', 'REFUS'))
                        OR EXISTS (SELECT 1 FROM entretiens e WHERE e.candidature_id = c.id)
                    THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.statut = 'REFUS' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN c.statut = 'EN_ATTENTE' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN
                        c.statut = 'RELANCEE'
                        OR EXISTS (SELECT 1 FROM statut_history h
                                   WHERE h.candidature_id = c.id AND h.statut = 'RELANCEE')
                        OR EXISTS (SELECT 1 FROM relances r WHERE r.candidature_id = c.id)
                    THEN 1 ELSE 0 END), 0)
                 FROM candidatures c
                 WHERE ?1 IS NULL OR substr(c.date_envoi, 1, 10) >= ?1",
                rusqlite::params![depuis],
                |row| {
                    Ok((
                        row.get::<_, u64>(0)?,
                        row.get::<_, u64>(1)?,
                        row.get::<_, u64>(2)?,
                        row.get::<_, u64>(3)?,
                        row.get::<_, u64>(4)?,
                        row.get::<_, u64>(5)?,
                    ))
                },
            )
            .map_err(|error| traduire_erreur(error, "indicateurs"))?;
        Ok(Indicateurs {
            candidatures,
            entretiens,
            reponses,
            refus,
            en_attente,
            relancees,
            taux_reponse: pourcentage(reponses, candidatures),
            taux_entretien: pourcentage(entretiens, candidatures),
        })
    }

    fn performance(&self, depuis: Option<&str>) -> AppResult<Performance> {
        let conn = connexion(&self.pool)?;
        let (nombre, premiere): (u64, Option<String>) = conn
            .query_row(
                "SELECT count(*), min(substr(date_envoi, 1, 10)) FROM candidatures
                 WHERE ?1 IS NULL OR substr(date_envoi, 1, 10) >= ?1",
                rusqlite::params![depuis],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| traduire_erreur(error, "rythme des candidatures"))?;

        let delai: Option<f64> = conn
            .query_row(
                "WITH reponses(candidature_id, jour) AS (
                    SELECT candidature_id, substr(changed_at, 1, 10)
                    FROM statut_history WHERE statut IN ('ENTRETIEN', 'REFUS')
                    UNION ALL
                    SELECT candidature_id, substr(date_entretien, 1, 10) FROM entretiens
                    UNION ALL
                    SELECT id, substr(updated_at, 1, 10) FROM candidatures
                    WHERE statut IN ('ENTRETIEN', 'REFUS')
                 ), premieres AS (
                    SELECT candidature_id, min(jour) AS jour
                    FROM reponses GROUP BY candidature_id
                 )
                 SELECT avg(max(0, julianday(p.jour) - julianday(substr(c.date_envoi, 1, 10))))
                 FROM candidatures c
                 JOIN premieres p ON p.candidature_id = c.id
                 WHERE ?1 IS NULL OR substr(c.date_envoi, 1, 10) >= ?1",
                rusqlite::params![depuis],
                |row| row.get(0),
            )
            .map_err(|error| traduire_erreur(error, "délai de réponse"))?;

        let aujourdhui = chrono::Local::now().date_naive();
        let debut = depuis
            .and_then(|valeur| chrono::NaiveDate::parse_from_str(valeur, "%Y-%m-%d").ok())
            .or_else(|| {
                premiere
                    .as_deref()
                    .and_then(|valeur| chrono::NaiveDate::parse_from_str(valeur, "%Y-%m-%d").ok())
            })
            .unwrap_or(aujourdhui);
        let semaines = ((aujourdhui - debut).num_days().max(0) as f64 / 7.0).max(1.0);
        let candidatures_par_semaine = ((nombre as f64 / semaines) * 10.0).round() / 10.0;
        let jour = aujourdhui.format("%Y-%m-%d").to_string();
        let entretiens_a_venir = conn
            .query_row(
                "SELECT count(*) FROM entretiens WHERE substr(date_entretien, 1, 10) >= ?1",
                [&jour],
                |row| row.get(0),
            )
            .map_err(|error| traduire_erreur(error, "entretiens à venir"))?;
        let relances_en_retard = conn
            .query_row(
                "SELECT count(*) FROM relances WHERE date_relance < ?1",
                [&jour],
                |row| row.get(0),
            )
            .map_err(|error| traduire_erreur(error, "relances en retard"))?;

        Ok(Performance {
            delai_moyen_reponse: delai.map(|valeur| valeur.round().max(0.0) as u64),
            candidatures_par_semaine,
            entretiens_a_venir,
            relances_en_retard,
        })
    }

    fn activite_hebdomadaire(&self, semaines: u32) -> AppResult<Vec<SemaineActivite>> {
        let conn = connexion(&self.pool)?;
        let semaines = semaines.clamp(1, 104);
        let aujourdhui = chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        let mut requete = conn
            .prepare(
                "WITH RECURSIVE periodes(indice, debut, fin) AS (
                    SELECT 0,
                           date(?1, printf('-%d days', (?2 * 7) - 1)),
                           date(?1, printf('-%d days', (?2 * 7) - 7))
                    UNION ALL
                    SELECT indice + 1, date(debut, '+7 days'), date(fin, '+7 days')
                    FROM periodes WHERE indice + 1 < ?2
                 )
                 SELECT p.debut, count(c.id)
                 FROM periodes p
                 LEFT JOIN candidatures c
                   ON substr(c.date_envoi, 1, 10) BETWEEN p.debut AND p.fin
                 GROUP BY p.indice, p.debut ORDER BY p.indice",
            )
            .map_err(|error| traduire_erreur(error, "activité hebdomadaire"))?;
        let lignes = requete
            .query_map(rusqlite::params![aujourdhui, semaines], |row| {
                Ok(SemaineActivite {
                    debut: row.get(0)?,
                    nombre: row.get(1)?,
                })
            })
            .map_err(|error| traduire_erreur(error, "activité hebdomadaire"))?;
        lignes
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| traduire_erreur(error, "activité hebdomadaire"))
    }

    fn pipeline(&self) -> AppResult<Vec<Etape>> {
        let conn = connexion(&self.pool)?;
        let (total, attente, relancees, entretiens, refus): (u64, u64, u64, u64, u64) = conn
            .query_row(
                "SELECT count(*),
                    coalesce(sum(CASE WHEN statut = 'EN_ATTENTE' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN statut = 'RELANCEE' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN statut = 'ENTRETIEN' THEN 1 ELSE 0 END), 0),
                    coalesce(sum(CASE WHEN statut = 'REFUS' THEN 1 ELSE 0 END), 0)
                 FROM candidatures",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .map_err(|error| traduire_erreur(error, "pipeline"))?;
        Ok([
            ("En attente", attente),
            ("Relancées", relancees),
            ("Entretiens", entretiens),
            ("Refusées", refus),
        ]
        .into_iter()
        .map(|(label, nombre)| Etape {
            label: label.into(),
            nombre,
            pourcentage: pourcentage(nombre, total),
        })
        .collect())
    }

    fn echeances(&self, aujourdhui: &str, limite: u64) -> AppResult<Vec<Echeance>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(
                "SELECT e.id, 'entretien', e.date_entretien, c.poste, ent.nom, e.type
                 FROM entretiens e
                 LEFT JOIN candidatures c ON c.id = e.candidature_id
                 LEFT JOIN entreprises ent ON ent.id = c.entreprise_id
                 WHERE substr(e.date_entretien, 1, 10) >= ?1
                 UNION ALL
                 SELECT r.id, 'relance', r.date_relance, c.poste, ent.nom, r.type
                 FROM relances r
                 LEFT JOIN candidatures c ON c.id = r.candidature_id
                 LEFT JOIN entreprises ent ON ent.id = c.entreprise_id
                 WHERE r.date_relance >= ?1
                 ORDER BY 3 ASC LIMIT ?2",
            )
            .map_err(|error| traduire_erreur(error, "échéances"))?;
        let mut lignes = requete
            .query(rusqlite::params![aujourdhui, limite.max(1)])
            .map_err(|error| traduire_erreur(error, "échéances"))?;
        let mut items = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|error| traduire_erreur(error, "échéances"))?
        {
            items.push(Echeance {
                id: uuid_colonne(row, 0).map_err(|error| traduire_erreur(error, "échéance"))?,
                genre: row
                    .get(1)
                    .map_err(|error| traduire_erreur(error, "échéance"))?,
                date: row
                    .get(2)
                    .map_err(|error| traduire_erreur(error, "échéance"))?,
                poste: row
                    .get(3)
                    .map_err(|error| traduire_erreur(error, "échéance"))?,
                entreprise_nom: row
                    .get(4)
                    .map_err(|error| traduire_erreur(error, "échéance"))?,
                detail: row
                    .get(5)
                    .map_err(|error| traduire_erreur(error, "échéance"))?,
            });
        }
        Ok(items)
    }

    fn a_relancer(&self, aujourdhui: &str, jours: u64, limite: u64) -> AppResult<Vec<ARelancer>> {
        let conn = connexion(&self.pool)?;
        let date = chrono::NaiveDate::parse_from_str(aujourdhui, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let seuil = (date - chrono::Duration::days(i64::try_from(jours).unwrap_or(i64::MAX)))
            .format("%Y-%m-%d")
            .to_string();
        let mut requete = conn
            .prepare(
                "SELECT c.id, c.poste, e.nom, substr(c.date_envoi, 1, 10),
                        cast(max(0, julianday(?1) - julianday(substr(c.date_envoi, 1, 10))) AS INTEGER)
                 FROM candidatures c
                 LEFT JOIN entreprises e ON e.id = c.entreprise_id
                 WHERE c.statut = 'EN_ATTENTE' AND substr(c.date_envoi, 1, 10) <= ?2
                 ORDER BY c.date_envoi ASC LIMIT ?3",
            )
            .map_err(|error| traduire_erreur(error, "candidatures à relancer"))?;
        let mut lignes = requete
            .query(rusqlite::params![aujourdhui, seuil, limite.max(1)])
            .map_err(|error| traduire_erreur(error, "candidatures à relancer"))?;
        let mut items = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|error| traduire_erreur(error, "candidatures à relancer"))?
        {
            items.push(ARelancer {
                id: uuid_colonne(row, 0)
                    .map_err(|error| traduire_erreur(error, "candidature à relancer"))?,
                poste: row
                    .get(1)
                    .map_err(|error| traduire_erreur(error, "candidature à relancer"))?,
                entreprise_nom: row
                    .get(2)
                    .map_err(|error| traduire_erreur(error, "candidature à relancer"))?,
                date_envoi: row
                    .get(3)
                    .map_err(|error| traduire_erreur(error, "candidature à relancer"))?,
                jours: row
                    .get(4)
                    .map_err(|error| traduire_erreur(error, "candidature à relancer"))?,
            });
        }
        Ok(items)
    }

    fn recentes(&self, limite: u64) -> AppResult<Vec<Candidature>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(
                "SELECT c.id, c.poste, c.entreprise_id, e.nom, e.ville, c.contact_id,
                        c.type_contrat, c.statut, c.date_envoi, c.lien_offre, c.notes,
                        c.created_at, c.updated_at
                 FROM candidatures c
                 LEFT JOIN entreprises e ON e.id = c.entreprise_id
                 ORDER BY c.updated_at DESC LIMIT ?1",
            )
            .map_err(|error| traduire_erreur(error, "candidatures récentes"))?;
        let mut lignes = requete
            .query([limite.max(1)])
            .map_err(|error| traduire_erreur(error, "candidatures récentes"))?;
        let mut items = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|error| traduire_erreur(error, "candidatures récentes"))?
        {
            let contrat: String = row
                .get(6)
                .map_err(|error| traduire_erreur(error, "candidature récente"))?;
            let statut: String = row
                .get(7)
                .map_err(|error| traduire_erreur(error, "candidature récente"))?;
            items.push(Candidature {
                id: uuid_colonne(row, 0)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                poste: row
                    .get(1)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                entreprise_id: uuid_colonne(row, 2)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                entreprise_nom: row
                    .get(3)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                entreprise_ville: row
                    .get(4)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                contact_id: uuid_colonne_opt(row, 5)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                type_contrat: enum_depuis_texte::<TypeContrat>(&contrat)?,
                statut: enum_depuis_texte::<StatutCandidature>(&statut)?,
                date_envoi: row
                    .get(8)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                lien_offre: row
                    .get(9)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                notes: row
                    .get(10)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                created_at: row
                    .get(11)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
                updated_at: row
                    .get(12)
                    .map_err(|error| traduire_erreur(error, "candidature récente"))?,
            });
        }
        Ok(items)
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
