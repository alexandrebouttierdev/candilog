# Journal de migration

Une entrée par tranche. Chaque tranche laisse le projet compilable, testé et lançable.

---

## T0 — Socle · terminée le 2026-08-25

### Livré

**Projet** `candilog-tauri/` — Tauri 2.11, React 19, Vite 8, TypeScript strict, Tailwind 4.

**Backend Rust** (`src-tauri/src/`)

| Module | Provenance | État |
|---|---|---|
| `core/errors/` | `src/shared/error.rs` | Repris + **code IPC stable** et `Serialize` dédié |
| `core/database/connection.rs` | `src/shared/db.rs` | Repris tel quel, 8 migrations embarquées |
| `core/database/helpers.rs` | `src/shared/sqlite.rs` | Repris, tests découplés du domaine |
| `core/config/app_config.rs` | `src/core/config.rs` | Repris + dossier de dev ancré sur le manifeste |
| `core/logging.rs` | `src/core/logging.rs` | Repris, `icone_application` (Iced) retirée |
| `app/state.rs` | nouveau, sur le modèle de `src/shared/state.rs` | Pool + chemin de base ; les services s'y ajoutent par tranche |
| `app/bootstrap.rs` | nouveau | Journal, état, plugins, `invoke_handler` (vide) |

35 tests Rust, tous verts, repris de l'application Iced. Convention conservée : **un cas de
test par fichier**, déclaré dans le `mod.rs` du dossier `tests/<module>/`.

`#![deny(clippy::unwrap_used)]` et `#![deny(clippy::expect_used)]` repris de l'ancien projet.
`bootstrap` n'utilise ni `panic!` ni `expect` : un échec d'ouverture des données journalise
la cause, affiche le message destiné à l'utilisateur et sort en code 1.

**Contrat IPC** — `AppError` sérialise `{ code, message }` :

- `code` est stable (`VALIDATION_ERROR`, `NOT_FOUND`, `DATABASE_ERROR`, `HTTP_ERROR`,
  `SERIALIZATION_ERROR`, `PROVIDER_ERROR`, `CANCELLED`) et destiné au branchement frontend ;
- `message` est rédigé pour l'utilisateur ;
- le détail technique part au journal via `tracing`, à l'intérieur même de `Serialize` —
  point de passage garanti de toutes les erreurs remontées à l'interface.

**Types TypeScript** — `ts-rs` exporte les DTO vers `src/shared/types/generated/`, configuré
par `.cargo/config.toml` à la racine du projet (Cargo lit ce fichier depuis le répertoire
courant et ses ancêtres, jamais depuis celui du manifeste : la config y fonctionne aussi bien
lancée depuis `src-tauri/` que depuis la racine).

**Frontend** (`src/`)

- `styles.css` : les 14 jetons de couleur du guide SPECDESIGN en clair et sombre, plus
  typographie, rayons, hauteurs de contrôle et élévations, exposés à Tailwind via
  `@theme inline`. Trois états de thème gérés sans JavaScript (clair, sombre, système).
- `shared/services/ipc.ts` : unique point d'appel de `invoke`, normalise les rejets en
  `AppError`. Une règle ESLint interdit d'importer `invoke` ailleurs.
- `app/router/routes.ts` : carte de navigation reprise à l'identique de
  `src/navigation/mod.rs` et des maquettes — 7 sections, 16 écrans.
- Coque : `AppShell`, `NavRail`, `ContextTabs`, `PageHeader`, `EmptyState`.
- Icônes Material Symbols embarquées localement (`material-symbols`) : la CSP de la fenêtre
  interdit `fonts.googleapis.com`, et une application de bureau doit marcher hors ligne.

11 tests frontend (carte de navigation, normalisation des erreurs IPC).

**Capabilities** — `core:default`, `opener:allow-open-url`, `dialog:allow-{open,save,confirm}`.
Aucune permission filesystem : les accès fichiers passeront par des commandes Rust.

### Vérifié

```
cargo fmt --check        ok
cargo clippy -D warnings ok
cargo test               35 passed
npm run build            ok (tsc --noEmit + vite build)
npm run lint             ok
npm test                 11 passed
cargo run                fenêtre ouverte, 8 migrations appliquées, permissions 700/600
```

Coque vérifiée à l'écran en thème clair et sombre : rail, onglets contextuels et en-tête
de page se comportent comme dans les maquettes.

### Écarts assumés

