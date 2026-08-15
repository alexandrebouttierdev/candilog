-- Référentiel des secteurs d'activité, rattaché aux entreprises via `secteurs_activite.id`.
--
-- La liste stable des secteurs et le rattachement des valeurs libres déjà saisies sont posés
-- par le seeder Rust (`modules::secteurs::repository::garantir_referentiel`), seul générateur
-- d'identifiants UUID avec le reste de la base. Cette migration ne crée que la table, la
-- colonne de liaison et l'index.
CREATE TABLE IF NOT EXISTS secteurs_activite (
    id         TEXT PRIMARY KEY,
    nom        TEXT NOT NULL UNIQUE COLLATE NOCASE,
    ordre      INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

ALTER TABLE entreprises ADD COLUMN secteur_id TEXT
    REFERENCES secteurs_activite(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_entreprises_secteur_id ON entreprises(secteur_id);
