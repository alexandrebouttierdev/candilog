## Ce que fait cette PR

<!-- En une ou deux phrases, et le pourquoi. -->

## Validations exécutées

Aucune CI ne rejoue ces contrôles : ce qui n'est pas lancé ici ne l'est nulle part
(`docs/CODE_RULES.md` §20). Cocher ce qui a **réellement** été exécuté, et dire ce qui ne
l'a pas été.

- [ ] `npm run lint`
- [ ] `npm test`
- [ ] `npm run build`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets`
- [ ] `cargo deny --manifest-path src-tauri/Cargo.toml check` *(si les dépendances changent)*
- [ ] `git status --short` propre après `cargo test` *(types ts-rs à jour)*

Non exécuté, et pourquoi :

## Tests

<!-- Quel comportement est couvert ? Pour une correction : le test échouait-il avant ? -->

## Documentation

- [ ] La documentation concernée est à jour dans la même PR (`docs/CODE_RULES.md` §22)
- [ ] Aucun changement documentaire nécessaire

## Licence

- [ ] J'ai lu [`CONTRIBUTING.md`](../CONTRIBUTING.md) et [`CLA.md`](../CLA.md)