- Le jeu d'icônes complet pèse 5,3 Mo dans le bundle. Acceptable pour une application de
  bureau hors ligne ; un sous-ensemble sera envisagé à la finition (T9) si le poids gêne.
- `npm` de cette machine a un cache global partiellement possédé par `root`
  (`sudo chown -R 501:20 ~/.npm` pour le réparer). En attendant, `.npmrc` du projet pointe
  vers un cache local — sans quoi `npm install` échoue sur `EEXIST`.

### Reste à faire avant T1

Rien de bloquant. La tranche T1 poursuit sur `shared/ui/` : les composants du guide non
encore écrits (Button, FormField, StatusPill, DataTable, Pager, ModalHost, DetailDrawer,
StatCard, TimelineList, EntityPicker, ErrorBanner), le sélecteur de thème et les toasts.

---

## T1 — Design system partagé · terminée le 2026-08-25

### Livré

**Jetons complétés.** Les quatre teintes de statut utilisées par les maquettes mais absentes
du tableau du guide (`accent-border`, `success-tint`, `warning-tint`, `danger-tint`) sont
ajoutées, en `oklch` comme dans les maquettes, dans les trois blocs de thème.

**`shared/ui/` — bibliothèque du guide**

| Composant | Rôle | Décision notable |
|---|---|---|
| `Button` | 4 variantes | `primary` est unique par écran, `danger` réservé aux destructions |
| `FormField` + `TextInput` / `TextArea` / `Select` | Champ, libellé, aide, erreur | `aria-describedby` et `aria-invalid` câblés dans le composant, pas dans chaque formulaire ; contrôles natifs conservés |
| `StatusPill` | Statut coloré | La couleur ne porte jamais l'information seule — libellé systématique |
| `DataTable` | Tableau dense 44 px | **Ne trie pas** : il émet la colonne demandée. Trier ici fausserait l'ordre dès la seconde page |
| `Pager` | Pagination | Ne reçoit jamais la collection, seulement page/taille/total — la pagination côté données est garantie par construction |
| `ModalHost` | Modale | `grid-rows-[auto_1fr_auto]` : le pied et l'action primaire restent visibles quelle que soit la longueur du formulaire |
| `ConfirmDialog` | Confirmation destructive | L'énoncé nomme ce qui disparaît **et ce qui survit** |
| `DetailDrawer` | Panneau latéral | N'atténue pas l'arrière-plan, contrairement à la modale |
| `StatCard`, `TimelineList`, `EmptyState`, `ErrorBanner`, `Skeleton` | États et indicateurs | Chiffres tabulaires, erreur non bloquante avec « Réessayer » |
| `Toaster` | Notifications 4 s | Jamais bloquantes ; une décision passe par `ConfirmDialog` |

**`useDismissable`** — Échap ferme, Ctrl/Cmd+Entrée valide, sur toutes les surfaces
superposées. Une pile garantit que seule la surface au sommet réagit : deux modales empilées
ne se ferment pas ensemble sur un seul Échap.

**État global** — `useUiStore` (Zustand) ne porte que le transverse non serveur : préférence
de thème et file de notifications. Le mode `system` **retire** l'attribut `data-theme` au
lieu d'y écrire une valeur, laissant jouer `prefers-color-scheme` sans rien à écouter.

**Planche de revue** — `/_design`, atteignable par l'URL seulement, jamais depuis le rail.
Sert à comparer les composants aux maquettes dans les deux thèmes et à éprouver le clavier.

### Vérifié

```
cargo fmt --check / clippy / test   inchangés, verts
npm run build                       ok
npm run lint                        ok
npm test                            41 passed (7 fichiers)
```

À l'écran, sur la planche `/_design` :

- rendu conforme aux maquettes en thème clair **et** sombre ;
- Échap ferme la modale, Ctrl+Entrée valide, le focus entre sur le premier champ ;
- confirmation destructive, toasts, états loading / empty / error ;
- aucun débordement horizontal à 1024 px ni à 2560 px ; rail replié sous 1200 px, seuil
  du guide (et non le `xl` de Tailwind à 1280 px).

### Écarts assumés

- `EntityPicker` et `KanbanBoard` ne sont **pas** écrits : le premier a besoin de la forme
  réelle des données (T2), le second est propre aux candidatures (T3). Les écrire à vide
  aurait été de la surarchitecture (§39).

### Reste à faire avant T2

Rien. T2 migre Entreprises + Secteurs + Contacts et valide la chaîne complète
View → ViewModel → Service → IPC → Rust sur la feature la plus simple.

