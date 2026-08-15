PRAGMA foreign_keys = ON;
BEGIN IMMEDIATE;

DELETE FROM cache_ia;
DELETE FROM llm_appels;
DELETE FROM scores_ats;
DELETE FROM lettres_motivation;
DELETE FROM entretiens;
DELETE FROM relances;
DELETE FROM statut_history;
DELETE FROM candidatures;
DELETE FROM contacts;
DELETE FROM entreprises;
DELETE FROM profil;
DELETE FROM parametres;

INSERT INTO parametres (id, data, updated_at) VALUES (
    1,
    '{"llm":{"provider":"ollama","api_key":null,"endpoint":"http://localhost:11434","model":"llama3.2:3b","temperature":0.7,"mode":"auto"},"theme":"dark","langue":"fr"}',
    '2026-08-15T08:00:00Z'
);

INSERT INTO profil (id, data, updated_at) VALUES (
    1,
    '{"personal":{"first_name":"Camille","last_name":"Moreau","email":"camille.moreau@example.test","phone":"06 00 00 00 00","city":"Nantes","headline":"Cheffe de projet digital","summary":"Cheffe de projet spécialisée dans la coordination de produits numériques, la recherche utilisateur et la collaboration entre équipes métier et design.","linkedin":"linkedin.com/in/camille-moreau-demo","github":null,"website":"camille-portfolio.example"},"experiences":[{"title":"Cheffe de projet digital","company":"Studio Nébula","location":"Nantes","start_date":"2023-09","end_date":null,"current":true,"description":"Pilotage de projets numériques, animation des ateliers et suivi des indicateurs produit."},{"title":"Chargée de communication","company":"Maison Sépia","location":"Bordeaux","start_date":"2021-03","end_date":"2023-08","current":false,"description":"Coordination des campagnes, production de contenus et analyse des performances."},{"title":"Assistante cheffe de projet","company":"Atelier Lumen","location":"Rennes","start_date":"2019-09","end_date":"2021-02","current":false,"description":"Préparation des ateliers, suivi des livrables et relation avec les partenaires."}],"skills":[{"name":"Gestion de projet"},{"name":"Méthodes agiles"},{"name":"Recherche utilisateur"},{"name":"Analyse de données"},{"name":"Animation atelier"},{"name":"Figma"},{"name":"Notion"},{"name":"Communication"}],"education":[{"degree":"Master stratégie digitale","school":"Institut Mercure","location":"Nantes","start_date":"2017","end_date":"2019","description":"Stratégie produit, expérience utilisateur et conduite du changement."},{"degree":"Licence information et communication","school":"Campus Atlantique","location":"La Rochelle","start_date":"2014","end_date":"2017","description":null}],"languages":[{"name":"Français","level":"Natif"},{"name":"Anglais","level":"C1"},{"name":"Espagnol","level":"B1"}],"projects":[{"name":"Refonte espace client","description":"Coordination de la refonte et des tests utilisateurs.","url":"https://demo.example/projet-espace-client","technologies":"Figma, Notion, Analytics"},{"name":"Design system éditorial","description":"Mise en place des règles et composants partagés.","url":null,"technologies":"Figma, Documentation"}],"certifications":[{"name":"Product discovery","issuer":"Academy Demo","date":"2025-11","url":null},{"name":"Agile product management","issuer":"Institut Mercure","date":"2024-06","url":null}]}',
    '2026-08-15T08:00:00Z'
);

INSERT INTO entreprises (id, nom, secteur, type, site_web, ville, adresse, notes, created_at, updated_at) VALUES
('10000000-0000-4000-8000-000000000001','Boussole Labs','Logiciels B2B','Startup','https://boussole-labs.example','Nantes',NULL,'Entreprise entièrement fictive pour la démonstration.','2026-06-02T09:00:00Z','2026-08-14T09:00:00Z'),
('10000000-0000-4000-8000-000000000002','Orbite Conseil','Conseil produit','PME','https://orbite-conseil.example','Rennes',NULL,'Données de démonstration.','2026-06-09T09:00:00Z','2026-08-13T09:00:00Z'),
('10000000-0000-4000-8000-000000000003','Nacre Studio','Design numérique','Agence','https://nacre-studio.example','Bordeaux',NULL,'Données de démonstration.','2026-06-18T09:00:00Z','2026-08-12T09:00:00Z'),
('10000000-0000-4000-8000-000000000004','Kumo Digital','Commerce numérique','PME','https://kumo-digital.example','Lyon',NULL,'Données de démonstration.','2026-07-01T09:00:00Z','2026-08-11T09:00:00Z'),
('10000000-0000-4000-8000-000000000005','Élan Produit','Services numériques','Startup','https://elan-produit.example','Paris',NULL,'Données de démonstration.','2026-07-04T09:00:00Z','2026-08-10T09:00:00Z'),
('10000000-0000-4000-8000-000000000006','Studio Nébula','Communication','Agence','https://studio-nebula.example','Nantes',NULL,'Données de démonstration.','2026-07-08T09:00:00Z','2026-08-09T09:00:00Z'),
('10000000-0000-4000-8000-000000000007','Maison Sépia','Édition','PME','https://maison-sepia.example','Bordeaux',NULL,'Données de démonstration.','2026-07-12T09:00:00Z','2026-08-08T09:00:00Z'),
('10000000-0000-4000-8000-000000000008','Horizon Démo','Média','Association','https://horizon-demo.example','Angers',NULL,'Données de démonstration.','2026-07-16T09:00:00Z','2026-08-07T09:00:00Z');

