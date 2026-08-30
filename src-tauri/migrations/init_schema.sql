-- Schéma SQLite local de Candilog — source de vérité unique et complète.
--
-- Un seul fichier d'initialisation, idempotent (`CREATE IF NOT EXISTS`,
-- `INSERT OR IGNORE`) : il crée les tables, les index et les référentiels métier.
-- Aucune migration héritée n'est conservée ; une base neuve obtient directement le
-- modèle final.
--
-- Identifiants techniques en anglais, `snake_case`. Les libellés métier restent en
-- français : ce sont eux que l'interface affiche, jamais les codes.
--
-- Les UUID et horodatages ISO 8601 des données utilisateur sont générés côté Rust ;
-- seuls les référentiels sont semés ici, avec des identifiants stables entre
-- installations.

-- ── Télémétrie locale et cache ──────────────────────────────────────────

CREATE TABLE IF NOT EXISTS llm_calls (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    operation  TEXT NOT NULL,
    provider   TEXT NOT NULL,
    model      TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    success    INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ats_scores (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    score      INTEGER NOT NULL,
    origin     TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_cache (
    cache_key   TEXT PRIMARY KEY,
    cache_value TEXT NOT NULL,
    provider    TEXT NOT NULL,
    model       TEXT NOT NULL,
    operation   TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_kv (
    kv_key   TEXT PRIMARY KEY,
    kv_value TEXT NOT NULL
);

-- ── Référentiels ────────────────────────────────────────────────────────
--
-- Quatre catalogues distincts, jamais fusionnés : le secteur qualifie l'activité de
-- l'entreprise, le domaine professionnel qualifie le poste visé, le type d'entreprise
-- qualifie la nature de l'organisation, et le type de contrat qualifie l'engagement.

-- Secteur d'activité de l'entreprise.
CREATE TABLE IF NOT EXISTS sectors (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Domaine professionnel du poste. Le code métier sert de clé primaire : générer un UUID
-- pour une valeur déjà identifiante ajouterait une indirection sans rien apporter.
CREATE TABLE IF NOT EXISTS professional_domains (
    code       TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Nature de l'organisation employeuse.
CREATE TABLE IF NOT EXISTS company_types (
    code       TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Type de contrat visé.
CREATE TABLE IF NOT EXISTS contract_types (
    code       TEXT PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- ── Données métier ──────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS companies (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,

    -- Secteur d'activité de l'entreprise, jamais le métier recherché.
    sector_id       TEXT REFERENCES sectors(id) ON DELETE SET NULL,
    -- Nature de l'organisation ; dimension distincte de la taille.
    company_type_id TEXT REFERENCES company_types(code) ON DELETE SET NULL,
    -- Taille de l'entreprise ; `UNKNOWN` plutôt que `NULL` pour n'avoir qu'une
    -- représentation du « non renseigné » à filtrer et à afficher.
    company_size    TEXT NOT NULL DEFAULT 'UNKNOWN'
        CHECK (company_size IN ('MICRO', 'TPE', 'PME', 'ETI', 'LARGE', 'UNKNOWN')),

    website         TEXT,
    -- Ville et adresse du siège ou de l'implantation principale.
    city            TEXT,
    address         TEXT,
    notes           TEXT,

    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contacts (
    id            TEXT PRIMARY KEY,
    company_id    TEXT REFERENCES companies(id) ON DELETE SET NULL,
    first_name    TEXT NOT NULL,
    name          TEXT NOT NULL,
    job_title     TEXT,
    email         TEXT,
    phone         TEXT,
    linkedin      TEXT,
    notes         TEXT,
    tracking_role TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS applications (
    id                     TEXT PRIMARY KEY,

    company_id             TEXT NOT NULL REFERENCES companies(id) ON DELETE RESTRICT,
    contact_id             TEXT REFERENCES contacts(id) ON DELETE SET NULL,

    job_title              TEXT NOT NULL,

    -- Candidature à une offre publiée, ou démarche spontanée.
    application_type       TEXT NOT NULL DEFAULT 'OFFRE'
        CHECK (application_type IN ('OFFRE', 'SPONTANEE')),

    contract_type_code     TEXT NOT NULL REFERENCES contract_types(code),

    -- Temps plein / partiel, et volume horaire hebdomadaire réel.
    weekly_work_schedule   TEXT NOT NULL DEFAULT 'UNSPECIFIED'
        CHECK (weekly_work_schedule IN ('FULL_TIME', 'PART_TIME', 'UNSPECIFIED')),
    weekly_hours           REAL
        CHECK (weekly_hours IS NULL OR (weekly_hours > 0 AND weekly_hours <= 168)),

    -- Domaine professionnel du poste. `NULL` signifie « non renseigné » : il n'est
    -- jamais déduit du secteur de l'entreprise, qui décrit une tout autre chose.
    professional_domain_id TEXT REFERENCES professional_domains(code) ON DELETE SET NULL,

    -- Surcharges propres à la candidature. `NULL` = hériter de l'entreprise ; la valeur
    -- héritée n'est jamais recopiée ici, sans quoi un changement d'entreprise laisserait
    -- des données périmées.
    city                   TEXT,
    address                TEXT,
    company_type_id        TEXT REFERENCES company_types(code) ON DELETE SET NULL,

    status                 TEXT NOT NULL DEFAULT 'EN_ATTENTE'
        CHECK (status IN ('EN_ATTENTE', 'RELANCEE', 'ENTRETIEN', 'REFUS')),

    sent_date              TEXT NOT NULL,
    -- Renseigné pour une candidature à une offre, toujours `NULL` pour une spontanée.
    job_url                TEXT
        CHECK (application_type <> 'SPONTANEE' OR job_url IS NULL),
    notes                  TEXT,

    created_at             TEXT NOT NULL,
    updated_at             TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS status_history (
    id             TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    status         TEXT NOT NULL,
    changed_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS follow_ups (
    id             TEXT PRIMARY KEY,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    follow_up_date TEXT NOT NULL,
    type           TEXT NOT NULL DEFAULT 'Email',
    notes          TEXT,
    created_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS interviews (
    id                TEXT PRIMARY KEY,
    application_id    TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    contact_id        TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    interview_date    TEXT NOT NULL,
    type              TEXT NOT NULL DEFAULT 'Présentiel'
        CHECK (type IN ('Présentiel', 'Visio', 'Téléphonique', 'Technique', 'RH', 'Autre')),
    location          TEXT,
    notes             TEXT,
    minutes           TEXT,
    calendar_event_id TEXT,
    ai_analysis       TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS resume_versions (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cover_letters (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    company    TEXT,
    job_title  TEXT,
    tone       TEXT NOT NULL DEFAULT 'formal',
    length     TEXT NOT NULL DEFAULT 'medium',
    content    TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    data       TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS profile (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    data       TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

-- ── Index ───────────────────────────────────────────────────────────────
--
-- Chacun sert un `WHERE`, un `JOIN` ou un `ORDER BY` réel des dépôts : filtres du suivi,
-- répertoire des entreprises, plages du calendrier, bibliothèques de documents.

CREATE INDEX IF NOT EXISTS idx_applications_company     ON applications(company_id);
CREATE INDEX IF NOT EXISTS idx_applications_contact     ON applications(contact_id);
CREATE INDEX IF NOT EXISTS idx_applications_status      ON applications(status);
CREATE INDEX IF NOT EXISTS idx_applications_date        ON applications(sent_date);
CREATE INDEX IF NOT EXISTS idx_applications_type        ON applications(application_type);
CREATE INDEX IF NOT EXISTS idx_applications_contract    ON applications(contract_type_code);
CREATE INDEX IF NOT EXISTS idx_applications_domain      ON applications(professional_domain_id);
CREATE INDEX IF NOT EXISTS idx_applications_company_type ON applications(company_type_id);
CREATE INDEX IF NOT EXISTS idx_applications_schedule    ON applications(weekly_work_schedule);

CREATE INDEX IF NOT EXISTS idx_companies_sector_id    ON companies(sector_id);
CREATE INDEX IF NOT EXISTS idx_companies_company_type ON companies(company_type_id);
CREATE INDEX IF NOT EXISTS idx_companies_size         ON companies(company_size);

CREATE INDEX IF NOT EXISTS idx_contacts_company            ON contacts(company_id);
CREATE INDEX IF NOT EXISTS idx_status_history_application   ON status_history(application_id);
CREATE INDEX IF NOT EXISTS idx_follow_ups_application       ON follow_ups(application_id);
CREATE INDEX IF NOT EXISTS idx_follow_ups_date             ON follow_ups(follow_up_date);
CREATE INDEX IF NOT EXISTS idx_interviews_application      ON interviews(application_id);
CREATE INDEX IF NOT EXISTS idx_interviews_date             ON interviews(interview_date);
CREATE INDEX IF NOT EXISTS idx_resume_versions_created_at  ON resume_versions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_cover_letters_created_at    ON cover_letters(created_at DESC);

-- ── Semences des référentiels ───────────────────────────────────────────
--
-- `INSERT OR IGNORE` : rejouer le fichier sur une base déjà initialisée ne duplique rien
-- et n'écrase aucune valeur. Les identifiants sont fixes — un secteur porte donc le même
-- id d'une installation à l'autre, ce qui rend les sauvegardes interchangeables.
--
-- La BDD locale est la seule source de ces listes : ni Rust ni React n'en tient de copie.

INSERT OR IGNORE INTO sectors (id, name, sort_order, created_at) VALUES
    ('5ec70000-0000-4000-8000-000000000001', 'Achats / Comptabilité / Gestion', 1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000002', 'Arts / Artisanat d''art', 2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000003', 'Banque / Assurance', 3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000004', 'Bâtiment / Travaux Publics', 4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000005', 'Commerce / Vente', 5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000006', 'Communication / Multimédia', 6, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000007', 'Conseil / Études', 7, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000008', 'Direction d''entreprise', 8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000009', 'Espaces verts et naturels / Agriculture / Pêche / Soins aux animaux', 9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-00000000000a', 'Hôtellerie - Restauration / Tourisme / Animation', 10, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-00000000000b', 'Immobilier', 11, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-00000000000c', 'Industrie', 12, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-00000000000d', 'Informatique / Télécommunication', 13, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-00000000000e', 'Installation / Maintenance', 14, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-00000000000f', 'Marketing / Stratégie commerciale', 15, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000010', 'Ressources Humaines', 16, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000011', 'Santé', 17, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000012', 'Secrétariat / Assistanat', 18, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000013', 'Services à la personne / à la collectivité', 19, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000014', 'Spectacle', 20, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000015', 'Sport', 21, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000016', 'Transport / Logistique', 22, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('5ec70000-0000-4000-8000-000000000017', 'Autre', 23, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));

-- Codes du référentiel public des domaines professionnels : la lettre désigne la grande
-- famille, le suffixe numérique une spécialisation détachée de celle-ci.
INSERT OR IGNORE INTO professional_domains (code, name, sort_order, created_at) VALUES
    ('M',   'Achats / Comptabilité / Gestion', 1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('B',   'Arts / Artisanat d''art', 2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('C',   'Banque / Assurance', 3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('F',   'Bâtiment / Travaux Publics', 4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('D',   'Commerce / Vente', 5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('E',   'Communication / Multimédia', 6, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('M14', 'Conseil / Etudes', 7, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('M13', 'Direction d''entreprise', 8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('A',   'Espaces verts et naturels / Agriculture / Pêche / Soins aux animaux', 9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('G',   'Hôtellerie - Restauration / Tourisme / Animation', 10, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('C15', 'Immobilier', 11, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('H',   'Industrie', 12, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('M18', 'Informatique / Télécommunication', 13, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('I',   'Installation / Maintenance', 14, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('M17', 'Marketing / Stratégie commerciale', 15, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('M15', 'Ressources Humaines', 16, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('J',   'Santé', 17, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('M16', 'Secrétariat / Assistanat', 18, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('K',   'Services à la personne / à la collectivité', 19, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('L',   'Spectacle', 20, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('L14', 'Sport', 21, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('N',   'Transport / Logistique', 22, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));

INSERT OR IGNORE INTO company_types (code, name, sort_order, created_at) VALUES
    ('FINAL_CLIENT',            'Client final', 1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('IT_SERVICES_COMPANY',     'ESN / Société de services numériques', 2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CONSULTING_FIRM',         'Cabinet de conseil', 3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('RECRUITMENT_AGENCY',      'Cabinet de recrutement', 4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('TEMP_AGENCY',             'Agence d''intérim', 5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SOFTWARE_PUBLISHER',      'Éditeur de logiciels', 6, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SAAS_COMPANY',            'Éditeur SaaS', 7, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('STARTUP',                 'Startup', 8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SCALEUP',                 'Scale-up', 9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('INDUSTRIAL_COMPANY',      'Entreprise industrielle', 10, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('RETAIL_COMPANY',          'Commerce / Distribution', 11, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CRAFT_BUSINESS',          'Entreprise artisanale', 12, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('AGRICULTURAL_COMPANY',    'Entreprise agricole', 13, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('BANK',                    'Banque', 14, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('INSURANCE_COMPANY',       'Assurance', 15, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('REAL_ESTATE_COMPANY',     'Entreprise immobilière', 16, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('TRANSPORT_COMPANY',       'Transport / Logistique', 17, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('TELECOM_COMPANY',         'Télécommunications', 18, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('MEDIA_COMPANY',           'Média', 19, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('PUBLIC_ADMINISTRATION',   'Administration publique', 20, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('LOCAL_AUTHORITY',         'Collectivité territoriale', 21, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('PUBLIC_INSTITUTION',      'Établissement public', 22, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('PUBLIC_COMPANY',          'Entreprise publique', 23, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('UNIVERSITY',              'Université / Enseignement supérieur', 24, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('EDUCATIONAL_INSTITUTION', 'Établissement d''enseignement', 25, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('HEALTHCARE_INSTITUTION',  'Établissement de santé', 26, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('HOSPITAL',                'Hôpital', 27, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('ASSOCIATION',             'Association', 28, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FOUNDATION',              'Fondation', 29, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('NGO',                     'ONG', 30, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('COOPERATIVE',             'Coopérative', 31, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SCOP',                    'SCOP', 32, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SCIC',                    'SCIC', 33, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SELF_EMPLOYED',           'Indépendant / Travailleur indépendant', 34, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FREELANCE_COLLECTIVE',    'Collectif de freelances', 35, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FRANCHISE',               'Franchise', 36, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('NON_PROFIT',              'Organisme à but non lucratif', 37, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('OTHER',                   'Autre', 38, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));

-- Codes du référentiel public des natures de contrat.
INSERT OR IGNORE INTO contract_types (code, name, sort_order, created_at) VALUES
    ('CDI', 'CDI', 1, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CDD', 'CDD', 2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('MIS', 'Intérim', 3, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('DIN', 'CDI Intérimaire', 4, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('SAI', 'Saisonnier', 5, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('E2',  'Contrat apprentissage', 6, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FS',  'Cont. professionnalisation', 7, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FV',  'Prépa.opérationnel.emploi', 8, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CC',  'CDI de chantier ou d''opération', 9, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('EE',  'Contrat d''Engagement Educatif', 10, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CI',  'Contrat intermittent', 11, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FJ',  'Contrat pacte', 12, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CU',  'Contrat d''usage', 13, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FT',  'CUI - CAE', 14, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FU',  'CUI - CIE', 15, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('ER',  'Engagement à servir dans la réserve', 16, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('FRA', 'Franchise', 17, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('I1',  'Insertion par l''activ.éco.', 18, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('PS',  'Portage salarial', 19, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('CCE', 'Profession commerciale', 20, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('LIB', 'Profession libérale', 21, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    ('REP', 'Reprise d''entreprise', 22, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