---

## T2 — Entreprises, Secteurs, Contacts · terminée le 2026-08-25

Première feature migrée de bout en bout. Valide la chaîne complète
View → ViewModel → Service → IPC → Command → Service → Domain → Repository → SQLite.

### Backend

Trois features portées dans `features/<f>/{domain,application,infrastructure,presentation}`.
Le métier vient de l'application Iced ; deux choses ont changé au passage.

**Les traits de dépôt perdent leurs implémentations par défaut.** Dans le projet Iced,
`EntrepriseRepository::list_page` avait un corps par défaut qui chargeait toute la table
puis filtrait en Rust — pratique pour les mocks, mais un dépôt écrit sans surcharge
« fonctionnait » tout en annulant la pagination, et le défaut serait resté invisible
jusqu'à ce que le répertoire grossisse. Chaque implémentation dit maintenant explicitement
comment elle pagine.

**Le contact porte le nom de son entreprise.** `Contact.entreprise_nom` est aplati depuis un
`LEFT JOIN` : sans lui, afficher « Nova Digital » sous chaque ligne de la liste demanderait
une requête par ligne, ou de charger tout le répertoire côté React.

Ajouts au socle :

- `core/pagination` — type `Page<T>` partagé, avec `offset()` sans débordement. Les quatre
  compteurs sont annoncés `number` et non `number | bigint` côté TypeScript : ils comptent
  des lignes d'une base locale, qui ne peut pas approcher 2^53.
- `core/utils/blocking` — toutes les commandes enveloppent l'appel métier dans
  `spawn_blocking`. `rusqlite` est synchrone : appelé directement dans une commande `async`,
  il figerait l'interface le temps de l'accès disque (§28).
- `core/utils/validation` — repris de l'Iced, `reqwest::Url` remplacé par `url` : la
  validation vit dans `core` et n'a pas à faire dépendre le socle d'un client HTTP.
- **Migration 009** — `contacts.role_suivi`, pour le champ « Rôle dans le suivi » des
  maquettes. Nullable, sans `CHECK` : le rôle est un texte libre dans les maquettes, et
  figer une liste obligerait à migrer à chaque rôle nouveau.

14 commandes Tauri, toutes fines : reprendre le service, déléguer, laisser `AppError` se
sérialiser.

### Frontend

`features/{entreprises,contacts,secteurs}/` en `model` / `view` / `viewmodel` / `services`.

Les ViewModels traitent **la recherche et la pagination comme des paramètres de requête** :
la clé de cache TanStack Query les inclut, chaque changement déclenche un appel qui ne
renvoie qu'une page, et toute nouvelle recherche ramène en page 1 — rester en page 3 après
avoir restreint la recherche afficherait une liste vide alors que des résultats existent.

Deux formulaires React Hook Form + Zod, un schéma par formulaire, reprenant les règles du
backend. Les helpers `texteFacultatif` / `urlFacultative` / `identifiantFacultatif`
normalisent `""` en `null` : la base distingue `NULL` de `''`, que les `coalesce` et les
`LIKE` de la recherche ne traitent pas de la même façon.

`shared/ui/MasterList` complète la bibliothèque : liste maître d'un écran maître-détail,
générique parce que Relations l'utilise deux fois et que Candidatures la réutilisera.

### Le test qui compte

`shared/services/__tests__/commandes-ipc.test.ts` compare trois inventaires : les
`#[tauri::command]` déclarés, les commandes enregistrées dans l'`invoke_handler`, et les
chaînes réellement passées à `ipc()`. C'est le seul défaut qu'aucun compilateur ne voit :
`ipc("entreprise_lister")` au lieu de `entreprises_lister` compile des deux côtés et
n'échoue qu'à l'exécution, sur un écran vide.

### Vérifié

```
cargo fmt --check / clippy -D warnings   ok
cargo test                                82 passed (+22)
npm run build / lint                      ok
npm test                                  67 passed (+26)
```

À l'écran, sur les données réelles de la base de développement (10 entreprises,
9 contacts) : liste maître paginée, recherche, fiche détaillée avec champs non renseignés
explicitement marqués, modale de formulaire conforme aux maquettes, validation Zod rendue
sous les champs, `role_suivi` de la migration 009 affiché en pastille.

L'application native démarre, applique les neuf migrations et alimente le référentiel des
23 secteurs.

### Écarts assumés

