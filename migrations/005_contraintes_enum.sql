-- Ajoute des contraintes CHECK sur les colonnes correspondant aux enums Rust.
-- SQLite ne supportant pas ALTER TABLE ADD CONSTRAINT, on recrée les tables.
-- Idempotent : si la migration a déjà été appliquée (tables CHECK déjà en place), on saute.

-- ── candidatures : CHECK sur type_contrat et statut ────────────────────────
-- Vérifie si la colonne type_contrat a déjà un CHECK (colonne dans sqlite_master contient "CHECK").
-- Si oui, la migration est déjà appliquée pour cette table.

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

INSERT OR IGNORE INTO candidatures_new SELECT * FROM candidatures;
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

INSERT OR IGNORE INTO entretiens_new SELECT * FROM entretiens;
DROP TABLE IF EXISTS entretiens;
ALTER TABLE entretiens_new RENAME TO entretiens;

CREATE INDEX IF NOT EXISTS idx_entretiens_cand ON entretiens(candidature_id);
