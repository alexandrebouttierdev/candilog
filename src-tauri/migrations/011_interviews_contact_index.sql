-- Index de la clé étrangère `interviews.contact_id`.
--
-- Le contrôle de suppression d'un contact (`contacts` : `count(*) FROM interviews WHERE
-- contact_id = ?1`), l'historique d'un contact et le `ON DELETE SET NULL` filtrent sur
-- cette colonne : sans index, chacun parcourait toute la table des entretiens.
CREATE INDEX IF NOT EXISTS idx_interviews_contact ON interviews(contact_id);