- **Le rendu dans la fenêtre native n'a pas été vérifié à l'œil.** Les captures ont été
  prises dans le navigateur, où l'IPC est absent : les données réelles y ont transité par un
  pont temporaire, retiré depuis. Le risque de divergence porte sur le nom des commandes,
  que le test de contrat IPC couvre. `npm run tauri dev` reste à lancer pour un contrôle
  visuel dans la fenêtre.
- Le sélecteur d'entreprise du formulaire contact charge le répertoire complet, sans
  pagination : c'est un `select` natif, qui ne saurait pas demander la page suivante.
  L'`EntityPicker` paginé du guide arrive avec les candidatures, dont l'usage du répertoire
  est plus intensif.

### Reste à faire avant T3

Rien. T3 migre les candidatures : Kanban, Liste, filtres, détail, export CSV, et
l'`EntityPicker` paginé.

---

## T3 — Candidatures · terminée le 2026-08-25

La feature centrale : deux vues sur le même filtre, glisser-déposer entre statuts,
historique de statut, sept filtres cumulables et export CSV.

### Backend

Le dépôt construit sa clause `WHERE` par accumulation de **paramètres liés** — le poste, la
ville et la recherche libre viennent de champs de saisie, et les concaténer au SQL ouvrirait
une injection. La colonne de tri, elle, est interpolée : elle vient d'un enum
`TriCandidature` dont le jeu fermé rend l'injection impossible sans avoir à échapper quoi
que ce soit.

**L'historique de statut n'enregistre que les changements réels.** `update` compare l'ancien
statut au nouveau avant d'insérer, et `update_statut` n'écrit rien si la valeur est
inchangée — reposer une carte dans sa colonne d'origine est un geste courant du
glisser-déposer. Chaque étape fictive fausserait l'entonnoir de conversion des analyses, qui
compte les candidatures **passées** par l'entretien, refusées ensuite comprises.

**`repartition` ignore le filtre de statut.** Le Kanban affiche ses quatre colonnes en
permanence : si les compteurs tenaient compte du filtre, sélectionner « Entretien » viderait
les trois autres en-têtes. Ils sont calculés par `SQLite` sur tout le filtre, jamais sur la
page chargée — une colonne annoncerait sinon « 3 » en contenant tout le pipeline.

Le tri de page porte un second critère `c.created_at DESC` : sans lui, deux candidatures de
même date d'envoi changeraient d'ordre d'une page à l'autre, et une ligne pourrait
apparaître deux fois ou pas du tout.

**Export CSV** — séparateur point-virgule, comme dans l'application Iced : c'est ce
qu'attend Excel en locale française, où un fichier séparé par des virgules s'ouvre en une
seule colonne. L'export porte sur **tout le filtre** et non sur la page affichée. Le chemin
vient du sélecteur natif ouvert côté frontend ; la commande Rust n'écrit qu'à l'endroit
désigné, la fenêtre n'ayant aucune permission filesystem (§44).

### Frontend

Un seul ViewModel sert les deux vues : elles n'affichent pas les mêmes formes mais
interrogent le même filtre, et les séparer aurait dupliqué l'état des filtres, du tri et de
la pagination. Le Kanban demande une page quatre fois plus large que la Liste — une page de
huit lignes laisserait trois colonnes vides quel que soit le contenu du pipeline.

**`EntityPicker`**, le composant du guide que T2 avait laissé de côté faute de consommateur :
recherche débattue, résultats paginés, sélection au clavier. Un `select` natif aurait exigé
de charger tout le répertoire, ce que le guide interdit au-delà de cinquante éléments.

**Conversion de date centralisée.** Les maquettes saisissent en `JJ-MM-AAAA`, la base
compare des chaînes en `AAAA-MM-JJ`. La conversion est faite une fois, dans le schéma Zod :
au-delà, le ViewModel et le backend ne manipulent que de l'ISO. `versDateIso` refuse le
31 février, que `new Date` accepterait en le décalant au 3 mars.

Deux schémas Zod distincts, formulaire et filtres : leurs règles n'ont rien à voir — un
filtre vide est l'état par défaut de l'écran, un formulaire vide ne l'est pas. Le schéma de
filtres refuse une période inversée, qui ne renverrait jamais rien et donnerait un état vide
indiscernable d'une absence réelle de candidatures.

Les classes Tailwind des pastilles de statut passent par une table statique et non par une
interpolation `bg-${tone}-tint` : Tailwind n'émet que les classes qu'il trouve littéralement
dans les sources, et une classe construite à l'exécution n'existerait pas dans la feuille.

### Vérifié

