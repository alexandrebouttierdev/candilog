-- Ajoute le stage au référentiel des contrats pour les bases déjà installées.
INSERT OR IGNORE INTO contract_types (code, name, sort_order, created_at)
VALUES ('STAGE', 'Stage', 23, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