INSERT INTO contacts (id, entreprise_id, prenom, nom, poste, email, telephone, linkedin, notes, created_at, updated_at) VALUES
('20000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000001','Léa','Bernard','Responsable produit','lea.bernard@example.test','06 00 00 00 01','linkedin.com/in/lea-bernard-demo','Contact fictif.','2026-07-10T10:00:00Z','2026-08-14T10:00:00Z'),
('20000000-0000-4000-8000-000000000002','10000000-0000-4000-8000-000000000002','Hugo','Martin','Consultant senior','hugo.martin@example.test','06 00 00 00 02','linkedin.com/in/hugo-martin-demo','Contact fictif.','2026-07-14T10:00:00Z','2026-08-13T10:00:00Z'),
('20000000-0000-4000-8000-000000000003','10000000-0000-4000-8000-000000000003','Inès','Roux','Directrice de création','ines.roux@example.test','06 00 00 00 03','linkedin.com/in/ines-roux-demo','Contact fictif.','2026-07-18T10:00:00Z','2026-08-12T10:00:00Z'),
('20000000-0000-4000-8000-000000000004','10000000-0000-4000-8000-000000000004','Noé','Petit','Talent partner','noe.petit@example.test','06 00 00 00 04','linkedin.com/in/noe-petit-demo','Contact fictif.','2026-07-22T10:00:00Z','2026-08-11T10:00:00Z'),
('20000000-0000-4000-8000-000000000005','10000000-0000-4000-8000-000000000005','Sarah','Lopez','Head of product','sarah.lopez@example.test','06 00 00 00 05','linkedin.com/in/sarah-lopez-demo','Contact fictif.','2026-07-26T10:00:00Z','2026-08-10T10:00:00Z');

INSERT INTO candidatures (id, entreprise_id, contact_id, poste, type_contrat, statut, date_envoi, lien_offre, notes, created_at, updated_at) VALUES
('30000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000001','20000000-0000-4000-8000-000000000001','Product owner','CDI','ENTRETIEN','2026-08-04','https://jobs.example/product-owner','Dossier de démonstration.','2026-08-04T08:30:00Z','2026-08-14T16:20:00Z'),
('30000000-0000-4000-8000-000000000002','10000000-0000-4000-8000-000000000002','20000000-0000-4000-8000-000000000002','Cheffe de projet digital','CDI','RELANCEE','2026-08-06','https://jobs.example/chef-projet','Relance envoyée.','2026-08-06T09:10:00Z','2026-08-13T11:40:00Z'),
('30000000-0000-4000-8000-000000000003','10000000-0000-4000-8000-000000000003','20000000-0000-4000-8000-000000000003','Consultante UX','CDD','EN_ATTENTE','2026-08-12','https://jobs.example/consultante-ux','Portfolio joint.','2026-08-12T07:45:00Z','2026-08-12T07:45:00Z'),
('30000000-0000-4000-8000-000000000004','10000000-0000-4000-8000-000000000004','20000000-0000-4000-8000-000000000004','Responsable e-commerce','CDI','EN_ATTENTE','2026-08-11','https://jobs.example/ecommerce','Candidature spontanée.','2026-08-11T13:20:00Z','2026-08-11T13:20:00Z'),
('30000000-0000-4000-8000-000000000005','10000000-0000-4000-8000-000000000005','20000000-0000-4000-8000-000000000005','Product manager junior','CDI','ENTRETIEN','2026-07-28','https://jobs.example/product-manager','Entretien RH planifié.','2026-07-28T10:00:00Z','2026-08-14T08:00:00Z'),
('30000000-0000-4000-8000-000000000006','10000000-0000-4000-8000-000000000006',NULL,'Chargée de projet web','CDD','RELANCEE','2026-07-30','https://jobs.example/projet-web',NULL,'2026-07-30T15:00:00Z','2026-08-08T09:00:00Z'),
('30000000-0000-4000-8000-000000000007','10000000-0000-4000-8000-000000000007',NULL,'Coordinatrice éditoriale','CDI','REFUS','2026-07-21','https://jobs.example/editorial',NULL,'2026-07-21T12:00:00Z','2026-08-05T09:00:00Z'),
('30000000-0000-4000-8000-000000000008','10000000-0000-4000-8000-000000000008',NULL,'Responsable communication','CDD','REFUS','2026-07-18','https://jobs.example/communication',NULL,'2026-07-18T12:00:00Z','2026-08-02T09:00:00Z'),
('30000000-0000-4000-8000-000000000009','10000000-0000-4000-8000-000000000001',NULL,'Customer success manager','CDI','EN_ATTENTE','2026-08-13','https://jobs.example/customer-success',NULL,'2026-08-13T09:00:00Z','2026-08-13T09:00:00Z'),
('30000000-0000-4000-8000-000000000010','10000000-0000-4000-8000-000000000002',NULL,'Consultante transformation','Freelance','REFUS','2026-07-12','https://jobs.example/transformation',NULL,'2026-07-12T09:00:00Z','2026-07-29T09:00:00Z'),
('30000000-0000-4000-8000-000000000011','10000000-0000-4000-8000-000000000003',NULL,'UX project manager','CDI','ENTRETIEN','2026-08-01','https://jobs.example/ux-project',NULL,'2026-08-01T09:00:00Z','2026-08-12T09:00:00Z'),
('30000000-0000-4000-8000-000000000012','10000000-0000-4000-8000-000000000004',NULL,'Content manager','CDI','RELANCEE','2026-07-25','https://jobs.example/content-manager',NULL,'2026-07-25T09:00:00Z','2026-08-07T09:00:00Z'),
('30000000-0000-4000-8000-000000000013','10000000-0000-4000-8000-000000000005',NULL,'Scrum master','CDI','REFUS','2026-07-08','https://jobs.example/scrum-master',NULL,'2026-07-08T09:00:00Z','2026-07-24T09:00:00Z'),
('30000000-0000-4000-8000-000000000014','10000000-0000-4000-8000-000000000006',NULL,'Coordinatrice marketing','CDD','EN_ATTENTE','2026-08-14','https://jobs.example/marketing',NULL,'2026-08-14T09:00:00Z','2026-08-14T09:00:00Z');