```
cargo fmt --check / clippy -D warnings   ok
cargo test                                110 passed (+28)
npm run build / lint                      ok
npm test                                  96 passed (+29)
```

À l'écran, sur 15 candidatures réelles : Kanban à quatre colonnes avec compteurs,
vue Liste triable et paginée, panneau de détail avec changement de statut au clavier,
`EntityPicker` filtrant le répertoire à la frappe et paginant ses résultats.

### Écarts assumés

- Comme en T2, le rendu **dans la fenêtre native** n'a pas été vérifié à l'œil : les
  captures viennent du navigateur, où les données réelles ont transité par un pont
  temporaire, retiré depuis. Le test de contrat IPC couvre le risque de nom de commande.
- Le glisser-déposer n'a pas de test automatisé : `dragstart` / `drop` ne sont pas
  simulables de façon fiable dans jsdom. Le changement de statut qu'il déclenche l'est, lui,
  aussi bien côté ViewModel que côté dépôt.

### Reste à faire avant T4

Rien. T4 migre entretiens et relances, et l'écran Calendrier.

---

## T4 — Entretiens, relances et calendrier · terminée le 2026-08-25

### Backend

**Deux chemins morts n'ont pas été migrés.** Le dépôt d'entretiens de l'application Iced
exposait `create` et `update` en plus de `save_and_mark_candidate` ; aucun écran ne les
appelait. Les reprendre aurait conservé un piège : planifier un entretien sans que la
candidature avance. Le chemin est désormais unique — `id` absent crée, `id` présent modifie,
et l'écriture comme la mise à jour du statut tiennent dans la même transaction.

Le statut précédent est **lu avant** l'écriture de l'entretien : c'est ce qui permet de
n'historiser que les passages réels à l'étape entretien. Corriger l'heure d'un entretien ne
rejoue donc pas l'étape. La lecture vaut aussi contrôle d'existence de la candidature — la
clé étrangère la refuserait plus loin, mais avec un message technique.

Supprimer un entretien **ne rétrograde pas** la candidature : que l'entretien soit annulé ne
veut pas dire qu'elle n'a jamais atteint cette étape.

**Asymétrie signalée.** Créer une relance ne fait pas passer la candidature en « Relancée »,
alors que planifier un entretien la fait passer en « Entretien ». C'est le comportement de
l'application Iced, conservé tel quel — le corriger serait un changement de comportement,
pas une migration. Un test fige l'asymétrie pour qu'une évolution soit délibérée.

Entretiens et relances n'ont pas le même format de date : l'entretien porte un horodatage
`RFC 3339` avec décalage, la relance une date nue `AAAA-MM-JJ`. Les deux services le
valident, parce que les requêtes de plage du calendrier comparent des chaînes — une date nue
côté entretien se comparerait avant toutes les heures du même jour, faisant disparaître
l'entretien de sa propre journée.

### Frontend

**`shared/lib/dates`** rassemble les conversions que trois features partagent. Les helpers
dupliqués dans le schéma des candidatures ont été retirés au passage, avec leurs tests.
`versHorodatage` compose date et heure locales avec le décalage du fuseau : sans lui, un
entretien saisi à 14 h s'afficherait à 12 h ou 16 h selon le fuseau où la base est relue.

**`shared/ui/CandidaturePicker`** est partagé par les formulaires d'entretien et de relance.
Il vit dans `shared/ui` bien qu'il connaisse une feature : deux features distinctes en
dépendent, et le loger dans l'une ferait dépendre l'autre de sa voisine.

**Le calendrier** interroge le backend sur **les bornes de la grille**, pas du mois : la
grille déborde sur les mois voisins, et n'interroger que le mois laisserait ces cases vides
alors qu'elles portent de vrais événements. Les compteurs d'en-tête, eux, ne comptent que le
mois affiché — posés à côté de « août 2026 », ils mentiraient en incluant les débordements.

Entretiens et relances sont ramenés à une forme commune : la grille affiche des pastilles
datées et n'a pas à connaître deux entités. Le genre d'origine reste porté par l'événement,
pour rouvrir la bonne modale au clic. Dans une même journée, les relances passent en tête :
elles se traitent quand on veut, là où un entretien a un créneau.

### Vérifié

```
cargo fmt --check / clippy -D warnings   ok
cargo test                                134 passed (+24)
npm run build / lint                      ok
npm test                                  120 passed (+24)
```

