# Natif (Rust / Tauri) — règles spécifiques

Complète `AGENTS.md` à la racine (règles générales) et `docs/CODE_RULES.md` (contrat de
qualité). Ici uniquement ce qui est propre à `src-tauri/`.

Crate `candilog`, bibliothèque `candilog_lib`, édition 2021, `rust-version = 1.91`.

## Couches d'une feature

```text
features/<domaine>/
├── domain/          modèles, enums, traits de repository — ni Tauri, ni rusqlite
├── application/     service métier : validation, orchestration, transactions
├── infrastructure/  implémentation SQLite / HTTP du trait de repository
└── presentation/    commands.rs — commandes Tauri, fines
```

`core/` porte le socle partagé : `config` (chemins), `database` (pool, helpers), `errors`,
`logging`, `pagination`, `backup`, `updater`, `secrets`, `files`, `utils/validation`.
`infrastructure/pdf/` porte l'export PDF (CV et lettres).

## Commandes Tauri

- Nom `snake_case` préfixé par le domaine (`applications_list_page`), enregistré
  explicitement dans `app/bootstrap.rs`.
- Une commande prend `State<'_, AppState>` (et `AppHandle` si besoin) et délègue :
  `blocking::execute` pour le synchrone, `async` pour l'IA et le HTTP. Ni SQL, ni prompt,
  ni traitement lourd dans `commands.rs`.
- Toute entrée venant de React est revalidée ici ou dans le service — identifiants, URL,
  chemins, enums, bornes. Réutiliser `core::utils::validation`
  (`validate_optional_http_url`, `validate_user_file_path`) avant d'en écrire une autre.

## Contrat IPC (`ts-rs`)

Les structs `domain` annotées `Serialize` / `Deserialize` / `TS` **sont** le contrat IPC :
pas de couche DTO parallèle. `.cargo/config.toml` fixe
`TS_RS_EXPORT_DIR = src/shared/types/generated`, donc :

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

régénère les types TypeScript. Un DTO modifié sans régénération casse `npm run build`.

## SQL

`rusqlite` + `r2d2`. Requêtes paramétrées uniquement, colonnes explicites (pas de
`SELECT *`), tri dynamique restreint à un enum fermé. Filtrage, tri, recherche,
pagination et agrégation se font dans SQLite, via `core/pagination` pour les bornes.
Transaction dès qu'une suite d'écritures doit être atomique. Helpers dans
`core/database/helpers.rs`. Le schéma et ses invariants sont décrits dans `docs/DATA.md`.

## Erreurs et journal

`AppError` / `AppResult` sont le canal unique ; la variante exposée à l'IPC est
`AppErrorDto { code, message }`. Le `message` est destiné à l'utilisateur, en français ;
le détail technique (chemins, clés, cause) part dans `tracing`, jamais à l'écran. Garder
les variantes typées (`Validation`, `NotFound`, `Database`, `Provider`, `Cancelled`) le
plus longtemps possible.

## Sécurité

- `unwrap` / `expect` / `panic!` interdits hors tests (clippy `deny`). Pas d'`unsafe`.
- Les secrets vont dans le coffre système (`core::secrets`, `keyring`), jamais dans SQLite
  ni dans les journaux.
- Les capabilities (`capabilities/default.json`) n'exposent pas de filesystem générique :
  toute I/O fichier passe par une commande Rust avec chemin validé.
- Pas de commande système construite à partir d'une entrée utilisateur.

## IA

Toute l'IA vit dans `features/ai/` : prompts centralisés en Rust, aucune instruction de
modèle côté React. Une offre ou un PDF est de la **donnée**, jamais des instructions.
Sortie structurée : `parse → validate → grounding`. Le score ATS affiché est le calcul
déterministe Rust. Détails dans `docs/AI.md`.

## Tests

Modules `#[cfg(test)]` colocalisés, un fichier par comportement sous `tests/<module>/`,
nommés en français (`test_create_ouvre_l_historique_de_statut.rs`). SQLite **en mémoire**
uniquement : un test n'ouvre jamais la base utilisateur.

## Validation

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
```

Changement de dépendance : `cargo deny --manifest-path src-tauri/Cargo.toml check`
(politique et exceptions dans `deny.toml`, justifiées dans `docs/RELEASES.md`).
