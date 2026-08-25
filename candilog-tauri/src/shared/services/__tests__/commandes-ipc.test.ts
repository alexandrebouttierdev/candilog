import { describe, expect, it } from "vitest";
import { globSync, readFileSync } from "node:fs";

/**
 * Contrat entre les services frontend et les commandes Tauri.
 *
 * C'est le seul endroit où une faute de frappe ne se voit ni à la compilation Rust, ni à
 * celle de TypeScript : `ipc("entreprise_lister")` au lieu de `entreprises_lister` compile
 * des deux côtés et n'échoue qu'à l'exécution, dans la fenêtre native, sur un écran vide.
 * Ce test compare donc les deux inventaires.
 */

// `process.cwd()` et non `import.meta.url` : Vitest sert les modules de test par HTTP, et
// leur URL n'est pas un chemin de fichier. Vite lance toujours les tests depuis la racine
// du projet.
const racine = process.cwd();

/** Noms de commandes déclarés côté Rust par `#[tauri::command]`. */
function commandesRust(): Set<string> {
  const fichiers = globSync("src-tauri/src/features/*/presentation/commands.rs", {
    cwd: racine,
  });
  const noms = new Set<string>();
  for (const fichier of fichiers) {
    const source = readFileSync(`${racine}/${fichier}`, "utf8");
    for (const correspondance of source.matchAll(
      /#\[tauri::command\]\s*(?:pub\s+)?async\s+fn\s+(\w+)/g,
    )) {
      noms.add(correspondance[1]!);
    }
  }
  return noms;
}

/** Noms de commandes réellement appelés par les services frontend. */
function commandesAppelees(): Set<string> {
  const fichiers = globSync("src/features/*/services/*.service.ts", { cwd: racine });
  const noms = new Set<string>();
  for (const fichier of fichiers) {
    const source = readFileSync(`${racine}/${fichier}`, "utf8");
    for (const correspondance of source.matchAll(/ipc<[^>]*>\(\s*"([^"]+)"/g)) {
      noms.add(correspondance[1]!);
    }
  }
  return noms;
}

/** Commandes enregistrées dans l'`invoke_handler` du bootstrap. */
function commandesEnregistrees(): Set<string> {
  const source = readFileSync(`${racine}/src-tauri/src/app/bootstrap.rs`, "utf8");
  const bloc = source.slice(
    source.indexOf("generate_handler!["),
    source.indexOf("])", source.indexOf("generate_handler![")),
  );
  return new Set([...bloc.matchAll(/::(\w+),/g)].map((m) => m[1]!));
}

describe("contrat IPC", () => {
  it("trouve bien les commandes des deux côtés", () => {
    // Garde-fou du test lui-même : si les expressions régulières cessaient de correspondre,
    // les comparaisons suivantes passeraient sur deux ensembles vides.
    expect(commandesRust().size).toBeGreaterThan(0);
    expect(commandesAppelees().size).toBeGreaterThan(0);
  });

  it("n'appelle que des commandes qui existent côté Rust", () => {
    const rust = commandesRust();
    const inconnues = [...commandesAppelees()].filter((nom) => !rust.has(nom));
    expect(inconnues).toEqual([]);
  });

  it("enregistre dans l'invoke_handler toutes les commandes appelées", () => {
    // Une commande déclarée mais absente du handler est invisible à l'exécution : Tauri
    // rejette « command not found », et le frontend n'affiche qu'un bandeau d'erreur.
    const enregistrees = commandesEnregistrees();
    const manquantes = [...commandesAppelees()].filter((nom) => !enregistrees.has(nom));
    expect(manquantes).toEqual([]);
  });

  it("n'enregistre pas de commande sans déclaration Rust correspondante", () => {
    const rust = commandesRust();
    const fantomes = [...commandesEnregistrees()].filter((nom) => !rust.has(nom));
    expect(fantomes).toEqual([]);
  });
});
