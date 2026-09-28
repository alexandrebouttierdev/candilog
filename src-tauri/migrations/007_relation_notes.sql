-- Notes d'historique des relations (refonte v2, inspecteur de Relations, « Note »).
--
-- Une note datée appartient à une entreprise ou à un contact, jamais aux deux : elle suit
-- sa fiche et disparaît avec elle. Le reste de l'historique (candidatures, statuts,
-- entretiens, relances) est lu dans les tables existantes, jamais recopié ici.

CREATE TABLE IF NOT EXISTS relation_notes (
    id         TEXT PRIMARY KEY,
    company_id TEXT REFERENCES companies(id) ON DELETE CASCADE,
    contact_id TEXT REFERENCES contacts(id) ON DELETE CASCADE,
    body       TEXT NOT NULL CHECK (length(trim(body)) BETWEEN 1 AND 2000),
    noted_on   TEXT NOT NULL,
    created_at TEXT NOT NULL,
    CHECK ((company_id IS NULL) <> (contact_id IS NULL))
);

CREATE INDEX IF NOT EXISTS idx_relation_notes_company ON relation_notes(company_id);
CREATE INDEX IF NOT EXISTS idx_relation_notes_contact ON relation_notes(contact_id);
