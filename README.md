# Candilog Desktop

Réécriture native de Candilog en Rust, Iced, Tokio et SQLite.

```bash
cargo run
```

Validations :

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

L'ancienne application n'est jamais ouverte en écriture. L'avancement détaillé est dans `MIGRATION_PROGRESS.md` et la couverture fonctionnelle dans `MIGRATION_MATRIX.md`.