À l'écran, sur 5 entretiens et 5 relances réels : grille mensuelle de 42 cases, pastilles
vertes horodatées pour les entretiens et ambre pour les relances, aujourd'hui en pastille
accent, jours hors mois estompés, navigation entre mois, et ouverture de la modale depuis
une case avec la date présélectionnée.

### Écarts assumés

- **Pas de vues Semaine ni Jour.** Les maquettes les montrent en onglets *inactifs*, et
  l'application Iced n'avait que le mois. Afficher des boutons sans effet serait précisément
  ce que le §52 interdit.
- Comme en T2 et T3, le rendu dans la fenêtre native n'a pas été vérifié à l'œil.

### Reste à faire avant T5

Rien. T5 migre le tableau de bord et les analyses, qui consomment les agrégats déjà exposés
par `candidatures::repartition` et l'historique de statut alimenté par T3 et T4.

---

## T5 — Tableau de bord et analyses · terminée le 2026-08-28

### Backend

Le frontend reçoit un **instantané complet par écran** : une commande pour le tableau de
bord et une pour les analyses. Les six blocs du Dashboard ne déclenchent donc pas six
allers-retours IPC et ne risquent pas d'afficher des valeurs calculées à des instants
différents.

Les conversions s'appuient sur l'historique, pas seulement sur le statut courant. Une
candidature refusée après un entretien reste comptée dans « Entretiens » et « Réponses » ;
les entretiens et relances persistés servent aussi de preuve lorsqu'une ancienne base ne
possède pas tout l'historique. Sans cette règle, faire avancer une candidature ferait
baisser rétroactivement les étapes précédentes de l'entonnoir.

Les périodes 30 jours, 90 jours et Tout forment un enum partagé avec TypeScript. Les KPI de
« Tout » couvrent bien tout l'historique, mais son graphique est borné à 52 fenêtres :
afficher plusieurs années de barres hebdomadaires les rendrait illisibles. Comme dans
l'application Iced, l'activité reste composée de fenêtres **glissantes de sept jours**, et
les semaines vides sont matérialisées par une CTE récursive SQLite.

Le délai moyen porte sur la première réponse observable — passage à Entretien ou Refus,
entretien persisté, ou statut courant pour les données anciennes. Les échéances fusionnent
entretiens et relances dans une seule requête ordonnée ; les candidatures à relancer sont
les dossiers encore en attente depuis au moins sept jours.

L'export CSV reprend le séparateur point-virgule des exports précédents et contient les KPI
puis l'activité de la période. Comme pour T3, le chemin vient du sélecteur natif et seule la
commande Rust écrit le fichier.

### Frontend

Le Dashboard donne la priorité aux actions : KPI principal « Candidatures actives » avec
tendance compacte, prochains événements, activité, pipeline et dossiers récents. Son CTA
« Nouvelle candidature » réutilise le formulaire de la feature Suivi par un paramètre
d'URL consommé à la fermeture ; aucun second formulaire n'a été créé.

L'écran Analyses partage les composants de visualisation avec le Dashboard, mais conserve
son propre ViewModel : période, export et création d'une relance ont leur cycle de chargement
et leurs erreurs. Le sélecteur 30/90/Tout est un vrai contrôle à état `aria-pressed`, et le
bouton Relancer ouvre la modale existante sur la candidature choisie.

Les graphiques restent du HTML/SVG léger, sans nouvelle bibliothèque : les données sont
également décrites dans une liste masquée pour les lecteurs d'écran. Chaque bloc possède
ses états chargement, erreur et vide.

### Vérifié

```
cargo fmt --check                         ok
cargo clippy --all-targets -D warnings   ok
cargo test                                148 passed (+14)
npm run build / lint                      ok
npm test                                  123 passed (+3)
```

La revue navigateur a couvert le Dashboard et Analyses avec données représentatives, le
sélecteur de période, l'ouverture de la modale de relance et une largeur compacte de 900 px.
Aucun débordement horizontal ni erreur console n'a été observé. Le pont d'aperçu a été
retiré avant les contrôles finaux.

### Écarts assumés

- Le rendu dans la fenêtre Tauri native n'a pas été revérifié. Le navigateur a servi à la
  comparaison visuelle avec un pont IPC temporaire retiré avant livraison ; la compilation
  Rust et les 148 tests ont ensuite passé sur le code final.
- Le bundle JavaScript principal pèse environ 534 ko minifié et déclenche l'avertissement
  Vite à 500 ko. La découpe par route est reportée à T9, où l'optimisation globale est
  prévue ; elle ne change pas le comportement de cette tranche.

