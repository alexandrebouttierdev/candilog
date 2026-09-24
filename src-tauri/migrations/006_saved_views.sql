-- Vues enregistrées de Candidatures (refonte v2, section « Vues » de la navigation).
--
-- Une vue est un filtre nommé : il est stocké tel qu'envoyé au backend (`ApplicationFilter`
-- en JSON) et rejoué à l'ouverture. `position` fixe l'ordre de la navigation.

CREATE TABLE IF NOT EXISTS saved_views (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 60),
    filter     TEXT NOT NULL,
    position   INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
