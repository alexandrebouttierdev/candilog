# Architecture native Candilog

L'interface suit `docs/DESIGN.md` : ne pas inventer de style en dehors de ce système.

Candilog est une application desktop Tauri 2. React rend l'interface ; Rust porte le métier,
SQLite et l'IA. Le pont IPC est le seul contrat entre les deux côtés.

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

- `src/` : frontend feature-first (vues, ViewModels, services IPC).
- `src-tauri/src/app/` : état partagé (`AppState`) et démarrage.
- `src-tauri/src/core/` : chemins, base, erreurs, journal, événements, sauvegardes, mises à jour.
- `src-tauri/src/features/` : domaines métier (domain, application, infrastructure, presentation).
- `src-tauri/src/infrastructure/` : IA, PDF, HTTP, filesystem, coffre à secrets.
- `src-tauri/migrations/` : schéma SQLite historique embarqué.

Le `domain` n'importe ni Tauri ni rusqlite. Les vues n'appellent jamais `invoke` directement :
elles passent par `src/shared/services/ipc.ts`. Les providers IA sont derrière `LlmProvider`.
La clé API reste dans le coffre natif via `keyring`.
