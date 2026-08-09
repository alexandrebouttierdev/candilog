-- Tables locales : télémétrie LLM, historique ATS, cache IA, stockage clé/valeur.
CREATE TABLE IF NOT EXISTS llm_appels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,
    provider TEXT NOT NULL,
    modele TEXT NOT NULL,
    latence_ms INTEGER NOT NULL,
    succes INTEGER NOT NULL,
    cree_le TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scores_ats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    score INTEGER NOT NULL,
    origine TEXT NOT NULL,
    cree_le TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cache_ia (
    cle TEXT PRIMARY KEY,
    valeur TEXT NOT NULL,
    provider TEXT NOT NULL,
    modele TEXT NOT NULL,
    operation TEXT NOT NULL,
    cree_le TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_kv (
    cle TEXT PRIMARY KEY,
    valeur TEXT NOT NULL
);
