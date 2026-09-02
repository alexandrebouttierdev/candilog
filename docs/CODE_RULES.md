# Règles de code Candilog

Source de vérité **contractuelle** pour la qualité, l'architecture, les conventions et
les bonnes pratiques. Ces règles s'appliquent à tout nouveau code, toute modification et
tout refactoring. Ce ne sont pas des recommandations.

Avant toute modification significative : lire ce fichier, plus `docs/ARCHITECTURE.md` et
`docs/DESIGN.md` si l'écran ou les couches sont concernés. `AGENTS.md`, à la racine, est
le point d'entrée court des agents IA et renvoie ici — il ne remplace pas ce document.

En cas de conflit avec une autre règle du dépôt, signaler le conflit et appliquer la
variante la plus sûre, idiomatique, et alignée sur le code existant. Ne pas inventer une
couche, une bibliothèque ou un script qui n'existe pas.

---

## 0. Pile technique (ne pas substituer)

| Couche | Réalité du projet |
| --- | --- |
| Shell | Tauri 2 |
| UI | React 19, TypeScript strict, Vite, Tailwind 4 |
| Formulaires | React Hook Form + Zod 4 |
| État serveur | TanStack Query 5 |
| État UI transverse | Zustand (`src/shared/lib/ui-store.ts`) |
| Backend | Rust 2021, crate `candilog_lib` |
| SQL | SQLite via **rusqlite** + r2d2 — **pas sqlx** |
| IPC | Commandes Tauri, JSON `snake_case`, types **ts-rs** |
| Design system | `src/shared/ui/` — **pas shadcn / Radix** |
| Tests | Vitest + Testing Library ; tests Rust colocalisés |
| Qualité Rust | `#![deny(clippy::unwrap_used)]`, clippy `-D warnings`, cargo-deny |

Les types TypeScript d'IPC sont générés dans `src/shared/types/generated/`. Ne jamais les
éditer à la main : modifier le Rust, puis régénérer.

Consulter la documentation officielle **de la version réellement installée**
(`package.json`, `src-tauri/Cargo.toml`) lorsque l'API compte. Context7, s'il est
disponible, sert à ça — pas à importer un autre stack.

---

## 1. Langue

### Français uniquement

- textes visibles dans l'interface ;
- messages destinés à l'utilisateur (erreurs Zod, toasts, `AppError` côté IPC) ;
- commentaires de code ;
- documentation fonctionnelle ;
- messages de commit Git (préfixe Conventional Commits en anglais, description en français).

```tsx
<Button>Nouvelle candidature</Button>
```

```rust
// Vérifie que la candidature existe avant la modification.
```

```text
feat: ajoute l'inspector des candidatures
```

### Anglais obligatoire pour tout identifiant technique

Fichiers, dossiers, variables, constantes, fonctions, composants, hooks, stores, props,
types, traits, commandes Tauri, événements, tests, schémas.

```ts
// Interdit
const candidatureSelectionnee = ...
function ajouterCandidature() {}

// Correct
const selectedApplication = ...
function createApplication() {}
```

**Exception déjà en production (ne pas « corriger » sans migration) :**

- champs IPC / DTO / formulaires alignés sur ts-rs : **`snake_case`** (`job_title`, pas `jobTitle`) ;
- valeurs d'enums persistées : catalogue métier existant (`EN_ATTENTE`, `CDI`, `Présentiel`) ;
- noms de types et de variants Rust / TypeScript : anglais (`ApplicationStatus`).

Les identifiants français encore présents dans le code (`signalerEchec`, `valider`,
`textFacultatif`, événement `ia-progression`) sont des dettes. Tout **nouveau** symbole
est en anglais.

---

## 2. Nommage

Respecter l'idiome de chaque langage. Pas de convention globale artificielle.

### TypeScript / React

| Quoi | Convention | Exemple |
| --- | --- | --- |
| Composants, types, interfaces, enums | `PascalCase` | `ApplicationDetail` |
| Variables, fonctions, hooks, props, state | `camelCase` | `selectedApplication`, `useApplications` |
| Hooks | préfixe `use` | `useApplicationsViewModel` |
| Fichiers composants | `PascalCase.tsx` | `KanbanBoard.tsx` |
| Pages | `*Page.tsx` | `ApplicationsPage.tsx` |
| Modales de formulaire | `*FormModal.tsx` | `CompanyFormModal.tsx` |
| Hooks / ViewModels | `useSomething.ts` | `useContactsViewModel.ts` |
| Services frontend | `camelCase` + `Service` | `applicationService.ts` |
| Schémas Zod de formulaire | `kebab-case.schema.ts` | `application-form.schema.ts` |
| Lib / stores partagés | `kebab-case.ts` | `ui-store.ts`, `zod-helpers.ts` |
| Dossiers métier | anglais, minuscules | `view/`, `viewmodel/`, `model/`, `services/` |

