#!/usr/bin/env python3
"""Déplace les fonctions d'écran dans les dossiers views des domaines.

Les fichiers sont inclus par l'orchestrateur Iced afin de conserver un seul contexte de rendu
et éviter de rendre publics les helpers internes de l'application.
"""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "src/app/view.rs"
MAPPING = {
    "candidatures": "src/modules/candidatures/views/pipeline.rs",
    "cv_versions": "src/modules/cv/views/versions.rs",
    "entreprises": "src/modules/entreprises/views/directory.rs",
    "contacts": "src/modules/contacts/views/network.rs",
    "calendrier": "src/modules/entretiens/views/calendar.rs",
    "statistiques": "src/modules/metriques/views/dashboard.rs",
    "cv_generator": "src/modules/ia/views/cv_generator.rs",
    "lettre_motivation": "src/modules/ia/views/cover_letter.rs",
    "cv_import": "src/modules/ia/views/cv_import.rs",
    "profil": "src/modules/profil/views/profile.rs",
    "parametres": "src/modules/settings/views/settings.rs",
}


def function_end(text: str, start: int) -> int:
    opening = text.find("{", start)
    depth = 0
    in_string = False
    escaped = False
    for pos in range(opening, len(text)):
        char = text[pos]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return pos + 1
    raise RuntimeError("fonction non fermée")


def main() -> None:
    text = SOURCE.read_text()
    replacements = []
    for name, relative in MAPPING.items():
        match = re.search(rf"(?m)^fn {name}(?:<'[^>]+>)?\s*\(", text)
        if not match:
            raise SystemExit(f"fonction introuvable: {name}")
        end = function_end(text, match.start())
        function = text[match.start():end].strip() + "\n"
        target = ROOT / relative
        if target.exists():
            raise SystemExit(f"refus d'écraser {target}")
        target.write_text("//! Vue native du domaine.\n\n" + function)
        include = f'include!(concat!(env!("CARGO_MANIFEST_DIR"), "/{relative}"));'
        replacements.append((match.start(), end, include))
    for start, end, include in sorted(replacements, reverse=True):
        text = text[:start] + include + text[end:]
    SOURCE.write_text(text)
    print(f"vues_extraites={len(replacements)}")


if __name__ == "__main__":
    main()