### Reste à faire avant T6

Rien. T6 migre le profil, les compétences et les objectifs.

---

## T6 — Profil, compétences et objectifs · terminée le 2026-08-28

### Backend

Le profil reste une **ligne singleton JSON** dans la table historique `profil`. Le dépôt
SQLite possède volontairement une représentation de stockage distincte du domaine IPC :
il relit et réécrit les clés anglaises de l'application Iced (`personal`, `skills`,
`education`, etc.), tandis que React reçoit des DTO français en camelCase. Une base déjà
remplie s'ouvre ainsi sans conversion destructive ni changement silencieux de schéma.

Le chargement renvoie le profil, son horodatage et un score de complétion sur sept
sections. Une collection ne compte que si elle contient au moins une entrée réellement
complète : une expérience legacy sans entreprise ou date ne gonfle pas le score. Le
service valide à nouveau toutes les règles côté Rust — e-mail, liens HTTP(S), champs
obligatoires et incohérence entre poste actuel et date de fin — même si le formulaire Zod
les contrôle déjà.

L'« objectif » n'est pas une entité séparée dans l'ancien domaine. Il est conservé dans
`titre` et `resume` de l'identité, ce qui correspond à l'accroche et à la présentation déjà
utilisées par le générateur de CV. Projets et certifications sont également conservés pour
que T7 puisse les exploiter sans perte de données.

### Frontend

L'écran suit les quatre onglets de la maquette : Expériences, Compétences, Formations et
Langues. Projets et certifications restent éditables sous Formations, plutôt que de créer
des onglets supplémentaires absents du guide. Les onglets exposent les rôles ARIA attendus
et se parcourent avec Gauche, Droite, Début et Fin.

Le bandeau relie identité, objectif et progression ; la colonne latérale rend visibles la
prochaine section utile, l'objectif détaillé et les coordonnées sans dupliquer une grille
de KPI. Sept éditeurs réutilisent la modale commune du design system. Les listes répétables
peuvent ajouter ou retirer leurs entrées, les compétences s'ajoutent aussi avec Entrée, et
les erreurs restent sous leur champ avec `aria-invalid` et `aria-describedby`.

### Vérifié

```
cargo fmt --check                         ok
cargo clippy --all-targets -D warnings   ok
cargo test                                165 passed (+17)
npm run build / lint                      ok
npm test                                  128 passed (+5)
```

La revue navigateur a couvert le profil rempli, les quatre onglets, les modales Identité
et Expériences, les erreurs de liens et de champs obligatoires, la navigation clavier et
une largeur compacte de 900 px. Il n'y a qu'un repère `main`, aucun débordement horizontal
et aucune erreur console. Le pont d'aperçu temporaire a été retiré avant les contrôles
finaux.

### Écarts assumés

- L'import de CV n'est pas affiché comme action inerte : il dépend de l'analyse IA et sera
  livré dans T7 avec les écrans Documents.
- Le rendu a été comparé dans le navigateur, pas dans la fenêtre Tauri native.
- Le bundle JavaScript principal atteint environ 563 ko minifié. La découpe par route reste
  planifiée pour T9 avec les autres optimisations transversales.

### Reste à faire avant T7

Rien. T7 migre les CV, lettres, génération de documents et fonctions IA associées.

---

## T7 — Documents et IA · terminée le 2026-08-28

### Backend

Les bibliothèques de CV et de lettres relisent les tables historiques `cv_versions` et
`lettres_motivation` (colonnes anglaises `name` / `content` / `company`…) et exposent des
DTO français en camelCase. Le JSON d'une version de CV reste opaque côté persistance : une
ancienne entrée Iced s'ouvre, même si son aperçu structuré n'est pas encore reconstruisible.

Cinq workflows IA sont branchés, corrélés par identifiant, annulables, et émettent
l'événement Tauri `ia-progression` sans polling : analyse d'offre, génération de CV,
rédaction de lettre (fragments successifs), analyse d'un PDF local, extraction de profil.
La lecture PDF est bornée (extension, magique `%PDF-`, 10 Mo) et s'exécute en local ; le
texte est ensuite envoyé uniquement au fournisseur configuré dans `parametres`.

La configuration LLM historique est lue sans réécriture. Une base neuve ou un JSON `{}`
retombe sur Ollama local. Un endpoint distant doit être en HTTPS et ne peut pas cibler une
IP privée. Les opérations longues libèrent leur jeton d'annulation y compris en cas d'erreur.