Champs d'un DTO généré ou d'un schema qui part vers IPC : **garder le `snake_case` Rust**.
Ne pas mapper `job_title` → `jobTitle` dans React.

### Rust

| Quoi | Convention | Exemple |
| --- | --- | --- |
| Fonctions, variables, modules, fichiers | `snake_case` | `create_application` |
| Structs, enums, traits | `PascalCase` | `ApplicationRepository` |
| Constantes | `SCREAMING_SNAKE_CASE` | `MAX_PAGE_SIZE` |
| Commandes Tauri | `snake_case`, préfixe de domaine | `applications_list_page` |

---

## 3. Principes de qualité

Ordre de priorité :

```text
simplicité > lisibilité > maintenabilité > abstraction
```

Entre un code très abstrait et un code simple, lisible et testé : **toujours le second**.

- Une abstraction n'existe que pour un problème concret. Pas de factory, wrapper, generic
  repository ou héritage « pour faire propre ».
- Une duplication simple est préférable à une mauvaise abstraction.
- Chaque unité (fonction, composant, hook, service, commande, module) a **une**
  responsabilité claire. Ne pas mélanger fetch + validation + persistence + UI.
- Découper quand la compréhension, le test ou la modification deviennent difficiles.
  Ne pas découper une fonction simple juste pour réduire le nombre de lignes.
- Frontières de feature nettes : ne pas dépendre des détails internes d'une autre feature.
- Pas de code mort, d'implémentation commentée, d'import inutilisé. Git conserve l'historique.
- `TODO` / `FIXME` / `HACK` exceptionnels, précis, jamais un substitut à finir le travail.

---

## 4. Architecture frontend

Feature-first. Ne pas recréer des fourre-tout globaux `components/`, `hooks/`, `utils/`,
`services/` qui mélangent tous les domaines.

```text
Vue (features/<domaine>/view)
  → ViewModel (viewmodel/use*ViewModel.ts)
    → *Service.ts
      → ipc() dans src/shared/services/ipc.ts
        → commande Tauri
```

| Couche | Rôle | Emplacement |
| --- | --- | --- |
| Pages / UI métier | Rendu, interactions | `features/<domaine>/view/` |
| ViewModel | Query, mutations, toasts, sélection | `features/<domaine>/viewmodel/` |
| Service frontend | Appels IPC typés uniquement | `features/<domaine>/services/` |
| Schémas / modèle UI | Zod, helpers de feature | `features/<domaine>/model/` |
| UI partagée | Design system | `src/shared/ui/` |
| Infra partagée | dates, store UI, erreurs | `src/shared/lib/`, `shared/types/` |

Les vues et ViewModels n'importent **jamais** `invoke`. ESLint l'interdit. Tout passe par
`ipc()`.

**UI partagée vs UI de feature :** `Button`, `ModalHost`, `Inspector`, `SplitPane`,
`DataTable` restent dans `shared/ui`. `ApplicationDetail`, `KanbanBoard`, `ProfileUi`
restent dans leur feature. Ne pas tout pousser dans `shared` « au cas où ».

Zustand ne duplique pas les données serveur. TanStack Query est la cache métier.
Le ViewModel n'est pas un second backend TypeScript.

---

## 5. Architecture Rust

```text
Commande Tauri (presentation/commands.rs)
  → Service (application/)
    → Trait repository (domain/)
      → SQLite / filesystem / HTTP (infrastructure/)
```

Le `domain` n'importe ni Tauri ni rusqlite.

