//! Dépôt `SQLite` du référentiel des secteurs d'activité.

use crate::core::database::helpers::{connection, now_iso, translate_error, uuid_column};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::sectors::domain::{ActivitySector, SectorRepository};
use rusqlite::OptionalExtension;

/// List stable des secteurs proposés dans le formulaire entreprise, dans l'ordre d'affichage.
pub const SECTORS_CANONIQUES: [&str; 23] = [
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
    "Other",
];

/// Implémentation `SQLite` du référentiel des secteurs.
pub struct SqliteSectorRepository {
    pool: SqlitePool,
}

impl SqliteSectorRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn ensure_catalog(&self) -> AppResult<()> {
        let mut conn = connection(&self.pool)?;
        let transaction = conn
            .transaction()
            .map_err(|e| AppError::Database(e.to_string()))?;
        for (index, name) in SECTORS_CANONIQUES.iter().enumerate() {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO sectors (id, name, sort_order, created_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        uuid::Uuid::new_v4().to_string(),
                        name,
                        i64::try_from(index + 1).unwrap_or(i64::MAX),
                        now_iso()
                    ],
                )
                .map_err(|e| translate_error(e, "secteur d'activité"))?;
        }
        let values_libres: Vec<String> = {
            let mut query = transaction
                .prepare(
                    "SELECT DISTINCT trim(sector) FROM companies
                     WHERE sector IS NOT NULL AND trim(sector) <> ''",
                )
                .map_err(|e| translate_error(e, "secteur d'activité"))?;
            let rows = query
                .query_map([], |row| row.get(0))
                .map_err(|e| translate_error(e, "secteur d'activité"))?;
            let mut values = Vec::new();
            for row in rows {
                values.push(row.map_err(|e| translate_error(e, "secteur d'activité"))?);
            }
            values
        };
        for value in values_libres {
            let id_existant: Option<String> = transaction
                .query_row(
                    "SELECT id FROM sectors WHERE name = ?1 COLLATE NOCASE",
                    [&value],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| translate_error(e, "secteur d'activité"))?;
            let id = match id_existant {
                Some(id) => id,
                None => {
                    let id = uuid::Uuid::new_v4().to_string();
                    transaction
                        .execute(
                            "INSERT INTO sectors (id, name, sort_order, created_at)
                             VALUES (
                                 ?1,
                                 ?2,
                                 (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM sectors),
                                 ?3
                             )",
                            rusqlite::params![id, &value, now_iso()],
                        )
                        .map_err(|e| translate_error(e, "secteur d'activité"))?;
                    id
                }
            };
            transaction
                .execute(
                    "UPDATE companies SET sector_id = ?1
                     WHERE sector_id IS NULL AND trim(sector) = ?2 COLLATE NOCASE",
                    rusqlite::params![id, &value],
                )
                .map_err(|e| translate_error(e, "secteur d'activité"))?;
        }
        transaction
            .commit()
            .map_err(|e| AppError::Database(e.to_string()))
    }
}

impl SectorRepository for SqliteSectorRepository {
    fn list(&self) -> AppResult<Vec<ActivitySector>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(
                "SELECT id, name FROM sectors
                 ORDER BY sort_order ASC, name COLLATE NOCASE ASC",
            )
            .map_err(|e| translate_error(e, "secteurs d'activité"))?;
        let rows = query
            .query_map([], |row| {
                Ok(ActivitySector {
                    id: uuid_column(row, 0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| translate_error(e, "secteurs d'activité"))?;
        let mut sectors = Vec::new();
        for row in rows {
            sectors.push(row.map_err(|e| translate_error(e, "secteurs d'activité"))?);
        }
        Ok(sectors)
    }
}
