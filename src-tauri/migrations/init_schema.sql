-- Schéma SQLite local de Candilog.
-- Identifiants anglais en snake_case. Un seul fichier d'initialisation, idempotent
-- (`CREATE IF NOT EXISTS`). Les UUID et horodatages ISO 8601 sont générés côté Rust.
--
-- Les valeurs d'énumération stockées (`EN_ATTENTE`, `Présentiel`, `CDI`…) restent
-- celles contraintes par les enums Rust / serde.

-- ── Télémétrie locale et cache ──────────────────────────────────────────

CREATE TABLE IF NOT EXISTS llm_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    success INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ats_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    score INTEGER NOT NULL,
    origin TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_cache (
    cache_key TEXT PRIMARY KEY,
    cache_value TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    operation TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_kv (
    kv_key TEXT PRIMARY KEY,
    kv_value TEXT NOT NULL
);

-- ── Référentiel ─────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS sectors (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS companies (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    sector     TEXT,
    sector_id  TEXT REFERENCES sectors(id) ON DELETE SET NULL,
    type       TEXT,
    website    TEXT,
    city       TEXT,
    address    TEXT,
    notes      TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contacts (
    id            TEXT PRIMARY KEY,
    company_id    TEXT REFERENCES companies(id) ON DELETE SET NULL,
    first_name    TEXT NOT NULL,
    name          TEXT NOT NULL,
    job_title     TEXT,
    email         TEXT,
    phone         TEXT,
    linkedin      TEXT,
    notes         TEXT,
    tracking_role TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS applications (
    id            TEXT PRIMARY KEY,
    company_id    TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    contact_id    TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    job_title     TEXT NOT NULL,
    contract_type TEXT NOT NULL DEFAULT 'CDI'
        CHECK (contract_type IN ('CDI', 'CDD', 'Freelance', 'Stage', 'Alternance', 'Interim', 'Autre')),
    status        TEXT NOT NULL DEFAULT 'EN_ATTENTE'
        CHECK (status IN ('EN_ATTENTE', 'RELANCEE', 'ENTRETIEN', 'REFUS')),
    sent_date     TEXT NOT NULL,
    job_url       TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS status_history (
    id             TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    status         TEXT NOT NULL,
    changed_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS follow_ups (
    id             TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    follow_up_date TEXT NOT NULL,
    type           TEXT NOT NULL DEFAULT 'Email',
    notes          TEXT,
    created_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS interviews (
    id                TEXT PRIMARY KEY,
    application_id    TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    contact_id        TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    interview_date    TEXT NOT NULL,
    type              TEXT NOT NULL DEFAULT 'Présentiel'
        CHECK (type IN ('Présentiel', 'Visio', 'Téléphonique', 'Technique', 'RH', 'Autre')),
    location          TEXT,
    notes             TEXT,
    minutes           TEXT,
    calendar_event_id TEXT,
    ai_analysis       TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS resume_versions (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cover_letters (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    company    TEXT,
    job_title  TEXT,
    tone       TEXT NOT NULL DEFAULT 'formal',
    length     TEXT NOT NULL DEFAULT 'medium',
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    data       TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS profile (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    data       TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_applications_company ON applications(company_id);
CREATE INDEX IF NOT EXISTS idx_applications_status ON applications(status);
CREATE INDEX IF NOT EXISTS idx_applications_date ON applications(sent_date);
CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company_id);
CREATE INDEX IF NOT EXISTS idx_status_history_application ON status_history(application_id);
CREATE INDEX IF NOT EXISTS idx_follow_ups_application ON follow_ups(application_id);
CREATE INDEX IF NOT EXISTS idx_follow_ups_date ON follow_ups(follow_up_date);
CREATE INDEX IF NOT EXISTS idx_interviews_application ON interviews(application_id);
CREATE INDEX IF NOT EXISTS idx_interviews_date ON interviews(interview_date);
CREATE INDEX IF NOT EXISTS idx_companies_sector_id ON companies(sector_id);
CREATE INDEX IF NOT EXISTS idx_cover_letters_created_at ON cover_letters(created_at DESC);
