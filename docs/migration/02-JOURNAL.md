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
