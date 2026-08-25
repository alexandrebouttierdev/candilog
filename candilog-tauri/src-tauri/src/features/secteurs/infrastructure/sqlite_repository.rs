//! Dépôt `SQLite` du référentiel des secteurs d'activité.

use crate::core::database::helpers::{connexion, maintenant_iso, traduire_erreur, uuid_colonne};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::secteurs::domain::{SecteurActivite, SecteurRepository};
use rusqlite::OptionalExtension;

/// Liste stable des secteurs proposés dans le formulaire entreprise, dans l'ordre d'affichage.
pub const SECTEURS_CANONIQUES: [&str; 23] = [
    "Achats / Comptabilité / Gestion",
    "Arts / Artisanat d'art",
    "Banque / Assurance",
    "Bâtiment / Travaux Publics",
    "Commerce / Vente",
    "Communication / Multimédia",
    "Conseil / Études",
    "Direction d'entreprise",
    "Espaces verts et naturels / Agriculture / Pêche / Soins aux animaux",
    "Hôtellerie - Restauration / Tourisme / Animation",
    "Immobilier",
    "Industrie",
    "Informatique / Télécommunication",
    "Installation / Maintenance",
    "Marketing / Stratégie commerciale",
    "Ressources Humaines",
    "Santé",
    "Secrétariat / Assistanat",
    "Services à la personne / à la collectivité",
    "Spectacle",
    "Sport",
    "Transport / Logistique",
    "Autre",
];

/// Implémentation `SQLite` du référentiel des secteurs.
pub struct SqliteSecteurRepository {
    pool: SqlitePool,
}

impl SqliteSecteurRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn garantir_referentiel(&self) -> AppResult<()> {
        let mut conn = connexion(&self.pool)?;
        let transaction = conn
            .transaction()
            .map_err(|e| AppError::Database(e.to_string()))?;
        for (index, nom) in SECTEURS_CANONIQUES.iter().enumerate() {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO secteurs_activite (id, nom, ordre, created_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        uuid::Uuid::new_v4().to_string(),
                        nom,
                        i64::try_from(index + 1).unwrap_or(i64::MAX),
                        maintenant_iso()
                    ],
                )
                .map_err(|e| traduire_erreur(e, "secteur d'activité"))?;
        }
        let valeurs_libres: Vec<String> = {
            let mut requete = transaction
                .prepare(
                    "SELECT DISTINCT trim(secteur) FROM entreprises
                     WHERE secteur IS NOT NULL AND trim(secteur) <> ''",
                )
                .map_err(|e| traduire_erreur(e, "secteur d'activité"))?;
            let lignes = requete
                .query_map([], |row| row.get(0))
                .map_err(|e| traduire_erreur(e, "secteur d'activité"))?;
            let mut valeurs = Vec::new();
            for ligne in lignes {
                valeurs.push(ligne.map_err(|e| traduire_erreur(e, "secteur d'activité"))?);
            }
            valeurs
        };
        for valeur in valeurs_libres {
            let id_existant: Option<String> = transaction
                .query_row(
                    "SELECT id FROM secteurs_activite WHERE nom = ?1 COLLATE NOCASE",
                    [&valeur],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| traduire_erreur(e, "secteur d'activité"))?;
            let id = match id_existant {
                Some(id) => id,
                None => {
                    let id = uuid::Uuid::new_v4().to_string();
                    transaction
                        .execute(
                            "INSERT INTO secteurs_activite (id, nom, ordre, created_at)
                             VALUES (
                                 ?1,
                                 ?2,
                                 (SELECT COALESCE(MAX(ordre), 0) + 1 FROM secteurs_activite),
                                 ?3
                             )",
                            rusqlite::params![id, &valeur, maintenant_iso()],
                        )
                        .map_err(|e| traduire_erreur(e, "secteur d'activité"))?;
                    id
                }
            };
            transaction
                .execute(
                    "UPDATE entreprises SET secteur_id = ?1
                     WHERE secteur_id IS NULL AND trim(secteur) = ?2 COLLATE NOCASE",
                    rusqlite::params![id, &valeur],
                )
                .map_err(|e| traduire_erreur(e, "secteur d'activité"))?;
        }
        transaction
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))
    }
}

impl SecteurRepository for SqliteSecteurRepository {
    fn lister(&self) -> AppResult<Vec<SecteurActivite>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(
                "SELECT id, nom FROM secteurs_activite
                 ORDER BY ordre ASC, nom COLLATE NOCASE ASC",
            )
            .map_err(|e| traduire_erreur(e, "secteurs d'activité"))?;
        let lignes = requete
            .query_map([], |row| {
                Ok(SecteurActivite {
                    id: uuid_colonne(row, 0)?,
                    nom: row.get(1)?,
                })
            })
            .map_err(|e| traduire_erreur(e, "secteurs d'activité"))?;
        let mut secteurs = Vec::new();
        for ligne in lignes {
            secteurs.push(ligne.map_err(|e| traduire_erreur(e, "secteurs d'activité"))?);
        }
        Ok(secteurs)
    }
}
