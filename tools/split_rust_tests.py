#!/usr/bin/env python3
"""Sépare les tests Rust inline en un fichier par cas de test.

Le script conserve les imports et helpers communs dans `tests/<source>/mod.rs`, puis remplace
le module inline par un module `#[path]`. Il est volontairement borné aux modules
`#[cfg(test)] mod tests` du projet et refuse d'écraser un dossier déjà produit.
"""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
MODULE_RE = re.compile(r"#\[cfg\(test\)\][^\n]*\n(?:#\[[^\n]+\][^\n]*\n)*\s*mod\s+tests\s*\{")
TEST_RE = re.compile(
    r"(?m)^(?P<indent>\s*)(?P<attrs>(?:#\[[^\n]+\]\s*\n)+)"
    r"(?P<signature>\s*(?:async\s+)?fn\s+(?P<name>[a-zA-Z0-9_]+)\s*\()"
)


def mask_non_code(text: str) -> str:
    """Masque chaînes et commentaires en conservant positions et retours à la ligne."""
    chars = list(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                chars[pos] = " "
            i = end
            continue
        if text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) - 2 if end < 0 else end
            for pos in range(i, end + 2):
                if chars[pos] != "\n":
                    chars[pos] = " "
            i = end + 2
            continue
        raw = re.match(r"(?:b)?r(?P<hashes>#{0,16})\"", text[i:])
        if raw:
            marker = '"' + raw.group("hashes")
            end = text.find(marker, i + raw.end())
            end = len(text) - len(marker) if end < 0 else end
            stop = end + len(marker)
            for pos in range(i, stop):
                if chars[pos] != "\n":
                    chars[pos] = " "
            i = stop
            continue
        if text[i] == '"':
            end = i + 1
            escaped = False
            while end < len(text):
                if escaped:
                    escaped = False
                elif text[end] == "\\":
                    escaped = True
                elif text[end] == '"':
                    end += 1
                    break
                end += 1
            for pos in range(i, end):
                if chars[pos] != "\n":
                    chars[pos] = " "
            i = end
            continue
        i += 1
    return "".join(chars)


def matching_brace(text: str, opening: int) -> int:
    code = mask_non_code(text)
    depth = 0
    i = opening
    while i < len(code):
        char = code[i]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("accolade fermante introuvable")


def top_level_tests(body: str) -> list[tuple[int, int, str, str]]:
    results: list[tuple[int, int, str, str]] = []
    code = mask_non_code(body)
    for match in TEST_RE.finditer(body):
        prefix = code[: match.start()]
        depth = prefix.count("{") - prefix.count("}")
        if depth != 0:
            continue
        attrs = match.group("attrs")
        if "#[test]" not in attrs and "#[tokio::test" not in attrs:
            continue
        opening = body.find("{", match.end())
        if opening < 0:
            continue
        end = matching_brace(body, opening) + 1
        results.append((match.start(), end, match.group("name"), body[match.start() : end]))
    return results


def split_existing_suite(mod_file: Path) -> int:
    body = mod_file.read_text()
    tests = top_level_tests(body)
    if not tests:
        return 0
    support = body
    declarations = []
    for start, end, _, _ in reversed(tests):
        support = support[:start] + support[end:]
    for _, _, name, function in tests:
        target = mod_file.parent / f"{name}.rs"
        if target.exists():
            raise SystemExit(f"Refus d'écraser {target}")
        target.write_text("//! Cas de test isolé.\n\nuse super::*;\n\n" + function.strip() + "\n")
        declarations.append(f"mod {name};")
    mod_file.write_text(support.rstrip() + "\n\n" + "\n".join(declarations) + "\n")
    return len(tests)


def main() -> None:
    total = 0
    for mod_file in sorted(SRC.rglob("tests/*/mod.rs")):
        total += split_existing_suite(mod_file)
    for source in sorted(SRC.rglob("*.rs")):
        if "/tests/" in source.as_posix():
            continue
        text = source.read_text()
        module = MODULE_RE.search(text)
        if not module:
            continue
        opening = text.find("{", module.start())
        closing = matching_brace(text, opening)
        body = text[opening + 1 : closing]
        tests = top_level_tests(body)
        if not tests:
            continue

        relative = source.relative_to(SRC)
        domain_dir = relative.parent
        suite_name = source.stem
        output_dir = SRC / domain_dir / "tests" / suite_name
        if output_dir.exists():
            raise SystemExit(f"Refus d'écraser {output_dir}")
        output_dir.mkdir(parents=True)

        support = body
        for start, end, _, _ in reversed(tests):
            support = support[:start] + support[end:]
        declarations = []
        for _, _, name, function in tests:
            (output_dir / f"{name}.rs").write_text(
                "//! Cas de test isolé.\n\nuse super::*;\n\n" + function.strip() + "\n"
            )
            declarations.append(f"mod {name};")
        (output_dir / "mod.rs").write_text(
            "//! Helpers communs et déclaration des cas de test.\n"
            + support.strip()
            + "\n\n"
            + "\n".join(declarations)
            + "\n"
        )

        path = f'tests/{suite_name}/mod.rs'
        replacement = f'#[cfg(test)]\n#[path = "{path}"]\nmod tests;'
        source.write_text(text[: module.start()] + replacement + text[closing + 1 :])
        total += len(tests)
    print(f"tests_extraits={total}")


if __name__ == "__main__":
    main()
