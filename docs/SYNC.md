# Synchronisation entre ordinateurs

Candilog fonctionne d'abord **en local** : c'est le mode par défaut, sans compte ni réseau.
La synchronisation est facultative. Rattaché à un **espace** — un dossier de votre choix,
Google Drive ou OneDrive —, un ordinateur y publie des lots chiffrés de ses modifications
et applique ceux des autres. Aucun serveur Candilog, aucun compte Candilog, aucun
abonnement : l'espace est un simple ensemble de fichiers chez le fournisseur choisi.

Code : `src-tauri/src/features/sync/` (Rust) et `src/features/sync/` (interface). Réglages
→ « Synchronisation et sauvegarde ».

## Ce qui est synchronisé

| Synchronisé | Reste sur l'ordinateur |
| --- | --- |
| Candidatures, historique de statut, relances, entretiens | Réglages (`settings`) : fournisseur IA, modèles locaux, thème |
| Entreprises, contacts, notes de relation | Clés API et secrets du coffre système |
| CV et lettres (toutes les versions) | `app_kv` (compteurs internes), référentiels semés |
| Vues enregistrées | Modèles d'IA téléchargés, journaux, exports |
| Profil, et sa photo (fichier joint) | Les PDF exportés (choisis par l'utilisateur, hors Candilog) |

La liste vit dans `domain/entities.rs` ; un test la compare au schéma réel, si bien qu'une
migration qui ajoute une colonne sans la déclarer échoue.

## Architecture

```text
Écriture métier ──► déclencheur SQLite ──► sync_outbox        (même transaction)
                                              │
SyncService (tâche de fond) ──► SyncEngine ──► SqliteSyncStore ──► lot chiffré (sync_pending)
                                    │                                   │
                                    └──── RemoteStore ◄─────────────────┘
                                          ├─ FolderStore      (dossier local)
                                          ├─ GoogleDriveStore (Drive v3, appDataFolder)
                                          └─ OneDriveStore    (Graph, approot)
```

- `domain/` : vecteurs de versions, format des objets distants, arbitrage par colonne,
  contrat `RemoteStore`. Ni Tauri, ni rusqlite, ni client HTTP.
- `application/engine.rs` : le cycle (envoi, réception, instantanés, ménage), indépendant
  du fournisseur. `application/service.rs` : configuration, mise en service, planification,
  état affiché.
- `infrastructure/` : état SQLite (`store.rs`, `apply.rs`), chiffrement (`crypto.rs`),
  adaptateurs, OAuth (`oauth.rs`), coffre (`secrets.rs`).

Ajouter un fournisseur revient à implémenter `RemoteStore` (quatre opérations : lister,
lire, créer sans remplacer, supprimer).

## Modèle de données (migration 9)

Aucune table métier n'est modifiée. `009_sync.sql` ajoute :

| Table | Rôle |
| --- | --- |
| `sync_state` | Configuration locale (fournisseur, dossier, espace, installation), compteur de lots, dernière réussite |
| `sync_flags` | `tracking` : rattaché, on journalise ; `applying` : posé le temps d'appliquer des changements distants |
| `sync_outbox` | Enregistrements modifiés localement, non encore publiés |
| `sync_records` | Version connue de chaque enregistrement : vecteur de l'enregistrement, vecteur par colonne, contenu de référence, marqueur de suppression |
| `sync_cursors` | Dernier lot appliqué de chaque autre installation |
| `sync_pending` | Lot chiffré préparé, pas encore confirmé par le fournisseur |
| `sync_conflicts` | Conflits en attente d'une décision |
| `sync_deferred` | Changements reçus dont le parent n'est pas encore arrivé |

Trois déclencheurs par table synchronisée (`sync_<table>_insert|update|delete`) écrivent
dans `sync_outbox` **dans la transaction de l'écriture métier** : une modification validée
est toujours journalisée, une modification annulée ne l'est jamais. Les cascades et les
`SET NULL` des clés étrangères déclenchent eux aussi ces déclencheurs. En mode local, le
drapeau `tracking` est absent et rien n'est journalisé.

La configuration vit dans `sync_state`, pas dans `settings` : restaurer une sauvegarde ou
remettre à zéro **détache** l'ordinateur (`core::backup::clear_sync_state`), sans quoi il
reprendrait l'identité et les compteurs d'un autre état. Une remise à zéro ne publie donc
aucune suppression vers les autres ordinateurs.

## Protocole

### Objets distants

```text
space.json                                  manifeste en clair, sans donnée personnelle
changes/<installation>/<seq:012>.batch      lot de changements chiffré
snapshots/<installation>/<seq:012>.snap     instantané complet chiffré
blobs/<hmac>.blob                           fichier joint chiffré (photo du profil)
```

Tous les objets sont **immuables** et chaque installation n'écrit que sous son propre
identifiant. Deux ordinateurs ne peuvent donc pas s'écraser, et aucun fournisseur n'a à
offrir d'écriture conditionnelle. Un nom inattendu (copie de conflit d'un outil tiers,
fichier temporaire) est ignoré.

