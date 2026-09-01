-- Mise en page de la lettre (template A4) : destinataire et référence d'offre.
-- Colonnes facultatives : une lettre enregistrée avant cette version reste lisible.

ALTER TABLE cover_letters ADD COLUMN recipient TEXT;
ALTER TABLE cover_letters ADD COLUMN recipient_address TEXT;
ALTER TABLE cover_letters ADD COLUMN job_reference TEXT;
