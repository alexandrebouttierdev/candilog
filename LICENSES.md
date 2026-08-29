# Licences de Candilog

## Licence du code Candilog

Le code Candilog détenu ou licencié par le projet est mis à disposition sous la **PolyForm Noncommercial License 1.0.0** pour les usages qu'elle autorise. Le [texte officiel](./LICENSE) est la référence.

Copyright © 2026 Alexandre Bouttier

## Licence commerciale

Les droits d'utilisation commerciale sont disponibles séparément, uniquement sur autorisation explicite du titulaire des droits. Le document [`COMMERCIAL_LICENSE.md`](./COMMERCIAL_LICENSE.md) explique la démarche sans constituer lui-même un contrat commercial.

## Licences des dépendances tierces

Les bibliothèques, assets, polices, icônes, logos et autres composants tiers conservent leurs propres licences et mentions de copyright. La licence de Candilog ne remplace jamais leurs conditions et n'accorde aucun droit supplémentaire sur ces éléments.

Les métadonnées des dépendances JavaScript sont consignées dans `package-lock.json`. Pour Rust, le dépôt utilise déjà `cargo deny` et sa configuration [`deny.toml`](./deny.toml) afin de contrôler les licences et les sources déclarées :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check licenses sources
```

Le dépôt ne contient pas actuellement de mécanisme générant une notice consolidée de toutes les licences tierces. Les fichiers de licence fournis directement avec une dépendance restent applicables et ne doivent pas être modifiés.
