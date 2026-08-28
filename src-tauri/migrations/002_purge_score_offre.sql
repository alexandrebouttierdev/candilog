-- La fonctionnalité « score d'offre » a été retirée : purge de sa télémétrie.
DELETE FROM llm_appels WHERE operation = 'score_offre';
