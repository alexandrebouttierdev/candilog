## pdf-extract
[![Build Status](https://github.com/jrmuizel/pdf-extract/actions/workflows/rust.yml/badge.svg)](https://github.com/jrmuizel/pdf-extract/actions)
[![crates.io](https://img.shields.io/crates/v/pdf-extract.svg)](https://crates.io/crates/pdf-extract)
[![Documentation](https://docs.rs/pdf-extract/badge.svg)](https://docs.rs/pdf-extract)

A rust library to extract content from PDF files.

```rust
let bytes = std::fs::read("tests/docs/simple.pdf").unwrap();
let out = pdf_extract::extract_text_from_mem(&bytes).unwrap();
assert!(out.contains("This is a small demonstration"));
```

## See also

- https://github.com/elacin/PDFExtract/
- https://github.com/euske/pdfminer / https://github.com/pdfminer/pdfminer.six
- https://gitlab.com/crossref/pdfextract
- https://github.com/VikParuchuri/marker
- https://github.com/kermitt2/pdfalto used by [grobid](https://github.com/kermitt2/grobid/)
- https://github.com/opendatalab/MinerU (uses PyMuPDF and pdfminer.six)

### Not PDF specific
- https://github.com/Layout-Parser/layout-parser

## Correctif vendored (candilog)

Cette copie de `pdf-extract` 0.12.0 porte un correctif local, référencée par
`[patch.crates-io]` dans le `Cargo.toml` de candilog.

**Bogue corrigé** : dans `PdfCIDFont::new` (src/lib.rs), le parsing du tableau `/W` au
format range (`c_first c_last w`) lisait `c_last` et `c_width` depuis `w[i]` au lieu de
`w[i + 1]` et `w[i + 2]`. Les largeurs au format range (produit par LibreOffice) n'étaient
jamais insérées, chaque glyphe retombait sur la largeur par défaut `/DW`, et l'heuristique
d'espace de `PlainTextOutput` (écart > 10 % de la taille de police) fabriquait un espace
parasite : « D éveloppeur » au lieu de « Développeur ».

Le correctif (3 lignes, src/lib.rs:1026-1030) a été validé sur un PDF reproduisant le cas,
et soumis en amont. Cette copie doit être resynchronisée avec une publication amont qui
l'intégrerait.
