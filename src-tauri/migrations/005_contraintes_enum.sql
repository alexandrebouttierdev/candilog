-- Ajoute des contraintes CHECK sur les colonnes correspondant aux enums Rust.
-- SQLite ne supportant pas ALTER TABLE ADD CONSTRAINT, on recrée les tables.
--
-- Le rejeu est empêché par le curseur `PRAGMA user_version` (src/shared/db.rs) : ce fichier
-- n'est pas idempotent par lui-même et n'a pas à l'être.
--
-- Deux précautions indispensables, toutes deux couvertes par des tests de migration :
--   1. `run_local_migrations` désactive `PRAGMA foreign_keys` autour de la transaction.
--      Sans cela, le `DROP TABLE candidatures` réalise un DELETE implicite qui déclenche les
--      `ON DELETE CASCADE` de `relances`, `entretiens` et `statut_history` — soit la perte
--      intégrale du suivi, les candidatures étant elles préservées par la table de travail.
--   2. La recopie utilise un `INSERT` strict, précédé d'une normalisation des valeurs
--      héritées. Un `INSERT OR IGNORE` ferait disparaître sans un mot toute ligne violant un
--      nouveau CHECK ; ici, soit la valeur est ramenée dans la liste, soit la migration échoue
--      de façon visible.

-- ── candidatures : CHECK sur type_contrat et statut ────────────────────────

DROP TABLE IF EXISTS candidatures_new;

CREATE TABLE candidatures_new (
    id            TEXT PRIMARY KEY,
    entreprise_id TEXT NOT NULL REFERENCES entreprises(id) ON DELETE RESTRICT,
    contact_id    TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    poste         TEXT NOT NULL,
    type_contrat  TEXT NOT NULL DEFAULT 'CDI'
        CHECK (type_contrat IN ('CDI', 'CDD', 'Freelance', 'Stage', 'Alternance', 'Interim', 'Autre')),
    statut        TEXT NOT NULL DEFAULT 'EN_ATTENTE'
        CHECK (statut IN ('EN_ATTENTE', 'RELANCEE', 'ENTRETIEN', 'REFUS')),
    date_envoi    TEXT NOT NULL,
    lien_offre    TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- Normalisation des valeurs héritées : une base venue d'une version antérieure peut porter
-- des libellés absents des nouvelles listes. On les ramène sur la valeur de repli plutôt que
-- de perdre la ligne.
UPDATE candidatures SET type_contrat = 'Autre'
    WHERE type_contrat IS NULL OR type_contrat NOT IN
        ('CDI', 'CDD', 'Freelance', 'Stage', 'Alternance', 'Interim', 'Autre');
UPDATE candidatures SET statut = 'EN_ATTENTE'
    WHERE statut IS NULL OR statut NOT IN
        ('EN_ATTENTE', 'RELANCEE', 'ENTRETIEN', 'REFUS');

INSERT INTO candidatures_new SELECT * FROM candidatures;
DROP TABLE IF EXISTS candidatures;
ALTER TABLE candidatures_new RENAME TO candidatures;

CREATE INDEX IF NOT EXISTS idx_candidatures_entreprise ON candidatures(entreprise_id);
CREATE INDEX IF NOT EXISTS idx_candidatures_statut     ON candidatures(statut);

-- ── entretiens : CHECK sur type ───────────────────────────────────────────

DROP TABLE IF EXISTS entretiens_new;

CREATE TABLE entretiens_new (
    id                TEXT PRIMARY KEY,
    candidature_id    TEXT NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
    contact_id        TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    date_entretien    TEXT NOT NULL,
    type              TEXT NOT NULL DEFAULT 'Présentiel'
        CHECK (type IN ('Présentiel', 'Visio', 'Téléphonique', 'Technique', 'RH', 'Autre')),
    lieu              TEXT,
    notes             TEXT,
    compte_rendu      TEXT,
    calendar_event_id TEXT,
    analyse_ia        TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

UPDATE entretiens SET type = 'Autre'
    WHERE type IS NULL OR type NOT IN
        ('Présentiel', 'Visio', 'Téléphonique', 'Technique', 'RH', 'Autre');

INSERT INTO entretiens_new SELECT * FROM entretiens;
DROP TABLE IF EXISTS entretiens;
ALTER TABLE entretiens_new RENAME TO entretiens;

CREATE INDEX IF NOT EXISTS idx_entretiens_cand ON entretiens(candidature_id);
