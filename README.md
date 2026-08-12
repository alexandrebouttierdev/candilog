# Candilog Desktop

Réécriture native de Candilog en Rust, Iced, Tokio et SQLite.

```bash
cargo run
```

## Validations

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo deny check          # licences et avis RustSec (voir deny.toml)
```

## Dépendances d'exécution

Candilog utilise le renderer graphique par défaut d'Iced. Sous Linux, il sélectionne le backend
compatible avec la session Wayland ou X11 et le pilote graphique disponibles.

| Plateforme | Requis |
|---|---|
| Linux | `libxkbcommon`, `libwayland-client` (session Wayland) ou `libX11`, `libdbus-1` et un pilote Vulkan fonctionnel |
| Windows | Bibliothèques système standard |
| macOS | Bibliothèques système standard |

Les paquets `.deb` et `.rpm` déclarent ces dépendances automatiquement.

## Configuration

| Variable | Effet |
|---|---|
| `CANDILOG_DATA_DIR` | Déplace le dossier de données (base, exports, journaux). Utile pour travailler sans toucher aux données réelles. |
| `RUST_LOG` | Niveau de journalisation, `candilog=info` par défaut. Le journal est écrit sur la sortie standard **et** dans `candilog.log` du dossier de données, avec rotation à chaque démarrage. |

La caractéristique Cargo `capture` (`cargo run --features capture`) active le harnais de
capture visuelle destiné à la revue de design. Elle est absente du binaire distribué.

## Documentation

L'architecture, le modèle de données et le processus de publication sont décrits dans
[`docs/`](docs/).
