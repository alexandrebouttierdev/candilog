# Candilog Desktop

Suivi de candidatures, entretiens et relances, avec assistance IA locale.
Application native **Tauri 2 + React + TypeScript**, base SQLite locale.

- Audit et plan de migration : [`docs/migration/01-AUDIT.md`](docs/migration/01-AUDIT.md)
- Source de vérité du design : [`SPECDESIGN/`](SPECDESIGN/)

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
npm run build
npm run lint
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo deny --manifest-path src-tauri/Cargo.toml check
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
`PRAGMA user_version`. Le schéma est celui de Candilog Desktop historique : les données
existantes sont conservées.

`CANDILOG_DATA_DIR` déplace le dossier de données. En développement, la base vit sous
`src-tauri/.candilog-dev/` (ancrée sur le manifeste Cargo) et ne touche jamais aux
données réelles. `RUST_LOG` règle le niveau de journalisation (`candilog=info` par défaut).

## Documentation

L'architecture, le modèle de données et le processus de publication sont décrits dans
[`docs/`](docs/). Les binaires sont publiés sur le dépôt dédié
`alexandrebouttierdev/candilog-releases`, déclenché à chaque push sur `master` de ce
dépôt source — voir [`docs/RELEASES.md`](docs/RELEASES.md).

## Licence

Candilog est un projet **source available avec double licence**.

### Usage non commercial

Le code Candilog est mis à disposition sous la **PolyForm Noncommercial License 1.0.0** pour les usages autorisés par cette licence. Consultez le [texte officiel](./LICENSE), qui reste la référence.

### Usage commercial

Toute utilisation commerciale nécessite une licence commerciale séparée, accordée explicitement par le titulaire des droits. Consultez [`COMMERCIAL_LICENSE.md`](./COMMERCIAL_LICENSE.md) pour comprendre la démarche ; ce document n'accorde pas à lui seul de droits commerciaux.

### Contributions

Les contributions sont les bienvenues lorsqu'elles respectent les règles décrites dans [`CONTRIBUTING.md`](./CONTRIBUTING.md) et le mécanisme de contribution et de licence prévu par le projet, notamment le [`CLA.md`](./CLA.md) lorsqu'il est requis.

Copyright © 2026 Alexandre Bouttier
