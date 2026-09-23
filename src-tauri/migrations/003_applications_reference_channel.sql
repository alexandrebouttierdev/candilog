-- Référence lisible et canal d'une candidature (refonte v2).
--
-- `reference_number` porte la référence affichée `CAN-142` : un numéro croissant, stable,
-- attribué à la création et jamais réutilisé après une suppression. Les candidatures
-- existantes sont numérotées dans leur ordre de création.
--
-- `channel` dit par où l'offre a été trouvée : offre publiée, site de l'entreprise, réseau,
-- ou démarche spontanée. Une candidature spontanée existante devient `SPONTANEOUS`, toutes
-- les autres `OFFER` — c'est ce qu'elles étaient.

ALTER TABLE applications ADD COLUMN reference_number INTEGER;

ALTER TABLE applications ADD COLUMN channel TEXT NOT NULL DEFAULT 'OFFER'
    CHECK (channel IN ('OFFER', 'COMPANY_SITE', 'NETWORK', 'SPONTANEOUS'));

UPDATE applications SET channel = 'SPONTANEOUS' WHERE application_type = 'SPONTANEE';

UPDATE applications
SET reference_number = (
    SELECT count(*)
    FROM applications AS earlier
    WHERE earlier.created_at < applications.created_at
       OR (earlier.created_at = applications.created_at AND earlier.id <= applications.id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_reference
    ON applications(reference_number);

-- Toute insertion reçoit le numéro suivant, quel que soit son chemin (écran, import,
-- restauration) : le dépôt n'a pas à y penser, et deux écritures concurrentes ne peuvent
-- pas obtenir le même numéro puisque SQLite sérialise les transactions d'écriture.
CREATE TRIGGER IF NOT EXISTS applications_assign_reference
AFTER INSERT ON applications
WHEN NEW.reference_number IS NULL
BEGIN
    UPDATE applications
    SET reference_number = (
        SELECT coalesce(max(reference_number), 0) + 1 FROM applications
    )
    WHERE id = NEW.id;
END;
