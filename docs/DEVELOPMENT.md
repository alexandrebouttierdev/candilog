# Développement

Installer, lancer, régénérer, valider. Les règles de code sont dans
[`CODE_RULES.md`](CODE_RULES.md) ; les couches dans [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Prérequis

| Outil | Version | Vérifié par |
| --- | --- | --- |
| Node.js | LTS | la CI utilise `node-version: lts/*` |
| Rust | 1.91 | `rust-version` de `src-tauri/Cargo.toml` |
| Cargo | fourni par la toolchain Rust | — |

Dépendances système Linux (liste appliquée par le workflow de release sur Ubuntu 22.04 ;
adapter les noms de paquets à la distribution) :

```
libwebkit2gtk-4.1-dev  libappindicator3-dev  librsvg2-dev  patchelf  xdg-utils
```

Sur macOS et Windows, suivre les prérequis Tauri 2 officiels (Xcode Command Line Tools,
Microsoft C++ Build Tools et WebView2).

Outils facultatifs : `cargo-deny` (audit des dépendances Rust, non installé par le dépôt).

## Installation

```bash
npm install
```

`.npmrc` fixe `cache=.npm-cache` : le cache npm est local au dépôt et ignoré par Git.

## Lancer

```bash
npm run tauri dev     # fenêtre native + backend Rust : le mode de travail normal
npm run dev           # frontend seul sur http://localhost:1420
```

`npm run dev` n'a **pas** d'IPC : tout écran qui charge des données échoue. C'est utile
pour le style et la galerie de composants, pas pour tester un comportement métier.

Le port 1420 est en `strictPort` : s'il est occupé, Vite échoue au lieu de basculer
silencieusement sur un autre port que la fenêtre native ne suivrait pas.

## Données de développement

Un binaire debug écrit obligatoirement dans `src-tauri/.candilog-dev/` (ancré sur
`CARGO_MANIFEST_DIR`), jamais dans la base utilisateur. Détails et invariants du schéma :
[`DATA.md`](DATA.md).

| Variable | Effet |
| --- | --- |
| `CANDILOG_DATA_DIR` | Remplace le dossier de données (base, exports, journal) |
| `RUST_LOG` | Niveau de journalisation ; défaut `candilog=info` |

Le journal est écrit à la fois sur la sortie standard et dans `candilog.log` du dossier de
données, avec rotation sur cinq fichiers.

La clé API du fournisseur IA vit dans le coffre du système (`keyring`), pas dans SQLite ni
dans un fichier du dépôt.

## Régénérer les types IPC

`src/shared/types/generated/` est produit par `ts-rs` depuis les structs Rust et ne
s'édite jamais à la main :

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

`.cargo/config.toml` pointe `TS_RS_EXPORT_DIR` vers ce dossier, à la racine du projet et
non sous `src-tauri/`, pour que la génération fonctionne aussi bien depuis la racine que
depuis `src-tauri/` (ce que fait la CLI Tauri). Un DTO Rust modifié sans régénération fait
échouer `npm run build`.

## Valider

Aucune CI ne vérifie la qualité : le seul workflow du dépôt publie les releases. Ces
commandes sont donc à lancer localement.

```bash
npm run lint
npm test
npm run build          # inclut tsc --noEmit

cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
```

Après un changement de dépendance Rust :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check
```

Il n'existe ni `npm run format`, ni `npm run typecheck` à la racine : `cargo fmt` couvre
le formatage Rust, `npm run build` couvre le typage TypeScript.

## Structure du dépôt

```text
src/            frontend React (feature-first)
src-tauri/      application native Rust + Tauri
website/        site candilog.fr (Next.js, projet autonome)
docs/           documentation de référence
vendor/         crate `pdf-extract` patchée (voir [patch.crates-io] de Cargo.toml)
```

`docs/superpowers/` conserve des plans de travail datés ; ce ne sont pas des documents de
référence.

## Site candilog.fr

Projet autonome, avec ses propres dépendances et commandes :

```bash
cd website
npm install
npm run dev            # http://localhost:3000
npm run lint
npm run typecheck
npm run build          # export statique dans out/
```

Voir [`../website/README.md`](../website/README.md).
