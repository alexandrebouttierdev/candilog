import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const styles = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), "../../../styles.css"),
  "utf8",
);
const sourceRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../../features/documents/view");
const documentSources = [
  "components/DocumentUi.tsx",
  "components/PaperPreview.tsx",
  "pages/DocumentsPages.tsx",
].map((path) => readFileSync(resolve(sourceRoot, path), "utf8"));

function themeNames(kind: "color" | "text"): string[] {
  const seen = new Set<string>();
  const pattern = new RegExp(`--${kind}-([a-z0-9-]+):`, "g");
  for (const match of styles.matchAll(pattern)) {
    const name = match[1];
    if (name && !name.includes("--")) seen.add(name);
  }
  return [...seen];
}

describe("jetons Tailwind", () => {
  it("n'écrase pas une taille de texte par une couleur du même nom", () => {
    const colors = themeNames("color");
    const texts = themeNames("text");
    expect(colors.filter((name) => texts.includes(name))).toEqual([]);
  });

  it("redessine les cases à cocher que le preflight Tailwind rend transparentes", () => {
    expect(styles).toContain("--candilog-border-checkbox:");
    expect(styles).toMatch(/input\[type="checkbox"\][\s\S]*?appearance:\s*none/);
    expect(styles).toMatch(/input\[type="checkbox"\][\s\S]*?background-color:\s*var\(--color-fill\)/);
  });

  it("donne à la feuille sa propre sélection, illisible sinon en thème sombre", () => {
    // La feuille reste blanche en thème sombre : héritée de l'application, la sélection y
    // écrivait en encre claire sur fond clair, et le texte sélectionné disparaissait.
    expect(styles).toContain("--paper-selection:");
    // Chaque surface papier doit être couverte : la lettre a changé de conteneur une fois,
    // et la sélection est redevenue illisible sans que rien ne le signale.
    const regle = styles.match(/([^}]*)::selection\s*{\s*background:\s*var\(--paper-selection\)/);
    expect(regle?.[1]).toContain(".paper-preview");
    expect(regle?.[1]).toContain(".letter-paper");
  });

  it("centralise les couleurs et le rayon de l'aperçu papier dans les jetons", () => {
    expect(styles).toContain("--paper-bg:");
    expect(styles).toContain("--paper-ink:");
    expect(styles).toContain("--paper-muted:");
    expect(styles).toContain("--paper-border:");
    for (const source of documentSources) {
      expect(source).not.toMatch(/#[0-9a-f]{3,8}\b/i);
      expect(source).not.toMatch(/rgb\(/i);
      expect(source).not.toMatch(/rounded-\[/);
    }
  });
});
