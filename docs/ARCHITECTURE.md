# Architecture native Candilog

Candilog est une application desktop Rust. Iced rend l'interface et Tokio exécute les opérations longues. Aucun navigateur embarqué ni pont IPC n'est présent.

```text
Vue Iced → Message → app/update.rs → Task Tokio
                                  → Service métier
                                  → Repository
                                  → SQLite
```

- `app/` : état global, messages, mise à jour, abonnements et dispatch des vues.
- `core/` : chemins multiplateformes, PDF, updater et fonctions système.
- `modules/` : domaines métier autonomes et moteur IA.
- `navigation/` : enum `Route` des douze pages.
- `shared/` : base, erreurs, HTTP, LLM, profil, secrets et validation.
- `ui/` : design system et composants Iced.
- `migrations/` : schéma SQLite historique embarqué.

Chaque domaine sous `src/modules/` expose des répertoires dédiés :

- `views/` pour ses écrans Iced ;
- `components/` pour ses composants visuels réutilisables ;
- `dtos/` avec un contrat d'entrée par fichier ;
- `tests/` avec un fichier Rust distinct par cas de test et des `mod.rs` réservés aux helpers.

`src/app/view.rs` reste l'orchestrateur du shell, des modales et des helpers partagés. Les onze
écrans fonctionnels sont physiquement inclus depuis les dossiers `views/` de leurs domaines.

Les vues ne contiennent aucun SQL. Les services n'importent pas Iced. Les providers IA sont derrière `LlmProvider`. La clé API reste dans le coffre natif via `keyring`.
