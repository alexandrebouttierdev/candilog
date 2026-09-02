#!/usr/bin/env python3
"""Réduit la police Material Symbols Rounded aux seules icônes utilisées par Candilog.

La police complète pèse 5,2 Mio pour ~4 300 icônes ; l'interface en utilise une centaine.
La sous-police produite est versionnée (`src/shared/ui/material-symbols-rounded.woff2`) :
seul l'ajout d'une icône oblige à rejouer ce script.

La liste d'icônes vient de `src/shared/ui/icon-names.ts`, qui est aussi le type `IconName`
du composant `Icon` — `tsc` refuse donc toute icône absente de la sous-police.

    python3 scripts/subset-icons.py

Prérequis : `fontTools` (`pip install "fonttools[woff]"`) et `npm install` (la police
complète est lue depuis `node_modules/material-symbols/`).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

from fontTools.subset import Options, Subsetter
from fontTools.ttLib import TTFont

RACINE = Path(__file__).resolve().parent.parent
SOURCE = RACINE / "node_modules/material-symbols/material-symbols-rounded.woff2"
NOMS = RACINE / "src/shared/ui/icon-names.ts"
SORTIE = RACINE / "src/shared/ui/material-symbols-rounded.woff2"

# Caractères composant un nom d'icône : ce sont eux que le moteur de rendu enchaîne avant
# que la ligature ne remplace la séquence par le glyphe.
ALPHABET = "abcdefghijklmnopqrstuvwxyz_0123456789"


def lire_noms() -> list[str]:
    source = NOMS.read_text(encoding="utf-8")
    noms = re.findall(r'^\s*"([a-z0-9_]+)",', source, re.MULTILINE)
    if not noms:
        sys.exit(f"Aucune icône lue dans {NOMS}")
    return noms


def ligatures(police: TTFont) -> dict[str, str]:
    """Nom d'icône → glyphe produit par la ligature."""
    cmap = police.getBestCmap()
    par_glyphe: dict[str, int] = {}
    for point, glyphe in cmap.items():
        par_glyphe.setdefault(glyphe, point)

    table: dict[str, str] = {}
    for lookup in police["GSUB"].table.LookupList.Lookup:
        for brute in lookup.SubTable:
            sous_table, genre = (
                (brute.ExtSubTable, brute.ExtensionLookupType)
                if lookup.LookupType == 7
                else (brute, lookup.LookupType)
            )
            if genre != 4:
                continue
            for premier, suites in sous_table.ligatures.items():
                for suite in suites:
                    sequence = [premier] + list(suite.Component)
                    try:
                        mot = "".join(chr(par_glyphe[g]) for g in sequence)
                    except KeyError:
                        continue
                    table[mot.lower()] = suite.LigGlyph
    return table


def main() -> None:
    if not SOURCE.exists():
        sys.exit(f"Police source absente : {SOURCE} — lancez `npm install`.")

    noms = lire_noms()
    police = TTFont(SOURCE)
    table = ligatures(police)

    inconnues = [nom for nom in noms if nom not in table]
    if inconnues:
        sys.exit(f"Icônes absentes de Material Symbols Rounded : {', '.join(inconnues)}")

    connus = set(police.getGlyphOrder())
    glyphes = {table[nom] for nom in noms}
    # L'axe FILL passe par une substitution vers un glyphe `.fill` dédié : sans lui,
    # `filled` n'aurait aucun effet sur les icônes concernées.
    glyphes |= {f"{glyphe}.fill" for glyphe in list(glyphes) if f"{glyphe}.fill" in connus}

    options = Options()
    options.flavor = "woff2"
    options.layout_features = ["rlig", "rclt"]
    # Sans cette désactivation, la clôture ajoute toutes les ligatures composables avec les
    # lettres retenues — c'est-à-dire la police entière.
    options.layout_closure = False
    options.name_IDs = ["*"]
    options.notdef_outline = True

    subsetter = Subsetter(options=options)
    subsetter.populate(glyphs=sorted(glyphes), text=ALPHABET)
    subsetter.subset(police)
    SORTIE.parent.mkdir(parents=True, exist_ok=True)
    police.save(SORTIE)

    poids = SORTIE.stat().st_size
    print(f"{len(noms)} icônes → {SORTIE.relative_to(RACINE)} ({poids / 1024:.1f} Kio)")


if __name__ == "__main__":
    main()