### Identifiants et versions

- **Enregistrement** : sa clé primaire existante (UUID ; `1` pour le profil).
- **Installation** : UUID tiré au rattachement, conservé avec la clé de l'espace dans le
  coffre système. Une base copiée sur un autre poste, dont le coffre ne connaît pas
  l'identité, demande la phrase secrète puis reçoit une **nouvelle** identité.
- **Version** : vecteur `installation → numéro du dernier lot vu`, pour l'enregistrement et
  pour **chaque colonne**. Les horloges ne servent jamais à arbitrer.

### Publication

1. `prepare_batch` lit le journal, calcule pour chaque enregistrement les colonnes modifiées
   depuis le contenu de référence, leur donne la version `version connue ⊔ {moi: n}`,
   chiffre le lot et l'**enregistre** dans `sync_pending`, le tout dans une transaction qui
   purge aussi le journal.
2. Les fichiers joints sont envoyés, puis le lot, en création seule.
3. Le lot n'est retiré de `sync_pending` qu'après confirmation du fournisseur.

Après une coupure, c'est exactement le même contenu, sous le même nom, qui est renvoyé. Si
le fournisseur a écrit le lot mais que la réponse s'est perdue, l'objet existant est relu :
identique, il vaut confirmation ; différent, l'identité de l'installation est renouvelée
(base copiée d'un autre poste).

Un enregistrement en conflit n'est jamais publié avant la décision de l'utilisateur.

### Réception

Les lots de chaque installation sont appliqués **dans l'ordre, sans jamais sauter un trou**
(un fichier pas encore livré par l'outil tiers bloque la suite de cette installation, pas
des autres). Chaque lot s'applique dans **une** transaction avec son curseur : une coupure
l'annule en entier et il sera rejoué. Rejouer un lot déjà appliqué ne change rien
(versions déjà couvertes) : aucune opération ne crée de doublon.

Ordre d'application : créations et modifications des parents vers les enfants, puis
suppressions des enfants vers les parents. Un enfant dont le parent n'est pas encore
arrivé attend dans `sync_deferred` ; un parent supprimé ici mais encore utilisé ailleurs
est restauré depuis son dernier contenu connu (perdre la candidature créée ailleurs serait
pire que de faire réapparaître son entreprise).

### Conflits

Arbitrage **colonne par colonne** :

- la version distante a vu la version locale → elle s'applique, sauf si la colonne a été
  modifiée ici depuis ;
- la version locale a vu la version distante → elle reste ;
- écritures parallèles de valeurs différentes → **conflit présenté**, la valeur locale est
  conservée en attendant. Deux ordinateurs qui modifient deux champs différents d'une même
  fiche ne sont pas en conflit.

Exceptions déterministes, identiques sur tous les ordinateurs : `updated_at` et
`reference_number` prennent le maximum. Deux contraintes d'unicité que deux ordinateurs
peuvent enfreindre sans le savoir sont réglées de la même façon partout : le plus petit
identifiant garde un numéro de candidature (l'autre est renuméroté), la version au numéro
le plus élevé reste la version courante d'un document (aucune version n'est perdue). Une
fiche enregistrée localement mais jamais publiée, présente aussi dans l'espace (fusion),
n'est en conflit que sur les colonnes qui diffèrent.

Une décision (« garder cet ordinateur » / « prendre l'autre version ») est **publiée** avec
une version qui a vu les deux côtés : les autres ordinateurs l'adoptent et leur propre
conflit se lève, sans avoir à trancher à nouveau.

### Suppressions

Une suppression est un **marqueur** versionné. Elle ne s'applique que si son auteur avait
vu la version locale ; sinon c'est un conflit « supprimé ailleurs, modifié ici ». À
l'inverse, une modification reçue pour une fiche supprimée ici est un conflit, sauf si son
auteur avait vu la suppression (recréation délibérée). Une version ancienne rejouée ne fait
jamais réapparaître une fiche supprimée. Une suppression distante qui emporterait en
cascade un enfant créé ici (que l'auteur ne connaissait pas) devient un conflit.

**Rétention** : les marqueurs sont gardés 180 jours. Les lots d'une installation sont
supprimés par elle seule, quand un de ses instantanés publié depuis plus de 90 jours les
couvre. Un ordinateur resté hors ligne plus longtemps repart du meilleur instantané ; les
enregistrements qu'il connaît, déjà vus par l'instantané mais absents de celui-ci, ont été
supprimés entre-temps et sont retirés.

### Instantanés

Toutes les 50 publications, une installation publie un instantané de l'état **publié**
(jamais de modification locale non publiée) avec la couverture `installation → lot`. Un
nouvel ordinateur part du plus complet puis rejoue les lots suivants.

### Garanties

- SQLite n'est jamais copié comme fichier actif ; les sauvegardes utilisent l'API backup.
- Une modification validée localement est journalisée atomiquement et n'est retirée du
  journal qu'une fois enregistrée dans un lot conservé localement ; ce lot n'est retiré
  qu'après confirmation du fournisseur.
- « Synchronisé » n'est affiché que si plus rien n'attend et que le dernier cycle a réussi.
- Aucune modification n'est écrasée en silence par une modification parallèle.
- Les ordinateurs convergent vers le même état une fois les mêmes lots appliqués et les
  conflits tranchés.

## Chiffrement

Le chiffrement côté client est **obligatoire** pour tous les fournisseurs, dossier compris.

- Clé d'espace aléatoire de 256 bits, tirée à la création.
- Sous-clés HKDF-SHA256 (sel : identifiant de l'espace) : chiffrement ChaCha20-Poly1305 des
  objets, et HMAC-SHA256 pour nommer les fichiers joints sans révéler l'empreinte de leur
  contenu.
- Chaque objet : en-tête `CLS1`, nonce aléatoire de 96 bits, données associées
  `espace + nom` (un fichier renommé ou déplacé ne se déchiffre plus). Contenu compressé
  avant chiffrement.
- La clé d'espace est déposée dans `space.json` chiffrée par une clé dérivée de la **phrase
  secrète** (Argon2id, 64 Mio, 3 passes, sel aléatoire ; paramètres bornés à la lecture).
- **Clé de récupération** : la clé d'espace en base32, affichée une seule fois à la
  création. Elle remplace la phrase secrète oubliée.
- Sur chaque ordinateur, la clé d'espace est conservée dans le coffre système ; elle ne
  passe jamais par SQLite, l'IPC ni les journaux.

Bibliothèques : `ring` (déjà présent par rustls) et `argon2` (RustCrypto). Aucun algorithme
écrit à la main.

**Sans la phrase secrète ni la clé de récupération, l'espace est illisible, y compris pour
Candilog** : il n'existe pas de recours. Ce qui reste visible chez le fournisseur : le
nombre d'ordinateurs (identifiants aléatoires), le nombre, la taille et les dates des lots.

## Fournisseurs

### Dossier personnalisé

Un dossier local, éventuellement synchronisé par Dropbox, iCloud Drive, Nextcloud ou un
partage réseau. Candilog crée `candilog-sync/` dedans, vérifie écriture, relecture et
suppression, écrit par fichier temporaire puis renommage, et refuse un dossier situé dans
ses propres données.

Limites : Candilog ne contrôle pas l'outil tiers. Un fichier encore en transit est traité
comme pas encore disponible (nouvel essai) ; un fichier tronqué échoue au déchiffrement et
bloque cette seule installation jusqu'à sa livraison complète. Plusieurs ordinateurs
peuvent écrire en même temps sans risque : chacun n'écrit que ses propres fichiers. Un
dossier démonté n'est jamais recréé vide : l'état passe en erreur temporaire.

### Google Drive

API Drive v3, permission `https://www.googleapis.com/auth/drive.appdata` (non sensible) :
uniquement le dossier caché `appDataFolder` de Candilog. Conséquences : invisible dans
l'interface de Drive ; supprimé si l'utilisateur retire l'accès de Candilog à son Drive ;
accessible depuis toute installation de la même application OAuth. Drive tolère deux
fichiers de même nom : l'existence est vérifiée avant chaque création.

### Microsoft OneDrive

Microsoft Graph v1.0, permissions `Files.ReadWrite.AppFolder` et `offline_access` :
uniquement `Applications/Candilog` (`/drive/special/approot`). Ce dossier est **visible**
et modifiable par l'utilisateur ; un fichier qu'il supprime est traité comme manquant.
Téléversement simple (moins de 250 Mo par objet, très au-delà d'un lot).

### OAuth

Code d'autorisation + PKCE S256, navigateur du système, redirection sur
`http://127.0.0.1:<port éphémère>`, `state` vérifié, délai de 5 minutes, annulable. Aucun
secret confidentiel : Microsoft n'en utilise pas pour une application publique ; la valeur
que Google remet aux applications de bureau n'est pas confidentielle par conception. Le
jeton de renouvellement est conservé dans le coffre système, le jeton d'accès en mémoire ;
un refus 401 provoque un seul renouvellement. Session révoquée → état « Reconnexion
nécessaire ». La déconnexion révoque le jeton chez Google ; Microsoft n'offre pas de
révocation à une application publique (l'utilisateur la retire depuis son compte).

Les redirections HTTP ne suivent que du HTTPS, et l'en-tête d'autorisation n'est jamais
transmis à un autre hôte. Les jetons ne sont jamais journalisés.

## Configurer les applications OAuth

Google Drive et OneDrive ne sont proposés que si cette version a été **compilée** avec un
identifiant d'application ; sinon l'écran les affiche indisponibles et le mode local et le
dossier personnalisé restent utilisables.

| Variable (compilation) | Contenu |
| --- | --- |
| `CANDILOG_GOOGLE_CLIENT_ID` | Identifiant client OAuth Google, type « Application de bureau » |
| `CANDILOG_GOOGLE_CLIENT_SECRET` | Valeur fournie par Google pour ce client (non confidentielle) |
| `CANDILOG_MICROSOFT_CLIENT_ID` | Identifiant d'application (client) Microsoft Entra |

**Google Cloud Console** : créer un projet, activer l'API Google Drive, configurer l'écran
de consentement (portée `.../auth/drive.appdata`), créer un identifiant « ID client OAuth »
de type **Application de bureau**. Les redirections sur la boucle locale sont acceptées
d'office pour ce type.

**Microsoft Entra (portail Azure)** : « Inscriptions d'applications », comptes « de
n'importe quel annuaire et comptes Microsoft personnels », plateforme **Applications
mobiles et de bureau**, autoriser les flux de client public. Déclarer la redirection
`http://127.0.0.1` (par le manifeste, `replyUrlsWithType`, le portail refusant le schéma
`http` en saisie). Permissions déléguées Microsoft Graph : `Files.ReadWrite.AppFolder`,
`offline_access`. Le nom de l'application devient celui du dossier
`Applications/<nom>`.

Le workflow de release lit ces valeurs dans les variables et secrets du dépôt GitHub
(`docs/RELEASES.md`).

## Cycle et états

La tâche de fond lance un cycle au démarrage, après 5 s de calme suivant une modification
(au plus tard 60 s), toutes les 5 minutes pour les services cloud (2 minutes pour un
dossier), sur demande, et après chaque reprise. Échecs passagers : nouveaux essais à 30 s,
1, 2, 5, 10 puis 15 minutes ; une limite de débit attend le délai annoncé. Session
révoquée, clé absente ou espace d'une version plus récente bloquent les cycles automatiques
jusqu'à une action de l'utilisateur. La tâche est annulée à la fermeture de la fenêtre ; une
transaction en cours se termine ou s'annule en entier.

États : non configuré, suspendu, synchronisé, en cours, modifications en attente, hors
ligne, reconnexion nécessaire, phrase secrète nécessaire, erreur temporaire, conflit,
restauration en cours, mise à jour nécessaire. L'événement `sync://status` les pousse à
l'interface.

## Première synchronisation et changement de fournisseur

Le scénario est toujours **choisi explicitement** :

1. **Créer l'espace** (emplacement vide) : phrase secrète, puis envoi des données locales.
2. **Récupérer l'espace** : les données locales sont remplacées, après une sauvegarde
   complète dans le dossier des exports (`candilog-avant-synchronisation-<date>.sqlite`).
   Tout l'espace est téléchargé et vérifié **avant** le remplacement, qui se fait en une
   transaction : un échec laisse les données locales intactes.
3. **Fusionner** : les deux ensembles sont réunis ; les fiches qui diffèrent des deux côtés
   deviennent des conflits.

Sans données locales, 2 et 3 reviennent au même.

Changer de fournisseur : désactiver (les données locales et l'ancien espace ne sont pas
touchés), éventuellement créer une sauvegarde, puis configurer le nouveau. Les espaces ne
sont jamais mélangés : chacun a son identifiant et sa clé.

## Restauration

- **Restaurer depuis l'espace** (Réglages) : sauvegarde automatique puis remplacement des
  données locales par l'espace, comme le scénario 2.
- **Restaurer une sauvegarde locale** : remplace la base (procédure de `docs/DATA.md`) et
  **détache** la synchronisation. Pour la reprendre, reconfigurer l'emplacement et choisir
  « Fusionner » (les différences avec l'espace deviennent des conflits) ou « Récupérer ».
- **Clé absente** (nouveau poste avec une base copiée, trousseau effacé) : saisir la phrase
  secrète ou la clé de récupération ; l'ordinateur reçoit une nouvelle identité.

## Limites connues

- Le profil est un seul champ JSON : deux modifications parallèles du profil donnent un
  conflit sur l'ensemble du profil.
- Deux nouvelles versions d'un même document créées en parallèle peuvent porter le même
  numéro (« v3 » deux fois) ; aucune n'est perdue.
- Les fournisseurs réels (Google, Microsoft) n'ont été validés que contre des serveurs
  simulés dans les tests : une validation avec de vrais comptes reste à faire avant de les
  annoncer.
- Le trousseau Windows limite la taille d'un secret ; un jeton de renouvellement très long
  pourrait y être refusé (message d'erreur explicite).
- Un espace écrit par une version plus récente de Candilog est refusé jusqu'à la mise à
  jour de cette installation.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml --locked features::sync
```

Scénarios couverts sans aucun compte réel : protocole et validation des objets,
arbitrage, deux et trois ordinateurs sur un espace simulé (propagation, idempotence,
conflits, suppressions concurrentes, coupures, réponses perdues, redémarrage, trous,
instantanés, remplacement, fusion, références et versions courantes), parcours du service
avec le fournisseur dossier réel sur disque (création, adhésion, mauvaise phrase, clé de
récupération, déconnexion, changement de fournisseur, dossier démonté, clé absente),
chiffrement, OAuth contre de faux serveurs (refus, `state` falsifié, révocation,
renouvellement), adaptateurs Drive et Graph contre des émulations HTTP, migration d'une
base existante et instantané SQLite sous écritures concurrentes.