INSERT INTO relances (id, candidature_id, date_relance, type, notes, created_at) VALUES
('40000000-0000-4000-8000-000000000001','30000000-0000-4000-8000-000000000002','2026-08-17','Email','Demander un retour sur le dossier.','2026-08-13T11:40:00Z'),
('40000000-0000-4000-8000-000000000002','30000000-0000-4000-8000-000000000006','2026-08-19','LinkedIn','Relance courte auprès du recruteur.','2026-08-08T09:00:00Z'),
('40000000-0000-4000-8000-000000000003','30000000-0000-4000-8000-000000000012','2026-08-20','Téléphone','Vérifier la disponibilité du poste.','2026-08-07T09:00:00Z'),
('40000000-0000-4000-8000-000000000004','30000000-0000-4000-8000-000000000003','2026-08-24','Email','Envoyer un exemple de projet.','2026-08-12T07:45:00Z');

INSERT INTO entretiens (id, candidature_id, contact_id, date_entretien, type, lieu, notes, compte_rendu, calendar_event_id, analyse_ia, created_at, updated_at) VALUES
('50000000-0000-4000-8000-000000000001','30000000-0000-4000-8000-000000000001','20000000-0000-4000-8000-000000000001','2026-08-18T10:00:00+02:00','Visio','https://meet.example/demo','Préparer deux exemples de priorisation produit.',NULL,NULL,NULL,'2026-08-14T16:20:00Z','2026-08-14T16:20:00Z'),
('50000000-0000-4000-8000-000000000002','30000000-0000-4000-8000-000000000005','20000000-0000-4000-8000-000000000005','2026-08-21T14:30:00+02:00','RH','Bureau Nantes','Clarifier les attentes du poste.',NULL,NULL,NULL,'2026-08-14T08:00:00Z','2026-08-14T08:00:00Z'),
('50000000-0000-4000-8000-000000000003','30000000-0000-4000-8000-000000000011','20000000-0000-4000-8000-000000000003','2026-08-12T09:30:00+02:00','Technique','Studio Nacre','Présenter le projet de refonte.','Échange clair et concret. La partie recherche utilisateur a suscité plusieurs questions positives.',NULL,'{"resume":"Entretien positif avec un bon échange sur la méthode de travail.","points_forts":["Exemples concrets","Bonne capacité de synthèse"],"points_faibles":["Préciser les indicateurs de succès"],"suggestions":["Préparer un exemple chiffré pour le prochain échange"]}','2026-08-12T11:00:00Z','2026-08-12T11:20:00Z');

