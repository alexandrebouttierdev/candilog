-- Bibliothèque locale des lettres de motivation générées.
CREATE TABLE IF NOT EXISTS lettres_motivation (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    company    TEXT,
    job_title  TEXT,
    tone       TEXT NOT NULL DEFAULT 'formal',
    length     TEXT NOT NULL DEFAULT 'medium',
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lettres_created_at
    ON lettres_motivation(created_at DESC);
