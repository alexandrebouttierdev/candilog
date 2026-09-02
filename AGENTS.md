# Instructions pour les agents IA

Référence commune du dépôt Candilog, valable pour tous les agents (Codex, OpenCode,
Cursor, Claude Code). Une seule vérité : ce fichier renvoie vers la documentation, il ne
la recopie pas.

## Projet

Candilog est une application **desktop Tauri 2** de suivi de recherche d'emploi
(candidatures, entreprises, contacts, entretiens, relances, CV et lettres) avec assistance
IA locale ou distante. React 19 + TypeScript pour l'interface, Rust pour le métier, SQLite
local pour les données. Tout reste sur la machine de l'utilisateur.

Le dépôt contient aussi `website/`, le site public candilog.fr (Next.js), **projet
autonome** avec ses propres dépendances, commandes et documentation.

## Documentation à consulter

Ne lire que ce qui sert à la tâche en cours.

| Tâche | Document |
| --- | --- |
| Toute modification de code | `docs/CODE_RULES.md` (règles contractuelles) |
| Placement des couches, frontières IPC | `docs/ARCHITECTURE.md` |
| Écran, composant, style | `docs/DESIGN.md` |
| Schéma SQLite, référentiels, chemins de données | `docs/DATA.md` |
| Providers IA, prompts, annulation | `docs/AI.md` |
| Installer, lancer, régénérer les types | `docs/DEVELOPMENT.md` |
| Publication d'une version | `docs/RELEASES.md` |
| Proposer une contribution | `CONTRIBUTING.md` |
| Attribution d'un composant tiers redistribué | `THIRD_PARTY_NOTICES.md` |
| Signalement d'une vulnérabilité | `SECURITY.md` |
| Travailler dans `website/` | `website/README.md` et `website/DESIGN.md` |

Règles propres à un périmètre : `src/AGENTS.md` (frontend), `src-tauri/AGENTS.md`
(natif), `website/AGENTS.md` (site). Ils ne contiennent que les différences.

## Architecture

```text
Vue React → ViewModel (hook) → service frontend → ipc() → commande Tauri
                                                              ↓
                                              Service Rust → Repository → SQLite
```

- `src/` — frontend feature-first (`features/<domaine>/{model,view,viewmodel,services}`),
  design system dans `src/shared/ui/`, IPC centralisé dans `src/shared/services/ipc.ts`.
- `src-tauri/src/` — `app/` (état, démarrage), `core/` (config, base, erreurs, journal,
  sauvegardes, mises à jour, secrets), `features/<domaine>/{domain,application,
  infrastructure,presentation}`, `infrastructure/pdf/`.
- `src-tauri/migrations/init_schema.sql` — schéma SQLite unique, appliqué par
  `PRAGMA user_version`.
- `src/shared/types/generated/` — types TypeScript **générés** par `ts-rs` depuis Rust.

## Règles absolues

1. **Ne jamais éditer `src/shared/types/generated/`** : modifier le Rust, puis régénérer
   avec `cargo test --manifest-path src-tauri/Cargo.toml`.
2. **Ne jamais appeler `invoke` hors de `src/shared/services/ipc.ts`** — ESLint l'interdit.
3. **Ne pas substituer la pile** : rusqlite (pas sqlx), design system maison (pas
   shadcn/Radix), Zustand pour l'UI seulement, TanStack Query pour les données serveur.
4. **Ne jamais masquer une erreur** : pas de `any`, `@ts-ignore`, `eslint-disable`,
   `#[allow(...)]` ni `unwrap` applicatif pour faire passer lint, typecheck ou CI. On
   corrige la cause.
5. **Le `domain` Rust n'importe ni Tauri ni rusqlite.**
6. **Toute entrée IPC est revalidée en Rust** : Zod n'est pas une frontière de sécurité.
7. **Aucune ligne d'attribution d'outil dans un commit ou une PR** : ni `Co-authored-by:`
   nommant un assistant, ni lien de session, ni « Generated with ». Un hook `commit-msg`
   versionné les retire de toute façon (`docs/CODE_RULES.md` §18).
8. **Aucune commande Git destructive** (`git reset --hard`, `git clean -fd`, `git
   checkout --` sur du travail non commité, `push --force`). Ne jamais commiter ni pousser
   sans demande explicite.

## Comportement attendu

- Préserver l'architecture existante ; ne pas la réécrire pour l'aligner sur une habitude
  personnelle.
- Réutiliser les composants, hooks, services et helpers existants avant d'en créer.
  Chercher un équivalent dans `src/shared/ui/`, `src/shared/lib/`,
  `src-tauri/src/core/utils/` avant d'écrire une nouvelle fonction.
- Préférer une duplication simple à une mauvaise abstraction ; pas de factory, wrapper ou
  generic repository « pour faire propre ».
- Ne pas ajouter de dépendance sans nécessité démontrée (voir `docs/CODE_RULES.md` §16).
- Ne modifier que les fichiers concernés par la tâche demandée.
- Interface, messages utilisateur, commentaires et commits en **français** ; identifiants
  de code en **anglais**, sauf les champs IPC/DTO en `snake_case` alignés sur Rust.
- Ajouter ou mettre à jour les tests pour tout comportement non trivial et tout bug
  corrigé (test rouge d'abord).
- Ne pas inventer une commande, un script ou un fichier qui n'existe pas dans le dépôt.

## Synchronisation documentation / code

Toute modification qui change le comportement, l'architecture, la configuration, les
commandes de développement, une API publique ou une fonctionnalité documentée met à jour
la documentation correspondante **dans la même tâche**. Une modification interne sans
impact documentaire ne doit provoquer aucun changement Markdown.

## Validation avant de terminer

Exécuter ce que la modification touche. Ces commandes existent réellement — il n'y a ni
`npm run format`, ni `npm run typecheck` à la racine, ni CI de qualité.

Frontend (`src/`) :

```bash
npm run lint
npm test
npm run build          # inclut tsc --noEmit
```

Natif (`src-tauri/`) :

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
```

Changement de dépendance Rust :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check
```

Documents générés (CV et lettres), après toute modification d'un gabarit, du moteur PDF ou
de la composition — la chaîne complète est décrite dans `docs/DEVELOPMENT.md` :

```bash
CANDILOG_E2E=1 cargo test --manifest-path src-tauri/Cargo.toml --locked --test e2e_documents
npm run e2e
```

Site (`website/`) : `npm run lint`, `npm run typecheck`, `npm run build` depuis `website/`.

Signaler explicitement toute vérification non exécutée et pourquoi.

## Sécurité Git

Inspecter `git status` avant un travail important et ne jamais écraser une modification
non commitée de l'utilisateur. La branche par défaut est `dev` ; `master` déclenche une
publication de release (voir `docs/RELEASES.md`).
