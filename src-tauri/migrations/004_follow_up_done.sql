-- Relance marquée faite (refonte v2, écran Aujourd'hui : « Faire »).
--
-- `done_at` est l'horodatage où l'utilisateur a déclaré la relance envoyée. `NULL` = encore à
-- faire. Une relance faite ne compte plus comme « en retard » et ne s'affiche plus comme la
-- prochaine échéance d'une candidature ; elle reste dans l'historique et au calendrier.

ALTER TABLE follow_ups ADD COLUMN done_at TEXT;
