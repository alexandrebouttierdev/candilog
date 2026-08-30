# Données locales

La base `candilog.sqlite` est résolue par `core::config::AppPaths`. Une release utilise le dossier historique `com.candilog.desktop` du répertoire de données de l'OS ; un binaire debug utilise obligatoirement `src-tauri/.candilog-dev/` (ancré sur `CARGO_MANIFEST_DIR`) afin de ne jamais ouvrir la base utilisateur pendant le développement. Le schéma complet vit dans un seul fichier embarqué, `init_schema.sql`, appliqué via `PRAGMA user_version` : tables, index et semences des référentiels. Aucune migration héritée n'est conservée — une base neuve obtient directement le modèle final.

Les règles de relation restent : entreprise/candidature en `RESTRICT`, candidature/dépendances en `CASCADE`, contact optionnel en `SET NULL`. Les UUID et dates ISO 8601 sont générés en Rust. Les tests n'ouvrent jamais la base utilisateur.

## Référentiels métier

Le schéma courant (`PRAGMA user_version = 1`, fichier unique `init_schema.sql`) porte
quatre catalogues **distincts**, semés par le fichier lui-même en `INSERT OR IGNORE` :

| Table | Clé | Rôle |
| --- | --- | --- |
| `sectors` | UUID fixe | secteur d'activité **de l'entreprise** |
| `professional_domains` | code métier (`M18`) | domaine professionnel **du poste** |
| `company_types` | code (`IT_SERVICES_COMPANY`) | nature de l'organisation |
| `contract_types` | code (`MIS`) | type de contrat |

Ces concepts ne sont jamais fusionnés : une banque (`sector`) recrute des informaticiens
(`professional_domain`). La taille (`companies.company_size`) est une cinquième dimension,
indépendante du type — « ESN + PME » et « Association + TPE » doivent rester exprimables.

Les identifiants des secteurs sont figés dans `init_schema.sql` plutôt que générés au
démarrage : une sauvegarde reste ainsi lisible sur une autre installation. Le code métier
sert de clé primaire aux trois autres catalogues — générer un UUID pour une valeur déjà
identifiante n'ajouterait qu'une indirection.

La base est l'unique source de ces listes : ni Rust ni React n'en tient de copie. Les
libellés affichés sont résolus par jointure (`sector_name`, `contract_type_name`,
`professional_domain_name`), jamais dupliqués en colonne — deux sources de vérité que rien
ne garderait d'accord.

## Valeurs héritées de l'entreprise

`applications.city`, `applications.address` et `applications.company_type_id` sont des
**surcharges** : `NULL` signifie « hériter de l'entreprise », jamais « vide ». La valeur
effective est calculée en SQL (`coalesce(applications.x, companies.x)`) et exposée sous
`effective_*`. La valeur héritée n'est jamais recopiée dans `applications` : elle se
figerait, et changer l'entreprise de la candidature laisserait derrière elle la ville de la
précédente. Les filtres portent donc eux aussi sur l'expression `coalesce`, faute de quoi
une candidature qui hérite de sa ville échapperait au filtre correspondant.

`company_size` appartient à l'entreprise seule : aucune surcharge n'existe côté
candidature, et le filtre par taille passe par la jointure sur `companies`.

## Contraintes portées par le schéma

Le service Rust valide, mais n'est pas la seule barrière : `CHECK` sur `company_size`,
`application_type`, `weekly_work_schedule`, `status`, sur `cover_letters.tone` et
`cover_letters.length`, sur les bornes de `weekly_hours` (`0 < h <= 168`), et sur
l'exclusion d'un `job_url` pour une candidature `SPONTANEE`. Les
clés étrangères vers les référentiels sont réelles — `PRAGMA foreign_keys = ON` est posé
par l'initialiseur de **chaque** connexion du pool.

## Recherche et normalisation

`lower()` de SQLite n'agit que sur l'ASCII : il laisse « É » intact. Les dépôts comparent
donc `search_key(colonne)` au motif produit par `like_contains`, qui applique la même
normalisation (NFD, retrait des marques combinantes, minuscules, espaces compactés).
`search_key` est une fonction scalaire déterministe enregistrée sur **chaque** connexion du
pool par l'initialiseur, à côté de `PRAGMA foreign_keys` ; son implémentation Rust vit dans
`core::utils::text`. Normaliser d'un seul côté rendait « ÉCOLE » introuvable, y compris en
le cherchant par son nom exact.

## Permissions du dossier de données

La base contient l'intégralité des données personnelles : profil, CV générés, coordonnées
des contacts, notes d'entretien. Sur Unix, `AppPaths` force donc `700` sur le dossier de
données et son sous-dossier `exports`, et `600` sur `candilog.sqlite`, ses journaux WAL et
SHM ainsi que `candilog.log` — le `umask` de session donnerait sinon `755` / `644`. Les
permissions sont réappliquées après l'ouverture de la base, le fichier n'existant pas
encore au premier démarrage. Un échec est journalisé sans empêcher le démarrage.

Hors Unix, il n'existe pas d'équivalent portable : sous Windows, la protection repose sur
les ACL héritées du profil utilisateur.

## Sauvegardes

Les sauvegardes doivent utiliser l'API backup SQLite. Le fichier produit est restreint à
`600` avant d'être rempli : il porte les mêmes données que la base, et la copie de secours
prise avant une restauration survit à l'échec de celle-ci. Une restauration doit valider l'en-tête, ouvrir la base, exécuter `PRAGMA integrity_check`, vérifier les versions puis remplacer la base avec possibilité de retour arrière.

## Compatibilité des bases

Seule la lignée du schéma courant est prise en charge. Une base ou une sauvegarde d'une
ancienne génération est refusée en lecture seule, avant l'ouverture du pool et donc avant
toute écriture de migration, de journal WAL ou de pragma. Candilog ne la migre et ne la
supprime jamais automatiquement : il faut déplacer ou supprimer manuellement le fichier
signalé avant de relancer l'application.
