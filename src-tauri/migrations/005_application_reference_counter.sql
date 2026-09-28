-- Référence de candidature jamais réattribuée (correctif de la migration 3).
--
-- Le déclencheur de la migration 3 prenait `max(reference_number) + 1` : supprimer la
-- candidature la plus récente rendait son numéro à la suivante, et « CAN-012 » pouvait
-- désigner deux candidatures dans l'historique de l'utilisateur. Le dernier numéro attribué
-- est désormais retenu dans `app_kv` ; le déclencheur prend le plus grand de ce compteur et
-- des numéros présents (une restauration peut apporter des numéros plus élevés).
--
-- Une remise à zéro des données vide `app_kv` avec le reste : la numérotation repart alors
-- de 1, puisqu'aucune candidature ne subsiste pour être confondue.

INSERT OR REPLACE INTO app_kv (kv_key, kv_value)
SELECT 'last_application_reference', CAST(coalesce(max(reference_number), 0) AS TEXT)
FROM applications;

DROP TRIGGER IF EXISTS applications_assign_reference;

CREATE TRIGGER applications_assign_reference
AFTER INSERT ON applications
WHEN NEW.reference_number IS NULL
BEGIN
    INSERT OR REPLACE INTO app_kv (kv_key, kv_value)
    SELECT 'last_application_reference', CAST(max(
        coalesce(
            (SELECT CAST(kv_value AS INTEGER) FROM app_kv
             WHERE kv_key = 'last_application_reference'),
            0
        ),
        coalesce((SELECT max(reference_number) FROM applications), 0)
    ) + 1 AS TEXT);

    UPDATE applications
    SET reference_number = (
        SELECT CAST(kv_value AS INTEGER) FROM app_kv
        WHERE kv_key = 'last_application_reference'
    )
    WHERE id = NEW.id;
END;