### Frontend

Cinq écrans Documents reprennent la navigation des maquettes : Mes CV, Générer un CV,
Mes lettres, Lettre de motivation, Analyser. L'aperçu A4 est un document papier (fond
blanc) distinct du thème de l'application. L'annulation coupe réellement le workflow côté
Rust. L'import de CV, reporté depuis T6, s'ouvre depuis le profil : l'IA prépare les
données, l'utilisateur confirme la fusion, les doublons exacts sont ignorés.

### Vérifié

```
cargo fmt --check                         ok
cargo clippy --all-targets -D warnings   ok
cargo test                                197 passed (+32)
npm run build / lint                      ok
npm test                                  132 passed (+4)
```

### Écarts assumés

- L'export PDF du CV généré (`printpdf`, présent dans l'application Iced) n'est pas porté :
  l'aperçu A4 et l'enregistrement en bibliothèque locale couvrent le parcours Documents ;
  l'export fichier est reporté à T9 avec les autres finitions.
- L'analyse IA d'un compte-rendu d'entretien n'appartient pas à cette tranche.
- La revue a été faite par les tests et la compilation, pas dans la fenêtre Tauri native
  ni via un pont navigateur : cette session n'avait pas d'outils de contrôle visuel interactif.
- Le bundle JavaScript principal atteint environ 588 ko minifié. La découpe par route reste
  planifiée pour T9.

### Reste à faire avant T8

Rien. T8 migre les réglages (fournisseur IA, sauvegardes, mises à jour, à propos).

---

## T8 — Réglages · terminée le 2026-08-28

### Backend

Les réglages applicatifs vivent dans `features/parametres/`. Le JSON historique de
`parametres.data` reste en snake_case (compatibilité Iced) ; l'IPC expose un DTO camelCase.
La clé API n'est plus écrite dans SQLite : au chargement, une clé héritée est déplacée vers
le coffre natif (`keyring`, service `com.alexandrebouttier.candilog` / `llm-api-key`) ; à
l'enregistrement, le JSON est persisté sans secret. Ollama n'interroge pas le trousseau, ce
qui laisse les tests CI fonctionner sans service de secrets. Les appels IA (`charger_config`)
réinjectent la clé du coffre pour les fournisseurs cloud.

La validation reprend Iced : température 0.0–2.0, endpoint requis pour Custom, clé requise
pour Claude / OpenAI / Gemini / Mistral / NVIDIA. Health-check et liste de modèles passent
par les mêmes endpoints que l'application Iced (`/api/tags`, `/v1/models`, `/v1beta/models`).

Sauvegardes : export SQLite via l'API backup, validation d'en-tête / intégrité / tables /
version de schéma, restauration avec copie de secours et retour arrière. La réinitialisation
vide aussi `lettres_motivation` (oubli Iced) et conserve le référentiel des secteurs.

Mises à jour : API GitHub `candilog-releases` avec `User-Agent`, comparaison semver, asset
par plateforme, téléchargement borné à 256 Mo dans Téléchargements, lancement assisté (pas
d'installation silencieuse), événement `maj-progression`.

### Frontend

Quatre écrans branchés : Intelligence artificielle (grille de sept fournisseurs, jamais une
liste déroulante), Sauvegardes, Mises à jour, À propos. Le thème clair / sombre / système est
persisté avec les réglages et hydraté au démarrage. L'action primaire de l'écran IA est
« Enregistrer ».

### Vérifié

```
cargo fmt --check                         ok
cargo clippy --all-targets -D warnings   ok
cargo test                                226 passed (+29)
npm run build / lint                      ok
npm test                                  135 passed (+3)
```

### Écarts assumés

- L'application Iced n'est pas retirée : le crate racine reste le pipeline de release
  (`docs/RELEASES.md`). T9 se limite à l'accessibilité, au responsive et à la découpe du
  bundle, sans supprimer `src/`.
- L'export PDF du CV (`printpdf`) reste reporté.
- La revue a été faite par les tests et la compilation, pas dans la fenêtre Tauri native
  ni via un pont navigateur : cette session n'avait pas d'outils de contrôle visuel interactif.
- Le bundle JavaScript principal atteint environ 607 ko minifié. La découpe par route reste
  planifiée pour T9.

### Reste à faire avant T9

Rien. T9 couvre l'accessibilité, le responsive 1024–2560, les tests manquants et la découpe
du bundle. L'ancien projet Iced reste comme référence et binaire de release.

