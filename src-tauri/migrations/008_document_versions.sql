-- Versions des documents (refonte v2, inspecteur Documents, section « Versions »).
--
-- Un document est la suite de ses enregistrements : chaque ligne de `resume_versions` ou de
-- `cover_letters` est une version, `document_id` les relie (l'identifiant de la première),
-- `version_number` les numérote (v1, v2…) et `is_current` désigne celle que la bibliothèque
-- affiche. Restaurer une version déplace `is_current` : aucune version n'est effacée.
-- Chaque document existant devient sa propre v1, courante.

ALTER TABLE resume_versions ADD COLUMN document_id TEXT;
ALTER TABLE resume_versions ADD COLUMN version_number INTEGER NOT NULL DEFAULT 1
    CHECK (version_number >= 1);
ALTER TABLE resume_versions ADD COLUMN is_current INTEGER NOT NULL DEFAULT 1
    CHECK (is_current IN (0, 1));
ALTER TABLE resume_versions ADD COLUMN version_note TEXT;
UPDATE resume_versions SET document_id = id WHERE document_id IS NULL;
CREATE INDEX IF NOT EXISTS idx_resume_versions_document
    ON resume_versions(document_id, version_number);
CREATE UNIQUE INDEX IF NOT EXISTS idx_resume_versions_current
    ON resume_versions(document_id) WHERE is_current = 1;

ALTER TABLE cover_letters ADD COLUMN document_id TEXT;
ALTER TABLE cover_letters ADD COLUMN version_number INTEGER NOT NULL DEFAULT 1
    CHECK (version_number >= 1);
ALTER TABLE cover_letters ADD COLUMN is_current INTEGER NOT NULL DEFAULT 1
    CHECK (is_current IN (0, 1));
ALTER TABLE cover_letters ADD COLUMN version_note TEXT;
UPDATE cover_letters SET document_id = id WHERE document_id IS NULL;
CREATE INDEX IF NOT EXISTS idx_cover_letters_document
    ON cover_letters(document_id, version_number);
CREATE UNIQUE INDEX IF NOT EXISTS idx_cover_letters_current
    ON cover_letters(document_id) WHERE is_current = 1;
