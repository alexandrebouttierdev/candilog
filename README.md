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

`ldd` ne les montre pas : `winit` et `wgpu` les chargent à la demande via `libloading`, ce qui
les rend invisibles aux outils d'inspection habituels. En leur absence, l'échec survient à
l'initialisation de la fenêtre.

| Plateforme | Requis |
|---|---|
| Linux | `libxkbcommon`, `libwayland-client` (session Wayland) ou `libX11` (session X11), `libdbus-1`, et un pilote graphique **Vulkan** (`mesa-vulkan-drivers`, `vulkan-loader`) |
| Windows | Pilote graphique à jour (DirectX 12 ou Vulkan) |
| macOS | Aucune : Metal est fourni par le système |

Sans pilote Vulkan utilisable, forcer un autre backend `wgpu` :

```bash
WGPU_BACKEND=gl cargo run     # repli OpenGL
```

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