INSERT INTO cv_versions (id, name, content, created_at) VALUES
('60000000-0000-4000-8000-000000000001','CV Product owner · Boussole Labs','{"cv":{"summary":"Cheffe de projet digital avec six années de coordination de produits numériques, de recherche utilisateur et de collaboration transverse.","experiences":[{"title":"Cheffe de projet digital","company":"Studio Nébula","description":"Pilotage de projets numériques, animation des ateliers de cadrage et suivi des indicateurs produit."},{"title":"Chargée de communication","company":"Maison Sépia","description":"Coordination de campagnes multicanales et analyse des performances."}],"skills":["Gestion de projet","Méthodes agiles","Recherche utilisateur","Analyse de données","Figma"],"education":[{"degree":"Master stratégie digitale","school":"Institut Mercure"}]},"analysis":{"score":86,"recap":"Le profil répond aux principaux attendus du poste.","suggestions":["Préciser les résultats des projets","Mettre en avant la priorisation"],"recommandations":[{"section":"resume","texte_original":"Cheffe de projet digital.","texte_propose":"Cheffe de projet digital spécialisée en pilotage produit et recherche utilisateur.","impact":8}]}}','2026-08-14T17:00:00Z'),
('60000000-0000-4000-8000-000000000002','CV Cheffe de projet · Orbite Conseil','{"cv":{"summary":"Cheffe de projet expérimentée en coordination transverse et conduite du changement.","experiences":[{"title":"Cheffe de projet digital","company":"Studio Nébula","description":"Coordination des équipes et suivi des livrables."}],"skills":["Gestion de projet","Animation atelier","Communication"],"education":[{"degree":"Master stratégie digitale","school":"Institut Mercure"}]},"analysis":{"score":78,"recap":"Bonne adéquation générale.","suggestions":["Ajouter un résultat mesurable"],"recommandations":[]}}','2026-08-10T15:00:00Z');

INSERT INTO lettres_motivation (id, name, company, job_title, tone, length, content, created_at) VALUES
('70000000-0000-4000-8000-000000000001','Boussole Labs · Product owner','Boussole Labs','Product owner','formal','medium','Madame, Monsieur,\n\nVotre offre de Product owner a retenu toute mon attention. Mon expérience en coordination de produits numériques et en recherche utilisateur correspond aux enjeux présentés.\n\nCordialement,\nCamille Moreau','2026-08-14T17:15:00Z'),
('70000000-0000-4000-8000-000000000002','Orbite Conseil · Cheffe de projet','Orbite Conseil','Cheffe de projet digital','casual','medium','Bonjour,\n\nJe souhaite vous proposer ma candidature au poste de Cheffe de projet digital. Mon parcours repose sur la coordination transverse et la conduite de projets centrés utilisateur.\n\nBien cordialement,\nCamille Moreau','2026-08-10T15:20:00Z'),
('70000000-0000-4000-8000-000000000003','Nacre Studio · UX project','Nacre Studio','UX project manager','formal','short','Madame, Monsieur,\n\nJe serais heureuse de mettre mon expérience de coordination et de recherche utilisateur au service de votre studio.\n\nCordialement,\nCamille Moreau','2026-08-07T12:00:00Z');

INSERT INTO scores_ats (score, origine, cree_le) VALUES
(86,'genere','2026-08-14T17:00:00Z'),
(78,'genere','2026-08-10T15:00:00Z'),
(72,'importe','2026-08-09T11:00:00Z'),
(84,'genere','2026-08-06T16:00:00Z'),
(67,'importe','2026-08-03T10:00:00Z'),
(89,'genere','2026-07-30T14:00:00Z'),
(74,'genere','2026-07-25T09:00:00Z'),
(61,'importe','2026-07-20T13:00:00Z');

INSERT INTO llm_appels (operation, provider, modele, latence_ms, succes, cree_le) VALUES
('parse_offer','ollama','llama3.2:3b',1840,1,'2026-08-14T16:50:00Z'),
('generate_cv','ollama','llama3.2:3b',6420,1,'2026-08-14T16:56:00Z'),
('analyze_ats','ollama','llama3.2:3b',2380,1,'2026-08-14T17:00:00Z'),
('cover_letter','ollama','llama3.2:3b',4920,1,'2026-08-14T17:15:00Z'),
('parse_cv','ollama','llama3.2:3b',3160,1,'2026-08-09T10:56:00Z'),
('analyser_entretien','ollama','llama3.2:3b',2740,1,'2026-08-12T11:20:00Z'),
('parse_offer','ollama','llama3.2:3b',1710,1,'2026-08-10T14:50:00Z'),
('generate_cv','ollama','llama3.2:3b',6150,1,'2026-08-10T15:00:00Z');

COMMIT;

PRAGMA integrity_check;
