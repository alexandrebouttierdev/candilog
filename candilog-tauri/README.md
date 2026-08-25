# Candilog — Tauri 2 + React

Réécriture de Candilog Desktop, migrée depuis l'application Rust/Iced qui vit toujours
sous `../src/` et sert de référence fonctionnelle jusqu'à la fin de la migration.

- Audit et plan : [`../docs/migration/01-AUDIT.md`](../docs/migration/01-AUDIT.md)
- Consignes de migration : [`../MIGRATION.md`](../MIGRATION.md)
- Source de vérité du design : [`../SPECDESIGN/`](../SPECDESIGN/)

## Démarrer

```bash
npm install
npm run tauri dev
```

Le frontend seul (sans fenêtre native, IPC indisponible) :

```bash
npm run dev
```

## Validations

```bash
npm run build                                     # tsc --noEmit + build Vite
npm run lint
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Architecture

```
React                          Tauri IPC                      Rust
─────                          ─────────                      ────
View                                                          Command
 ↓                                                             ↓
ViewModel (hook)                                              Application Service
 ↓                                                             ↓
Frontend Service ──────────────► invoke ──────────────────►   Domain
                                                               ↓
                                                              Repository (trait)
                                                               ↓
                                                              Infrastructure (SQLite, HTTP, IA)
```

### Frontend — `src/`

Feature-first + MVVM. Chaque feature est autonome :

```
features/<feature>/
├── model/       types, DTO, schémas Zod, mappers
├── view/        pages et composants React
├── viewmodel/   hooks d'orchestration UI
└── services/    seule couche connaissant les commandes Tauri de la feature
```

`shared/ui/` ne contient que du générique ; un composant métier reste dans sa feature.

Les appels IPC passent tous par `shared/services/ipc.ts` — une règle ESLint interdit
d'importer `invoke` ailleurs.

### Backend — `src-tauri/src/`

Architecture hexagonale pragmatique, feature-first :

```
app/              état partagé (AppState) et démarrage
core/             config, base de données, erreurs, journal, événements
features/<f>/     domain · application · infrastructure · presentation
infrastructure/   IA, PDF, HTTP, filesystem, coffre à secrets
```

Le `domain` ne dépend ni de Tauri, ni de rusqlite, ni d'un fournisseur IA.

### Contrat IPC

Les types TypeScript des DTO sont **générés** depuis Rust par `ts-rs` dans
`src/shared/types/generated/`. Ils ne s'éditent pas à la main : régénérer avec

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Un DTO Rust modifié sans régénération fait échouer `npm run build`.

### Erreurs

Le backend rejette toujours `{ code, message }` : `code` est stable et destiné au
branchement conditionnel, `message` est rédigé pour l'utilisateur. Le détail technique
part au journal, jamais à l'écran.

## Base de données

SQLite, `rusqlite` + `r2d2`, migrations embarquées dans le binaire et appliquées par
`PRAGMA user_version`. Le schéma est celui de l'application Iced : les données existantes
sont conservées.

`CANDILOG_DATA_DIR` déplace le dossier de données. En développement, la base vit sous
`.candilog-dev/` du répertoire courant et ne touche jamais aux données réelles.
