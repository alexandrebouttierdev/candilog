-- Synchronisation entre ordinateurs (docs/SYNC.md).
--
-- Le stockage local reste la source de vérité : ces tables ne décrivent que ce qui doit
-- partir vers l'espace distant et ce qui en est déjà connu. Aucune table métier n'est
-- modifiée.
--
-- `sync_state`     état opérationnel de la synchronisation (installation, espace, curseurs
--                  locaux, fournisseur choisi). Propre à cet ordinateur ; vidé par une
--                  restauration de sauvegarde ou une remise à zéro.
-- `sync_flags`     `tracking` : l'ordinateur est rattaché à un espace et journalise ses
--                  modifications ; `applying` : posé le temps d'appliquer des changements
--                  distants dans une transaction, pour ne pas les renvoyer en écho.
-- `sync_outbox`    journal des enregistrements modifiés localement, alimenté par les
--                  déclencheurs **dans la transaction de l'écriture métier** : une
--                  modification validée est toujours journalisée, et inversement.
-- `sync_records`   version connue de chaque enregistrement synchronisé (vecteurs de
--                  versions par colonne), contenu de référence et marqueurs de suppression.
-- `sync_cursors`   dernier lot appliqué pour chaque autre installation.
-- `sync_pending`   lot chiffré préparé mais pas encore confirmé par le fournisseur.
-- `sync_conflicts` conflits en attente d'une décision de l'utilisateur.
-- `sync_deferred`  changements reçus dont un enregistrement parent n'est pas encore arrivé.

CREATE TABLE IF NOT EXISTS sync_state (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_flags (
    name TEXT PRIMARY KEY CHECK (name IN ('tracking', 'applying'))
);

CREATE TABLE IF NOT EXISTS sync_outbox (
    seq       INTEGER PRIMARY KEY AUTOINCREMENT,
    entity    TEXT NOT NULL,
    record_id TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_outbox_record ON sync_outbox(entity, record_id);

CREATE TABLE IF NOT EXISTS sync_records (
    entity     TEXT NOT NULL,
    record_id  TEXT NOT NULL,
    version    TEXT NOT NULL,
    columns    TEXT NOT NULL,
    base       TEXT,
    forced     TEXT NOT NULL DEFAULT '[]',
    deleted    INTEGER NOT NULL DEFAULT 0 CHECK (deleted IN (0, 1)),
    updated_at TEXT NOT NULL,
    PRIMARY KEY (entity, record_id)
);

CREATE TABLE IF NOT EXISTS sync_cursors (
    installation_id TEXT PRIMARY KEY,
    last_seq        INTEGER NOT NULL CHECK (last_seq >= 0)
);

CREATE TABLE IF NOT EXISTS sync_pending (
    seq         INTEGER PRIMARY KEY,
    object_name TEXT NOT NULL,
    payload     BLOB NOT NULL,
    blobs       TEXT NOT NULL DEFAULT '[]',
    records     INTEGER NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_conflicts (
    id          TEXT PRIMARY KEY,
    entity      TEXT NOT NULL,
    record_id   TEXT NOT NULL,
    kind        TEXT NOT NULL
        CHECK (kind IN ('edit', 'deleted_remotely', 'deleted_locally')),
    remote      TEXT NOT NULL,
    fields      TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    UNIQUE (entity, record_id)
);

CREATE TABLE IF NOT EXISTS sync_deferred (
    entity      TEXT NOT NULL,
    record_id   TEXT NOT NULL,
    change      TEXT NOT NULL,
    received_at TEXT NOT NULL,
    PRIMARY KEY (entity, record_id)
);

-- Un déclencheur par table et par opération. Les suppressions en cascade et les `SET NULL`
-- des clés étrangères déclenchent eux aussi ces déclencheurs : la suppression d'une
-- candidature journalise donc ses relances, entretiens et statuts dans la même transaction.

CREATE TRIGGER IF NOT EXISTS sync_profile_insert
AFTER INSERT ON profile
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('profile', CAST(NEW.id AS TEXT));
END;

CREATE TRIGGER IF NOT EXISTS sync_profile_update
AFTER UPDATE ON profile
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('profile', CAST(NEW.id AS TEXT));
END;

CREATE TRIGGER IF NOT EXISTS sync_profile_delete
AFTER DELETE ON profile
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('profile', CAST(OLD.id AS TEXT));
END;

CREATE TRIGGER IF NOT EXISTS sync_companies_insert
AFTER INSERT ON companies
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('companies', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_companies_update
AFTER UPDATE ON companies
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('companies', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_companies_delete
AFTER DELETE ON companies
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('companies', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_contacts_insert
AFTER INSERT ON contacts
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('contacts', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_contacts_update
AFTER UPDATE ON contacts
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('contacts', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_contacts_delete
AFTER DELETE ON contacts
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('contacts', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_applications_insert
AFTER INSERT ON applications
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('applications', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_applications_update
AFTER UPDATE ON applications
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('applications', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_applications_delete
AFTER DELETE ON applications
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('applications', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_status_history_insert
AFTER INSERT ON status_history
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('status_history', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_status_history_update
AFTER UPDATE ON status_history
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('status_history', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_status_history_delete
AFTER DELETE ON status_history
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('status_history', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_follow_ups_insert
AFTER INSERT ON follow_ups
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('follow_ups', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_follow_ups_update
AFTER UPDATE ON follow_ups
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('follow_ups', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_follow_ups_delete
AFTER DELETE ON follow_ups
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('follow_ups', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_interviews_insert
AFTER INSERT ON interviews
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('interviews', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_interviews_update
AFTER UPDATE ON interviews
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('interviews', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_interviews_delete
AFTER DELETE ON interviews
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('interviews', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_relation_notes_insert
AFTER INSERT ON relation_notes
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('relation_notes', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_relation_notes_update
AFTER UPDATE ON relation_notes
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('relation_notes', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_relation_notes_delete
AFTER DELETE ON relation_notes
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('relation_notes', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_resume_versions_insert
AFTER INSERT ON resume_versions
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('resume_versions', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_resume_versions_update
AFTER UPDATE ON resume_versions
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('resume_versions', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_resume_versions_delete
AFTER DELETE ON resume_versions
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('resume_versions', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_cover_letters_insert
AFTER INSERT ON cover_letters
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('cover_letters', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_cover_letters_update
AFTER UPDATE ON cover_letters
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('cover_letters', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_cover_letters_delete
AFTER DELETE ON cover_letters
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('cover_letters', OLD.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_saved_views_insert
AFTER INSERT ON saved_views
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('saved_views', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_saved_views_update
AFTER UPDATE ON saved_views
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('saved_views', NEW.id);
END;

CREATE TRIGGER IF NOT EXISTS sync_saved_views_delete
AFTER DELETE ON saved_views
WHEN EXISTS (SELECT 1 FROM sync_flags WHERE name = 'tracking')
 AND NOT EXISTS (SELECT 1 FROM sync_flags WHERE name = 'applying')
BEGIN
    INSERT INTO sync_outbox (entity, record_id) VALUES ('saved_views', OLD.id);
END;
