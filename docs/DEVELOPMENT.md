# Développement

Installer, lancer, régénérer, valider. Les règles de code sont dans
[`CODE_RULES.md`](CODE_RULES.md) ; les couches dans [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Prérequis

| Outil | Version | Vérifié par |
| --- | --- | --- |
| Node.js | 24 | la CI épingle `node-version: "24"` |
| Yarn | 4.9.1, via Corepack | `packageManager` de `package.json` |
| Rust | 1.91 | `rust-version` de `src-tauri/Cargo.toml` |
| Cargo | fourni par la toolchain Rust | — |
Dépendances système Linux (liste appliquée par le workflow de release sur Ubuntu 22.04 ;
adapter les noms de paquets à la distribution) :

```
libwebkit2gtk-4.1-dev  libappindicator3-dev  librsvg2-dev  patchelf  xdg-utils
```

Sur macOS et Windows, suivre les prérequis Tauri 2 officiels (Xcode Command Line Tools,
Microsoft C++ Build Tools et WebView2).

### Dépendances d'exécution

À distinguer des prérequis ci-dessus, qui ne servent qu'à **construire** : celles-ci sont
nécessaires à l'application **installée**.

| Outil | Requis par | Déclaré dans |
| --- | --- | --- |
| `pdftoppm`, `pdftotext`, `pdfinfo` (Poppler) | import de CV en mode Vision, extraction du texte dans l'ordre de mise en page | `bundle.linux.deb.depends` et `rpm.depends` (`tauri.conf.json`), `depends` du [`PKGBUILD`](../packaging/arch/PKGBUILD) |

Les paquets Linux posent donc cette dépendance eux-mêmes (`poppler-utils` sous Debian,
Ubuntu, Fedora et RHEL ; `poppler` sous Arch). En développement, l'installer à la main :
sans elle, l'import Vision échoue avec un message explicite et l'extraction de texte
retombe sur l'extracteur de flux, à l'ordre de colonnes incorrect.

**macOS et Windows n'ont pas d'équivalent** : aucun gestionnaire de paquets n'est supposé
présent, et les binaires ne sont pas embarqués. L'import Vision y dépend donc d'une
installation Poppler faite par l'utilisateur ; à défaut, seul le mode Texte fonctionne.
C'est une limite de plateforme assumée, signalée à l'utilisateur par le message d'erreur du
mode Vision (`docs/AI.md`).

Le paquet macOS cible macOS 11.0 au minimum. Cette borne, déclarée dans
`tauri.conf.json`, couvre les API requises par le runtime natif et reste cohérente avec
la cible minimale du binaire produit par la toolchain de release.

Outils facultatifs : `cargo-deny` (audit des dépendances Rust, non installé par le dépôt).

## Installation

```bash
yarn install
```

Le dépôt utilise **Yarn** (`yarn.lock` fait foi), dans la version épinglée par le champ
`packageManager` de `package.json` : `corepack enable` une fois suffit à l'obtenir. Les
dépendances sont installées dans `node_modules/` (`nodeLinker: node-modules`,
`.yarnrc.yml`). Le site `website/` reste un projet npm autonome.

## Lancer

```bash
yarn tauri dev     # fenêtre native + backend Rust : le mode de travail normal
yarn dev           # frontend seul sur http://localhost:1420
```

`yarn dev` n'a **pas** d'IPC : tout écran qui charge des données échoue. C'est utile
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
échouer `yarn build`.

Lancer la commande **sans filtre** : `cargo test … <motif>` n'exécute que les tests
d'export retenus par le motif, et laisse les autres fichiers de `generated/` amputés. Un
`git status --short` après coup doit être vide.

## Valider

Le job `quality` du workflow de release rejoue ces commandes et conditionne tous les builds.
Elles restent obligatoires localement avant de terminer une tâche : le workflow protège une
publication, il ne sert pas de boucle de développement.

```bash
yarn lint
yarn test
yarn build          # inclut tsc --noEmit

cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets
```

Avant une publication, ajouter le build de release réel — c'est la seule commande locale qui
exerce le bundler. Le workflow l'exécute ensuite sur chaque plateforme uniquement si son job
`quality` est vert :

```bash
yarn tauri build
```

Il doit sortir en 0 et produire exactement les cibles de `bundle.targets`
(`docs/RELEASES.md`).

Après un changement de dépendance Rust :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check
```

Il n'existe ni `yarn format`, ni `yarn typecheck` à la racine : `cargo fmt` couvre
le formatage Rust, `yarn build` couvre le typage TypeScript.

## Scénario de bout en bout des documents

`src-tauri/tests/e2e_documents.rs` traverse toute la chaîne de génération — `AiService`,
`prepare_workspace`, `ResumePdf`, `CoverLetterPdf` — pour les profils fictifs de
`src-tauri/tests/fixtures/profiles/`, et dépose ses artefacts dans `test-output/`
(profil source, génération, poste de travail, PDF). Il est **ignoré** tant que
`CANDILOG_E2E` est absent : aucune suite standard ne déclenche d'appel payant. Le mode
rejeu, lui, ne demande ni réseau ni fournisseur : le job `quality` du workflow de release
le rejoue à chaque publication (`CANDILOG_E2E=1`, sans `CANDILOG_E2E_LIVE`).

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
yarn playwright install chromium   # une fois
yarn e2e                       # lance le serveur Vite au besoin
yarn e2e:typecheck
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
```

`docs/superpowers/` conserve des plans de travail datés. Ce ne sont pas des documents de
référence, et ils **ne sont pas suivis par Git** : ce sont des notes de conception, parfois
sur des fonctionnalités non annoncées, que la publication du dépôt n'a pas à diffuser. Même
raison pour `AUDIT_APP_PROMPT.md` et pour `.claude/`, dont la configuration d'agent est
propre à chaque poste.

## Hooks Git

`.githooks/` contient un `commit-msg` qui retire les lignes d'attribution ajoutées par les
assistants de code (`Co-authored-by:` nommant un outil, lien de session, « Generated
with »). Les instructions du dépôt l'exigent déjà (`docs/CODE_RULES.md` §18) ; le hook le
garantit quel que soit l'outil. Il ne touche à rien d'autre dans le message, et laisse
passer un co-auteur humain.

À activer une fois par clone — Git n'installe aucun hook automatiquement :

```bash
git config core.hooksPath .githooks
```

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
