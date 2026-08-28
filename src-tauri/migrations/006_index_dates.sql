-- Accélère les fenêtres calendrier et les tris chronologiques paginés.
CREATE INDEX IF NOT EXISTS idx_candidatures_date ON candidatures(date_envoi);
CREATE INDEX IF NOT EXISTS idx_relances_date ON relances(date_relance);
CREATE INDEX IF NOT EXISTS idx_entretiens_date ON entretiens(date_entretien);
