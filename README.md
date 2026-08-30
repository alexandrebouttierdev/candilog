# Candilog

Application desktop de suivi de recherche d'emploi : candidatures, entreprises, contacts,
entretiens, relances, CV et lettres de motivation, avec assistance IA locale ou distante.

**Tauri 2 + React 19 + TypeScript** pour l'interface, **Rust** pour le métier, **SQLite**
local pour les données. Tout reste sur la machine de l'utilisateur : aucune donnée n'est
envoyée ailleurs, hors appel explicite à un fournisseur IA distant configuré par
l'utilisateur.

Version 0.0.1. Le dépôt contient également [`website/`](website/), le site public
candilog.fr, projet Next.js autonome.

## Fonctionnalités

- Suivi des candidatures : kanban ou table, filtres et recherche côté base, historique de
  statut, export CSV.
- Répertoire d'entreprises et de contacts, avec héritage des valeurs de l'entreprise
  (ville, adresse, type) sur la candidature.
- Calendrier des entretiens et des relances.
- Profil professionnel, génération de CV et de lettres de motivation en PDF A4 une page,
  analyse ATS déterministe.
- Fournisseurs IA au choix : Ollama (local), Claude, OpenAI, Gemini, Mistral, Nvidia ou
  point de terminaison personnalisé. La clé API vit dans le coffre du système.
- Sauvegarde et restauration de la base, mise à jour assistée depuis les GitHub Releases.

## Démarrer

Prérequis : Node.js LTS, Rust 1.91, et les dépendances système de Tauri 2. Détails dans
[`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md).

```bash
npm install
npm run tauri dev
```

Le frontend seul (sans fenêtre native, IPC indisponible) :

```bash
npm run dev
```

## Validations

Aucune CI ne vérifie la qualité — le seul workflow publie les releases. Ces commandes sont
à lancer localement.

```bash
npm run lint
npm test
npm run build            # inclut tsc --noEmit

cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
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
├── model/       types, schémas Zod, constantes métier
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
core/             config, base de données, erreurs, journal, pagination,
                  sauvegardes, mises à jour, coffre à secrets
features/<f>/     domain · application · infrastructure · presentation
infrastructure/   export PDF (CV et lettres)
migrations/       schéma SQLite embarqué (init_schema.sql)
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

SQLite, `rusqlite` + `r2d2`, schéma embarqué dans le binaire et appliqué par
`PRAGMA user_version`. Une base d'une génération antérieure est refusée en lecture seule
plutôt que migrée automatiquement.

`CANDILOG_DATA_DIR` déplace le dossier de données. En développement, la base vit sous
`src-tauri/.candilog-dev/` (ancrée sur le manifeste Cargo) et ne touche jamais aux
données réelles. `RUST_LOG` règle le niveau de journalisation (`candilog=info` par défaut).

## Documentation

| Document | Sujet |
| --- | --- |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Couches et frontières |
| [`docs/CODE_RULES.md`](docs/CODE_RULES.md) | Qualité, conventions, tests, sécurité |
| [`docs/DESIGN.md`](docs/DESIGN.md) | Design system de l'application |
| [`docs/DATA.md`](docs/DATA.md) | Schéma SQLite, référentiels, chemins de données |
| [`docs/AI.md`](docs/AI.md) | Fournisseurs IA, streaming, annulation |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | Installation, exécution, validations |
| [`docs/RELEASES.md`](docs/RELEASES.md) | Publication des binaires |

Les binaires sont publiés en GitHub Release sur ce dépôt à chaque push sur `master`.

### Agents IA

[`AGENTS.md`](AGENTS.md) est la référence commune (Codex, OpenCode, Cursor et tout agent
lisant ce format) ; [`CLAUDE.md`](CLAUDE.md) est le point d'entrée Claude Code et
l'importe. Des fichiers `AGENTS.md` imbriqués précisent les règles propres à
[`src/`](src/AGENTS.md), [`src-tauri/`](src-tauri/AGENTS.md) et
[`website/`](website/AGENTS.md).

## Licence

Candilog est un projet **source available avec double licence**.

### Usage non commercial

Le code Candilog est mis à disposition sous la **PolyForm Noncommercial License 1.0.0** pour les usages autorisés par cette licence. Consultez le [texte officiel](./LICENSE), qui reste la référence.

### Usage commercial

Toute utilisation commerciale nécessite une licence commerciale séparée, accordée explicitement par le titulaire des droits. Consultez [`COMMERCIAL_LICENSE.md`](./COMMERCIAL_LICENSE.md) pour comprendre la démarche ; ce document n'accorde pas à lui seul de droits commerciaux.

Les licences des dépendances tierces sont traitées dans [`LICENSES.md`](./LICENSES.md).

### Contributions

Les contributions sont les bienvenues lorsqu'elles respectent les règles décrites dans [`CONTRIBUTING.md`](./CONTRIBUTING.md) et le mécanisme de contribution et de licence prévu par le projet, notamment le [`CLA.md`](./CLA.md) lorsqu'il est requis.

Copyright © 2026 Alexandre Bouttier
