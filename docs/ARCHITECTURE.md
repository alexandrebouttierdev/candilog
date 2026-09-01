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

## CV ciblé — `ResumeWorkspace`

Après une génération IA, l'éditeur de CV ne dépend plus du profil ni du snapshot
`ResumeGeneration` : Rust compose un **`ResumeWorkspace`** (`schema_version = 1`) contenant
le document complet (`ResumeDocument`), l'offre structurée, l'analyse ATS, le score local
(`MatchScore`), le score initial et les propositions (`ResumeProposal`).

| Commande IPC | Rôle |
| --- | --- |
| `documents_resume_prepare` | Fige profil + `ResumeGeneration` en workspace autonome |
| `documents_resume_recalculate` | Revalide le document, recalcule score et propositions après édition manuelle |
| `documents_resume_apply_proposal` | Applique une proposition puis recalcule |
| `documents_resume_reject_proposal` | Refuse une proposition sans modifier le document, puis recalcule |
| `documents_resume_export_pdf` | Exporte un `ResumeDocument` (plus un `ResumeGeneration`) |

Toute validation, simulation de gain, construction des propositions et recalcul de score
vivent dans `features/documents/application/resume_workspace.rs`. L'interface ne fait que
transmettre le workspace courant et afficher le résultat.

Une compétence manquante peut être ajoutée au profil depuis une proposition :
`profile_add_skill` (`features/profile/presentation/commands.rs`) — sans doublon, après
confirmation utilisateur (« CV uniquement » ou « Ajouter au profil »).

Les anciens contenus `ResumeGeneration` restent lisibles en bibliothèque ; la conversion en
workspace n'a lieu qu'à la première ouverture en édition ou à l'export (`prepare_workspace`),
sans réécrire silencieusement la bibliothèque.

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