Les commandes restent fines : `State`, éventuellement `AppHandle`, délégation au service
(`blocking::execute` pour le synchrone, `async` pour l'IA / HTTP). Pas de SQL, pas de
prompt IA, pas de gros traitement dans `commands.rs`.

Ne pas inventer une couche DTO parallèle. Les structs de `domain/` annotées `Serialize`,
`Deserialize` et `ts_rs::TS` **sont** le contrat IPC. Créer un type plus étroit (`New*`,
filtre, page) quand le modèle interne complet est trop large — pas un `*Dto` générique
pour toute l'application.

`AppError` / `AppResult` sont le canal d'erreur unique. La variante IPC est
`AppErrorDto { code, message }` ; le détail technique reste dans les logs.

---

## 6. TypeScript

`tsconfig` est strict (`strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
etc.). Le respecter.

Interdit sauf justification locale documentée :

- `any`
- `as unknown as T` pour faire taire le compilateur
- `!` non justifié
- `@ts-ignore` / `@ts-expect-error` de confort
- `eslint-disable` pour passer la CI

Le type doit décrire les données réellement manipulées. Les payloads IPC suivent les
types générés, y compris `null` vs `undefined` et les dates ISO en `string`.

---

## 7. React

Bonnes pratiques React 19 du projet :

- state dérivable → le dériver, pas un `useEffect` de synchronisation ;
- pas de state dupliqué (liste Query + copie locale « source de vérité ») ;
- nettoyer listeners, timers, abonnements Tauri (`listen` → `unlisten`) ;
- `useMemo` / `useCallback` seulement s'il y a une raison mesurable (identité passée à
  un enfant coûteux, dépendance d'effet), pas par réflexe ;
- composition plutôt que composants-dieu.

Les écrans volumineux (listes + inspecteur + génération IA) doivent rester découpés :
page d'orchestration, composants de feature, ViewModel, service.

---

## 8. Formulaires et Zod

Tout formulaire React d'édition d'entité utilise **React Hook Form + Zod**
(`zodResolver`). Pas de validation manuelle dispersée dans le JSX.

```ts
export const applicationFormSchema = z.object({
  job_title: z.string().trim().min(1, "Le poste est obligatoire"),
  // ...
});

export type ApplicationFormValues = z.output<typeof applicationFormSchema>;
export type ApplicationFormInput = z.input<typeof applicationFormSchema>;
```

Utiliser `z.output` / `z.input` (pas `z.infer`) dès qu'il y a `transform` : RHF manipule
l'entrée, le service IPC reçoit la sortie.

Le schema :

- vit dans la feature (`model/schemas/*.schema.ts` ou équivalent) ;
- est réutilisable et **testé** ;
- reprend les règles métier du service Rust correspondant (poste requis, URL HTTP(S), …) ;
- produit des messages d'erreur en français.

Les filtres de liste peuvent rester hors RHF s'ils ne sont pas un formulaire d'entité,
mais ils ont quand même un schema Zod testé.

---

## 9. Validation Rust et cohérence des types

La validation Zod n'est **pas** une frontière de sécurité. Toute entrée IPC est non fiable.

Le service Rust revalide les champs critiques (requis, URL, chemins, limites, enums)
**avant** repository, filesystem, SQL ou HTTP.

Chaîne de vérité :

```text
SQLite → modèles Rust → IPC (ts-rs) → TypeScript généré → Zod → React
```

Surveiller : optionnel vs requis, `null` / `undefined`, enums, dates ISO, nombres,
noms de champs. Ne pas maintenir à la main un second type TS incompatible avec ts-rs.

Helpers partagés : `core::utils::validation` (`validate_optional_http_url`,
`validate_user_file_path`). Réutiliser avant d'en inventer.

---

## 10. Tauri et IPC

Tauri 2, principe du moindre privilège. Les capabilities actuelles n'exposent pas le
filesystem générique : I/O fichiers via commandes Rust + chemins validés.

- commandes enregistrées explicitement dans `app/bootstrap.rs` ;
- JSON et arguments en `snake_case` ;
- état partagé dans `AppState` (`Arc<Service>`) ;
- événements : nom anglais pour tout **nouvel** événement ; payload typé.

Toute entrée React (IDs, URLs, chemins, enums, textes, limites, SQL params) est validée
côté Rust. TypeScript n'est pas une frontière de sécurité.

`validate_user_file_path` : rejeter vide, `\0`, `..`. Ne pas interpoler un chemin
utilisateur dans une commande système.

---

## 11. SQL (rusqlite)

Requêtes **paramétrées** (`?1`, `params!`, `params_from_iter`). Interdiction de concaténer
une entrée utilisateur dans le SQL. Un `ORDER BY` dynamique n'accepte qu'un enum fermé
(`sort_column`), jamais une chaîne libre.

- colonnes explicites : pas de `SELECT *` ;
- filtre, tri, recherche, pagination, agrégation **dans SQLite**, pas « tout charger →
  filtrer en Rust → paginer en React » ;
- `LIMIT` / `OFFSET` via `core/pagination` (`Page<T>`, taille bornée) ;
- transactions dès qu'une suite d'écritures doit être atomique (candidature + historique
  de statut, entretien + statut, restore) ;
- index justifiés par des `WHERE` / `JOIN` / `ORDER BY` réels, pas ajoutés par réflexe ;
- traquer les N+1 (JOIN, `IN`, batch) ;
- `EXPLAIN` / `EXPLAIN ANALYZE` sur un environnement sûr pour les requêtes sensibles.

Helpers : `core/database/helpers.rs` (`like_contains`, `translate_error`, …).

---

## 12. IA

L'IA vit en Rust (`features/ai`). Le frontend n'envoie que des DTO (`ResumeGenerationRequest`,
`generation_id`) et écoute la progression. **Aucun prompt dans React.**

Concevoir pour : rapidité, robustesse, économie de tokens, maintenabilité.

- centraliser les prompts (constantes / module dédié du service IA) ;
- séparer **instructions** et **données** (`bloc_donnees` / `DONNEES_NON_FIABLES`) ;
- une offre d'emploi ou un PDF est de la **DATA**, jamais des INSTRUCTIONS (prompt injection) ;
- ne pas envoyer de contexte inutile ; un appel structuré plutôt que plusieurs allers-retours ;
- sortie structurée : `parse → validate → use` (JSON repair, deserializers, puis grounding) ;
- ne jamais faire confiance au JSON brut du modèle ;
- CV / lettres : interdiction d'inventer expérience, entreprise, diplôme, compétence, date,
  certification — le grounding Rust (`ground_generated_resume`) n'est pas optionnel ;
- score ATS déterministe en Rust, pas un chiffre LLM pris tel quel ;
- opérations longues : pas de blocage UI, état de chargement, timeout, erreur provider,
  anti double-soumission, annulation (`CancellationToken` + `ai_cancel`) ;
- bornes déclarées dans `features/ai/domain/validation.rs` (`MAX_SOURCE_CHARS`,
  `MAX_CONTEXT_CHARS`, `MAX_STRUCTURED_CHARS`, `MAX_ITEMS`, `MAX_ITEM_CHARS`), HTTPS hors
  Ollama, rejet des IP privées pour les endpoints distants.

---

## 13. Erreurs, logs, ressources

Les erreurs ne sont pas avalées. Pas de `catch(() => {})` sans commentaire justifiant un
cas réellement silencieux (thème à la première ouverture, revue navigateur sans Tauri).

Propagation :

```text
SQLite → repository → service → AppError → IPC → AppError TS → toast / ErrorBanner
```

L'utilisateur voit un message français compréhensible. Les détails sensibles (chemins
internes, clés, stack) restent dans `tracing`, pas dans l'UI.

Rust : conserver des erreurs typées (`Validation`, `NotFound`, `Database`, `Provider`,
`Cancelled`, …) aussi longtemps que possible. Ne pas tout réduire en `String` trop tôt.

Ne jamais logger : mot de passe, token, clé API, secret, contenu personnel inutile.

Pas de `console.log` / `dbg!` / `println!` de debug dans le code livré. `tracing` côté
Rust ; toasts / `ErrorBanner` côté UI.

Nettoyer : listeners, timers, `listen` Tauri, tâches async, fichiers, connexions, caches
bornés. Aucune collection globale à croissance infinie.

---

## 14. Sécurité Rust

- zéro `unsafe` sauf nécessité réelle, minimale, commentée, justifiée — jamais pour
  contourner l'ownership ;
- `unwrap` / `expect` / `panic!` interdits dans les chemins applicatifs (clippy `deny`) ;
  autorisés dans les tests comme assertions ;
- secrets dans le keyring, pas dans SQLite ni les logs ;
- chemins utilisateur validés ; data dir `chmod` restreint en release Unix ;
- `cargo-deny` (`deny.toml`) lors d'un changement de dépendances ;
- pas de commande shell construite avec une entrée utilisateur.

---

## 15. Tests

Toute logique non triviale est testée : métier, transformations, validation, parsing,
calculs, services, repositories, ViewModels, ATS, prompts / préparation IA, mapping DTO,
branches d'erreur.

Couvrir au minimum : succès, erreur, cas limites, entrée invalide, donnée manquante.

Les tests vérifient des **comportements**, pas des détails d'implémentation.

Exception : pas de test pour un getter trivial ou un composant purement présentatif.
La couverture protège la logique, elle ne vise pas 100 % artificiel.

Emplacements :

- frontend : `__tests__/` colocalisé, Vitest ;
- Rust : modules `#[cfg(test)]` / `tests/` colocalisés ; SQLite **in-memory**, jamais la
  base utilisateur (`docs/DATA.md`).

**Non-régression :** un bug reproductible automatiquement → test qui échoue d'abord →
correctif → test vert.

Les chaînes de commandes IPC frontend sont synchronisées avec Rust
(`commandes-ipc.test.ts`). Préserver ce contrat.

---

## 16. Dépendances

Toute nouvelle dépendance est justifiée :

1. le projet a-t-il déjà une solution ?
2. quelques lignes suffisent-elles ?
3. crate / package maintenu, sans avis de sécurité bloquant ?
4. impact bundle / binaire ?

Pas de bibliothèque lourde pour un helper de vingt lignes. Frontend : pas de second
design system. Rust : pas de sqlx « en plus » de rusqlite.

---

## 17. Performance

Éviter d'introduire : calculs inutiles, IPC excessif, SQL répétitif, re-renders de liste
entière, clones / allocations évitables, parsing répété, appels IA redondants, lecture
de fichier complet si un extrait suffit.

Pas de micro-optimisation prématurée. Paginer et filtrer près de SQLite. Les listes
métier (candidatures, entreprises, contacts) sont déjà paginées : ne pas casser ce
modèle.

---

## 18. Git et commentaires

Commits **en français**, préfixe Conventional Commits :

```text
feat: ajoute le panneau de détail des candidatures
fix: corrige la validation du formulaire de profil
refactor: simplifie le service de génération de CV
test: ajoute les tests du calcul ATS
```

Ne pas committer sauf demande explicite.

**Aucune ligne d'attribution d'outil** dans un message de commit ni dans une description de
Pull Request : ni `Co-authored-by:` nommant un assistant, ni lien de session, ni mention
« Generated with ». Les commits sont signés par la personne qui en répond ; l'outil qui a
tenu le clavier ne fait pas partie du contrat. Un `commit-msg` versionné
(`.githooks/commit-msg`, activé par `git config core.hooksPath .githooks`) retire ces lignes
même quand la consigne est ignorée.

Commentaires en français, expliquant le **pourquoi**. Pas de paraphrase du code.

```ts
// Interdit : // Incrémente le compteur.
// Acceptable : // Plafonné pour rester compatible avec l'ancien calcul ATS.
```

---

## 19. Aucun contournement

Une erreur lint / TypeScript / clippy / test se corrige à la cause.

Interdit comme rustine de CI : `@ts-ignore`, `eslint-disable`, `as any`,
`#[allow(...)]` sans nécessité locale documentée.

---

## 20. Validation avant fin de tâche

Exécuter **uniquement** les commandes qui existent. Ne pas inventer `npm run format`.

### Frontend

```bash
npm run lint
npm test
npm run build
```

(`build` inclut `tsc --noEmit`.)

### Rust

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
```

Audit d'éventuelles nouvelles crates :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check
```

Le job `quality` du workflow de release (`.github/workflows/release.yml`) rejoue ces
vérifications avant d'autoriser les builds multi-plateformes. Ce filet de publication ne
remplace pas le contrôle local : une tâche ne se termine pas en déléguant sa validation au
prochain push sur `master`.

---

## 21. Autres documents du dépôt

| Fichier | Autorité |
| --- | --- |
| `docs/CODE_RULES.md` | qualité, conventions, tests, sécurité du code |
| `docs/ARCHITECTURE.md` | couches et frontières |
| `docs/DESIGN.md` | UI, jetons, interdits visuels |
| `docs/DATA.md` | SQLite, schéma, relations, chemins de données |
| `docs/AI.md` | providers, prompts, cache, annulation IA |
| `docs/DEVELOPMENT.md` | prérequis, exécution, régénération des types, validations |
| `docs/RELEASES.md` | publication des binaires et politique `cargo-deny` |
| `THIRD_PARTY_NOTICES.md` | attribution des composants tiers redistribués avec les binaires |
| `SECURITY.md` | signalement d'une vulnérabilité, périmètre et délais |
| `CHANGELOG.md` | journal des versions publiées |
| `AGENTS.md` | point d'entrée des agents IA (renvoie vers ce tableau) |

`DESIGN.md` prime sur l'apparence. `ARCHITECTURE.md` prime sur le placement des couches.
Le présent fichier prime sur le style de code, la validation, les tests et la sécurité.

**Choix du dépôt, souvent contredits par les habitudes générales :** SQL via rusqlite et
non sqlx ; `z.input` / `z.output` plutôt que `z.infer` ; pas de couche DTO distincte des
structs `domain` annotées ts-rs. Ces choix l'emportent sur toute convention externe.

---

## 22. Documentation

Toute modification qui change le comportement, l'architecture, la configuration, les
commandes de développement, une API publique ou une fonctionnalité documentée met à jour
la documentation correspondante **dans la même tâche**. Une modification interne sans
impact documentaire ne doit produire aucun changement Markdown.

Documenter l'architecture, les responsabilités, les conventions, les commandes et les
décisions — pas l'implémentation ligne par ligne : le code reste la source de vérité des
détails. Ne jamais documenter une commande, un script ou un fichier qui n'existe pas.
