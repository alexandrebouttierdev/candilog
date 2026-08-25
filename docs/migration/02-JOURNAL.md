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
