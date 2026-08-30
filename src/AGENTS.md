# Frontend — règles spécifiques

Complète `AGENTS.md` à la racine (règles générales) et `docs/CODE_RULES.md` (contrat de
qualité). Ici uniquement ce qui est propre à `src/`.

## Couches d'une feature

```text
features/<domaine>/
├── model/       types UI, schémas Zod (*.schema.ts), constantes métier
├── view/        pages (*Page.tsx) et composants
├── viewmodel/   hooks use*ViewModel.ts — Query, mutations, sélection, toasts
└── services/    <domaine>Service.ts — seul module connaissant les commandes Tauri
```

Sens des dépendances : `view` → `viewmodel` → `services` → `ipc()`. Jamais l'inverse, et
jamais de raccourci d'une vue vers un service.

Une feature ne dépend pas des internes d'une autre : passer par son `index.ts` quand il
existe.

## IPC

- Tout appel passe par `ipc()` de `@/shared/services/ipc` ; `invoke` y est confiné et
  ESLint bloque l'import ailleurs.
- Les arguments et champs de payload restent en `snake_case` (contrat Rust). Ne pas
  convertir `job_title` en `jobTitle`.
- Les types viennent de `@/shared/types/generated/` : lecture seule, régénérés par
  `cargo test --manifest-path src-tauri/Cargo.toml`. Un DTO Rust modifié sans
  régénération fait échouer `npm run build`.
- Les erreurs arrivent déjà normalisées en `AppError { code, message }` : brancher sur
  `code`, afficher `message`.

## UI

- Importer depuis `@/shared/ui` (barrel `index.ts`). Ajouter un composant partagé implique
  de l'exporter là.
- `docs/DESIGN.md` fait autorité : pas de nouvelle couleur, de nouveau rayon ni de
  variante de bouton. Aucun hexadécimal ni `rgb()` dans le TSX — uniquement les classes de
  thème (`bg-surface`, `text-ink`, `border-line`).
- Un composant **métier** reste dans sa feature (`ApplicationCard`, `ProfileUi`) ; seul le
  générique va dans `shared/ui`.
- Vérifier les deux thèmes (clair et sombre) : les classes sémantiques suffisent, pas de
  `bg-white`.

## État

- Données serveur : TanStack Query. Ne pas recopier une réponse de query dans un `useState`.
- `src/shared/lib/ui-store.ts` (Zustand) ne porte que l'état UI transverse — jamais des
  données métier.
- Filtre, recherche, tri et pagination sont des **paramètres de requête backend**, pas un
  `.filter()` sur la page affichée.

## Formulaires

React Hook Form + `zodResolver`. Le schéma vit dans `model/schemas/<nom>.schema.ts`, est
testé, reprend les règles du service Rust correspondant et produit des messages en
français. Utiliser `z.input` / `z.output` (pas `z.infer`) dès qu'il y a un `transform`.

## Tests

Vitest + Testing Library, dans un dossier `__tests__/` colocalisé. Tester des
comportements observables, pas l'implémentation. `src/shared/services/__tests__/commandes-ipc.test.ts`
compare automatiquement les noms de commandes appelés par les services aux
`#[tauri::command]` déclarés en Rust : une faute de frappe dans `ipc("…")` y échoue au
lieu de produire un écran vide à l'exécution. Ne pas contourner ce test.

## Validation

```bash
npm run lint
npm test
npm run build
```
