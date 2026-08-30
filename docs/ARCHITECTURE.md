# Architecture native Candilog

Candilog est une application desktop Tauri 2. React rend l'interface ; Rust porte le métier,
SQLite et l'IA. Le pont IPC est le seul contrat entre les deux côtés.

L'interface suit `docs/DESIGN.md` : ne pas inventer de style en dehors de ce système.

```text
Vue React → ViewModel (hook) → service frontend → invoke
                                                    ↓
                                              Commande Tauri
                                                    ↓
                                              Service métier
                                                    ↓
                                              Repository
                                                    ↓
                                              SQLite
```

- `src/` : frontend feature-first (`model`, `view`, `viewmodel`, `services`), design system
  dans `src/shared/ui/`.
- `src-tauri/src/app/` : état partagé (`AppState`) et démarrage — `bootstrap.rs` enregistre
  toutes les commandes.
- `src-tauri/src/core/` : chemins, base de données, erreurs, journal, pagination,
  sauvegardes, mises à jour, coffre à secrets, validation partagée.
- `src-tauri/src/features/` : domaines métier (`domain`, `application`, `infrastructure`,
  `presentation`). L'IA vit dans `features/ai/` (prompts, providers HTTP, extraction PDF,
  scoring ATS).
- `src-tauri/src/infrastructure/` : PDF d'export (CV et lettres).
- `src-tauri/migrations/` : schéma SQLite embarqué (`init_schema.sql`, `PRAGMA user_version`).

Les accès au système restent natifs : dialogues de fichier (`core/files.rs`) et lecture du
presse-papiers (`core/clipboard.rs`, commande `documents_read_clipboard`). La webview
n'expose ni l'un ni l'autre, et aucune permission large n'est ouverte côté capacités.

Le `domain` n'importe ni Tauri ni rusqlite. Les vues n'appellent jamais `invoke` directement :
elles passent par `src/shared/services/ipc.ts`. Les providers IA sont derrière `LlmProvider`.
La clé API reste dans le coffre natif via `keyring`.

## Événements

Les traitements longs remontent leur progression par événements Tauri, émis depuis la
couche `presentation` de la feature concernée :

| Événement | Émetteur |
| --- | --- |
| `ia-progression` | `features/ai` — génération et analyse |
| `profile_import_progress` | `features/ai` — import de profil depuis un CV |
| `update-progress` | `features/settings` — téléchargement d'une mise à jour |

Quand une commande ouvre d'abord un sélecteur de fichier natif (import de CV), son premier
événement n'est émis qu'une fois le fichier choisi : l'interface s'en sert pour distinguer
la sélection du traitement et ne jamais annoncer une analyse qui n'a pas commencé.

`core/events/` reste un module de réservation, sans contenu.

## Le site

`website/` est le site public candilog.fr : un projet Next.js **autonome**, sans lien de
code avec l'application. Il ne partage ni dépendances, ni configuration TypeScript, ni
design system — voir `website/README.md` et `website/DESIGN.md`.

## Documents liés

`docs/CODE_RULES.md` (règles de code), `docs/DATA.md` (schéma et données),
`docs/AI.md` (IA), `docs/DESIGN.md` (interface), `docs/DEVELOPMENT.md` (commandes).
