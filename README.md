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

Candilog utilise le renderer logiciel `tiny-skia` d'Iced. Ce choix garde le rendu déterministe,
retire la chaîne `iced_glyphon -> lru 0.12.5` concernée par deux avis RustSec et évite d'imposer
Vulkan/Metal/DirectX pour cette interface 2D.

| Plateforme | Requis |
|---|---|
| Linux | `libxkbcommon`, `libwayland-client` (session Wayland) ou `libX11`, et `libdbus-1` |
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
