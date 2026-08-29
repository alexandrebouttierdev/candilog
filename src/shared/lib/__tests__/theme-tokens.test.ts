import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const styles = readFileSync(
  resolve(dirname(fileURLToPath(import.meta.url)), "../../../styles.css"),
  "utf8",
);

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
});
