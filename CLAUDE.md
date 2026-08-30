# CLAUDE.md

Point d'entrée Claude Code pour Candilog. Les règles du projet ne sont pas dupliquées
ici : elles vivent dans `AGENTS.md`, importé ci-dessous.

@AGENTS.md

## Spécifique à Claude Code

- **Avant toute modification significative**, relire `AGENTS.md` puis le document de
  référence de la tâche (`docs/CODE_RULES.md`, `docs/ARCHITECTURE.md`, `docs/DESIGN.md`,
  `docs/DATA.md`, `docs/AI.md`). Ne pas charger toute la documentation par réflexe.
- **Périmètre imbriqué** : en travaillant dans `src/`, `src-tauri/` ou `website/`, lire le
  `AGENTS.md` du dossier concerné — il ne contient que les différences.
- **Commandes longues** : `npm test` (~1 min), `cargo clippy` et `cargo test` (plusieurs
  minutes à froid). Les lancer en arrière-plan plutôt que d'augmenter les délais d'attente.
- **Lancer l'application** : `npm run tauri dev` ouvre la fenêtre native (requiert un
  environnement graphique). `npm run dev` sert le frontend seul sur le port 1420 — l'IPC
  est indisponible, donc les écrans dépendant des données échouent. `.claude/launch.json`
  déclare cette cible sous le nom `candilog-web`.
- **Git** : ne jamais commiter ni pousser sans demande explicite. En cas de commit
  demandé, message en français avec préfixe Conventional Commits
  (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`).
- **Rapport de fin de tâche** : indiquer les commandes de validation réellement exécutées
  et leur résultat, et ce qui n'a pas pu être vérifié. Ne pas annoncer « terminé » sans
  ces preuves.
