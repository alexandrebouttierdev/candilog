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

Lancer la commande **sans filtre** : `cargo test … <motif>` n'exécute que les tests
d'export retenus par le motif, et laisse les autres fichiers de `generated/` amputés. Un
`git status --short` après coup doit être vide.

## Ajouter une icône

Les icônes viennent d'une **sous-police** Material Symbols Rounded versionnée
(`src/shared/ui/material-symbols-rounded.woff2`, ~130 Kio) : la police complète pèse
5,2 Mio pour ~4 300 icônes, dont l'interface en emploie une centaine. Une icône absente de
la sous-police s'afficherait en toutes lettres à l'écran.

1. Inscrire le nom dans `src/shared/ui/icon-names.ts`, en ordre alphabétique. C'est aussi
   le type `IconName` du composant `Icon` : `tsc` refuse toute icône hors de cette liste,
   le défaut ne peut donc pas atteindre l'écran.
2. Régénérer la sous-police :

   ```bash
   python3 scripts/subset-icons.py
   ```

   Prérequis : `pip install "fonttools[woff]"` et `npm install` — la police complète est
   lue depuis `node_modules/material-symbols/`, déclarée en dépendance de développement
   pour cette seule raison. Le script échoue si un nom n'existe pas dans Material Symbols.
3. Committer le `.woff2` régénéré avec la liste.

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

Avant une publication, ajouter le build de release réel — c'est la seule commande qui
exerce le bundler, et le workflow ne fait rien d'autre :

```bash
npm run tauri build
```

Il doit sortir en 0 et produire exactement les cibles de `bundle.targets`
(`docs/RELEASES.md`).

Après un changement de dépendance Rust :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check
```

Il n'existe ni `npm run format`, ni `npm run typecheck` à la racine : `cargo fmt` couvre
le formatage Rust, `npm run build` couvre le typage TypeScript.

## Scénario de bout en bout des documents

`src-tauri/tests/e2e_documents.rs` traverse toute la chaîne de génération — `AiService`,
`prepare_workspace`, `ResumePdf`, `CoverLetterPdf` — pour les profils fictifs de
`src-tauri/tests/fixtures/profiles/`, et dépose ses artefacts dans `test-output/`
(profil source, génération, poste de travail, PDF). Il est **ignoré** tant que
`CANDILOG_E2E` est absent : aucune suite standard ne déclenche d'appel payant.

```bash
# Rejeu : la génération enregistrée est relue, seuls la composition et l'export sont rejoués.
CANDILOG_E2E=1 cargo test --manifest-path src-tauri/Cargo.toml --locked --test e2e_documents

# Appel réel au fournisseur IA configuré, puis enregistrement pour les rejeux suivants.
CANDILOG_E2E=1 CANDILOG_E2E_LIVE=1 CANDILOG_E2E_OFFER=/chemin/offre.txt   cargo test --manifest-path src-tauri/Cargo.toml --locked --test e2e_documents
```

| Variable | Rôle | Défaut |
| --- | --- | --- |
| `CANDILOG_E2E` | active le scénario | absent → ignoré |
| `CANDILOG_E2E_LIVE` | appelle réellement le fournisseur IA | absent → rejeu |
| `CANDILOG_E2E_OFFER` | fichier de l'offre | requis en live |
| `CANDILOG_E2E_SETTINGS_DB` | base dont les réglages IA sont copiés | base de développement |
| `CANDILOG_E2E_OUT` | dossier des artefacts | `test-output/` |
| `CANDILOG_E2E_ONLY` | profils à traiter (`01,07`) | tous |

Un profil dont le contenu dépasse réellement la page A4 a pour résultat correct un **refus**
d'export : il porte alors un `profile-NN.expected.json` à côté de sa fixture. Le scénario
échoue aussi bien si l'export refuse à tort que s'il accepte ce qu'il aurait dû refuser.

## Contrôle visuel des feuilles A4

Playwright monte les **vrais** composants `ResumePaper` et `LetterPaper` (banc de rendu
`e2e/harness/`, servi par le serveur Vite de l'application) sur les artefacts du scénario
ci-dessus, puis mesure la géométrie réelle : débordements, sorties de colonne, collisions,
polices, valeurs parasites, erreurs de console. Les PDF exportés sont ouverts avec Poppler
(`pdfinfo`, `pdftotext`, `pdftoppm`) : pages, marges, chevauchements, glyphes perdus, rendu
en image. Le banc ne fait pas partie du bundle — `vite build` n'a qu'une entrée,
`index.html`.

```bash
npx playwright install chromium   # une fois
npm run e2e                       # lance le serveur Vite au besoin
npm run e2e:typecheck
```

Prérequis : `poppler-utils` (`pdfinfo`, `pdftotext`, `pdftoppm`). Les artefacts et les
rapports sont écrits dans `test-output/`, ignoré par Git.

## Structure du dépôt

```text
src/            frontend React (feature-first)
src-tauri/      application native Rust + Tauri
website/        site candilog.fr (Next.js, projet autonome)
docs/           documentation de référence
vendor/         crate `pdf-extract` patchée (voir [patch.crates-io] de Cargo.toml)
scripts/        outillage ponctuel (sous-police des icônes)
```

`docs/superpowers/` conserve des plans de travail datés. Ce ne sont pas des documents de
référence, et ils **ne sont pas suivis par Git** : ce sont des notes de conception, parfois
sur des fonctionnalités non annoncées, que la publication du dépôt n'a pas à diffuser. Même
raison pour `AUDIT_APP_PROMPT.md`.

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
