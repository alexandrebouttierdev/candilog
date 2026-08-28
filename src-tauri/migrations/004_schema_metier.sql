-- Schéma métier local : reprise du modèle Supabase sans user_id ni table users.
-- Les identifiants sont des UUID en TEXT, générés côté Rust, comme les horodatages ISO 8601.
-- Utilise IF NOT EXISTS pour être idempotent.

CREATE TABLE IF NOT EXISTS entreprises (
    id         TEXT PRIMARY KEY,
    nom        TEXT NOT NULL,
    secteur    TEXT,
    type       TEXT,
    site_web   TEXT,
    ville      TEXT,
    adresse    TEXT,
    notes      TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contacts (
    id            TEXT PRIMARY KEY,
    entreprise_id TEXT REFERENCES entreprises(id) ON DELETE SET NULL,
    prenom        TEXT NOT NULL,
    nom           TEXT NOT NULL,
    poste         TEXT,
    email         TEXT,
    telephone     TEXT,
    linkedin      TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS candidatures (
    id            TEXT PRIMARY KEY,
    entreprise_id TEXT NOT NULL REFERENCES entreprises(id) ON DELETE RESTRICT,
    contact_id    TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    poste         TEXT NOT NULL,
    type_contrat  TEXT NOT NULL DEFAULT 'CDI',
    statut        TEXT NOT NULL DEFAULT 'EN_ATTENTE',
    date_envoi    TEXT NOT NULL,
    lien_offre    TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- Historique conservé depuis Supabase. Aucun code ne l'alimente aujourd'hui ;
-- la table existe pour ne pas perdre les lignes déjà accumulées.
CREATE TABLE IF NOT EXISTS statut_history (
    id             TEXT PRIMARY KEY,
    candidature_id TEXT NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
    statut         TEXT NOT NULL,
    changed_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS relances (
    id             TEXT PRIMARY KEY,
    candidature_id TEXT NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
    date_relance   TEXT NOT NULL,
    type           TEXT NOT NULL DEFAULT 'Email',
    notes          TEXT,
    created_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entretiens (
    id                TEXT PRIMARY KEY,
    candidature_id    TEXT NOT NULL REFERENCES candidatures(id) ON DELETE CASCADE,
    contact_id        TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    date_entretien    TEXT NOT NULL,
    type              TEXT NOT NULL DEFAULT 'Présentiel',
    lieu              TEXT,
    notes             TEXT,
    compte_rendu      TEXT,
    calendar_event_id TEXT,
    analyse_ia        TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cv_versions (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Tables singleton : une seule ligne, garantie par le CHECK.
CREATE TABLE IF NOT EXISTS parametres (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    data       TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS profil (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    data       TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_candidatures_entreprise ON candidatures(entreprise_id);
CREATE INDEX IF NOT EXISTS idx_candidatures_statut     ON candidatures(statut);
CREATE INDEX IF NOT EXISTS idx_contacts_entreprise     ON contacts(entreprise_id);
CREATE INDEX IF NOT EXISTS idx_statut_history_cand     ON statut_history(candidature_id);
CREATE INDEX IF NOT EXISTS idx_relances_cand           ON relances(candidature_id);
CREATE INDEX IF NOT EXISTS idx_entretiens_cand         ON entretiens(candidature_id);
