# Provenance et modification locale

Copie du crate `pdf-extract` **0.12.0** publié sur crates.io par Jeff Muizelaar
(<https://github.com/jrmuizel/pdf-extract>), sous licence **MIT** — voir [`LICENSE`](./LICENSE),
reproduit ici conformément à l'obligation d'attribution de cette licence. Le texte de
référence reste celui du dépôt d'origine.

Le crate est substitué au paquet publié par `[patch.crates-io]`
(`src-tauri/Cargo.toml`), et non ajouté à côté : Candilog compile donc bien cette copie.

## Seule modification apportée

| Fichier | Amont | Ici |
| --- | --- | --- |
| `Cargo.toml`, dépendance `lopdf` | `0.42` (cf. `Cargo.toml.orig` ligne 20) | `0.44` |

Aucun fichier de `src/` n'est modifié.

Cette montée de version aligne `pdf-extract` sur le `lopdf` déjà utilisé par `printpdf`
côté export, et retire de l'arbre `ttf-parser`, non maintenu et signalé par `cargo-deny`
(`docs/RELEASES.md`).

## Mettre à jour cette copie

1. Récupérer la version voulue depuis crates.io (`cargo vendor` ou téléchargement direct).
2. Réappliquer la ligne `lopdf` du tableau ci-dessus tant que l'amont n'a pas rattrapé.
3. Conserver `LICENSE` et ce fichier, puis mettre à jour le tableau.
4. Rejouer `cargo deny --manifest-path src-tauri/Cargo.toml check`.
