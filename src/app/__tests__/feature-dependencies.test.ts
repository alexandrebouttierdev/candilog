import { describe, expect, it } from "vitest";
import { globSync, readFileSync } from "node:fs";

/**
 * Graphe des dépendances entre features (`docs/ARCHITECTURE.md`).
 *
 * Une feature peut en utiliser une autre, jamais les deux l'une de l'autre : un cycle rend
 * l'ordre d'évaluation des modules fragile, empêche de découper le bundle et interdit de
 * comprendre une feature sans l'autre. Quand deux features doivent se rejoindre, la couche
 * `app` les relie (`ApplicationsRoute`, `AiRoute`), ou la dépendance passe par composition
 * (`FollowUpFormModal` reçoit son `applicationPicker`) ou par une marque de requête
 * (`PROFILE_DERIVED_META`).
 */

// `process.cwd()` et non `import.meta.url` : voir `commandes-ipc.test.ts`.
const root = process.cwd();

function featureGraph(): Map<string, Set<string>> {
  const graph = new Map<string, Set<string>>();
  const files = globSync("src/features/*/**/*.{ts,tsx}", { cwd: root }).filter(
    (file) => !file.includes("__tests__"),
  );
  for (const file of files) {
    const from = file.split("/")[2]!;
    const targets = graph.get(from) ?? new Set<string>();
    graph.set(from, targets);
    const source = readFileSync(`${root}/${file}`, "utf8");
    for (const match of source.matchAll(/from\s+"@\/features\/([a-z-]+)/g)) {
      const to = match[1]!;
      if (to !== from) targets.add(to);
    }
  }
  return graph;
}

/** Premier cycle trouvé, sous la forme `a → b → … → a`, ou `null`. */
function findCycle(graph: Map<string, Set<string>>): string[] | null {
  const state = new Map<string, "open" | "done">();
  const path: string[] = [];
  const visit = (node: string): string[] | null => {
    state.set(node, "open");
    path.push(node);
    for (const next of graph.get(node) ?? []) {
      if (state.get(next) === "open") return [...path.slice(path.indexOf(next)), next];
      if (!state.has(next)) {
        const cycle = visit(next);
        if (cycle) return cycle;
      }
    }
    path.pop();
    state.set(node, "done");
    return null;
  };
  for (const node of graph.keys()) {
    if (!state.has(node)) {
      const cycle = visit(node);
      if (cycle) return cycle;
    }
  }
  return null;
}

describe("dépendances entre features", () => {
  it("le graphe est lu", () => {
    const graph = featureGraph();
    expect(graph.size).toBeGreaterThan(10);
    expect(graph.get("documents")?.has("profile")).toBe(true);
  });

  it("ne contient aucun cycle", () => {
    expect(findCycle(featureGraph())?.join(" → ") ?? null).toBeNull();
  });

  it("détecte un cycle quand il y en a un", () => {
    const graph = new Map([
      ["a", new Set(["b"])],
      ["b", new Set(["c"])],
      ["c", new Set(["a"])],
    ]);
    expect(findCycle(graph)).toEqual(["a", "b", "c", "a"]);
  });
});
