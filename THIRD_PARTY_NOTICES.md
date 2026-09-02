# Notices des composants tiers

Candilog est distribué sous la **PolyForm Noncommercial License 1.0.0** (voir [`LICENSE`](./LICENSE)
et [`NOTICE`](./NOTICE)). Les composants ci-dessous conservent leurs propres licences, que la
licence de Candilog ne remplace pas.

Ce fichier accompagne les binaires publiés : il est embarqué dans le paquet aux côtés de
`LICENSE` et `NOTICE` (`src-tauri/tauri.conf.json`, section `bundle.resources`).

---

## Polices embarquées dans le binaire

Ce sont les seuls composants tiers dont les **fichiers** sont redistribués tels quels par
Candilog ; leurs licences exigent que la mention de copyright et le texte les accompagnent.

### IBM Plex Sans, IBM Plex Mono

Copyright © 2017 IBM Corp. with Reserved Font Name "Plex".
Licence **SIL Open Font License 1.1** — texte intégral :
[`src-tauri/assets/fonts/ibm-plex/LICENSE.txt`](./src-tauri/assets/fonts/ibm-plex/LICENSE.txt).

Utilisation : composition des CV et des lettres exportés en PDF (`src-tauri/src/infrastructure/pdf/`)
et de leur aperçu à l'écran, qui doit rester fidèle à la page imprimée.

### Material Symbols Rounded

Copyright © Google LLC. Licence **Apache License 2.0** —
<https://www.apache.org/licenses/LICENSE-2.0>.

Utilisation : icônes de l'interface. Le fichier embarqué
(`src/shared/ui/material-symbols-rounded.woff2`) est une **sous-police** dérivée de la
police publiée, réduite aux icônes réellement employées par `scripts/subset-icons.py`. Seuls
des glyphes ont été retirés ; aucune forme n'a été modifiée.

---

## Crate Rust recopiée dans le dépôt

### pdf-extract 0.12.0

Copyright © Jeff Muizelaar. Licence **MIT** —
[`vendor/pdf-extract-0.12.0/LICENSE`](./vendor/pdf-extract-0.12.0/LICENSE).
Provenance et nature de la modification locale :
[`vendor/pdf-extract-0.12.0/VENDOR.md`](./vendor/pdf-extract-0.12.0/VENDOR.md).

---

## Bibliothèques liées

Candilog est lié à des bibliothèques Rust et JavaScript qui restent sous leurs licences
respectives. Elles ne sont pas recopiées dans ce dépôt : leur code et leurs fichiers de
licence vivent dans les registres d'origine, et les manifestes du dépôt en donnent la liste
exacte et vérifiable.

| Écosystème | Inventaire | Licences autorisées |
| --- | --- | --- |
| Rust | `src-tauri/Cargo.lock` | `deny.toml`, section `[licenses]` |
| JavaScript | `package-lock.json` | — |

La conformité des licences Rust est contrôlée à chaque changement de dépendance :

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check licenses sources
```

Pour obtenir la liste nominative des crates et de leurs licences :

```bash
cargo tree --manifest-path src-tauri/Cargo.toml --format "{p} {l}" --prefix none | sort -u
```

Principales briques : **Tauri 2** (MIT / Apache-2.0), **WebKitGTK** et **GTK 3** (LGPL,
bibliothèques système non redistribuées), **rusqlite** et **SQLite** (MIT / domaine public),
**printpdf** et **lopdf** (MIT), **React** (MIT), **TanStack Query** (MIT), **Zod** (MIT),
**Tailwind CSS** (MIT).
