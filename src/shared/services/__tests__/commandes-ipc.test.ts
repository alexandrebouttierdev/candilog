import { describe, expect, it } from "vitest";
import { globSync, readFileSync } from "node:fs";

/**
 * Contrat entre les services frontend et les commandes Tauri.
 *
 * C'est le seul endroit où une faute de frappe ne se voit ni à la compilation Rust, ni à
 * celle de TypeScript : `ipc("company_list")` au lieu de `companies_list` compile
 * des deux côtés et n'échoue qu'à l'exécution, dans la fenêtre native, sur un écran vide.
 * Ce test compare donc les deux inventaires.
 */

// `process.cwd()` et non `import.meta.url` : Vitest sert les modules de test par HTTP, et
// leur URL n'est pas un chemin de fichier. Vite lance toujours les tests depuis la racine
// du projet.
const root = process.cwd();

/** Attributs `#[tauri::command…]` et le nom de fonction qui suit. */
function commandAttributes(): { attribute: string; name: string }[] {
  const files = globSync("src-tauri/src/features/*/presentation/*.rs", {
    cwd: root,
  });
  const attributes: { attribute: string; name: string }[] = [];
  for (const file of files) {
    const source = readFileSync(`${root}/${file}`, "utf8");
    for (const match of source.matchAll(
      /#\[tauri::command((?:\([^)]*\))?)\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/g,
    )) {
      attributes.push({ attribute: match[1] ?? "", name: match[2]! });
    }
  }
  return attributes;
}

/** Noms de commandes déclarés côté Rust par `#[tauri::command]`. */
function rustCommands(): Set<string> {
  return new Set(commandAttributes().map((command) => command.name));
}

/**
 * Noms de commandes réellement appelés par les services frontend.
 *
 * Le paramètre de type est facultatif et peut être générique : `ipc<Page<Application>>(…)`.
 * Une capture qui s'arrête au premier `>` manque toute forme imbriquée — donc les cinq
 * commandes paginées, c'est-à-dire le chemin de données de chaque écran de liste.
 */
function calledCommands(): Set<string> {
  const files = globSync("src/features/*/services/*.ts", { cwd: root });
  const names = new Set<string>();
  for (const file of files) {
    const source = readFileSync(`${root}/${file}`, "utf8");
    for (const match of source.matchAll(/\bipc\s*(?:<[\s\S]*?>)?\s*\(\s*"([^"]+)"/g)) {
      names.add(match[1]!);
    }
  }
  return names;
}

/** Commandes enregistrées dans l'`invoke_handler` du bootstrap. */
function registeredCommands(): Set<string> {
  const source = readFileSync(`${root}/src-tauri/src/app/bootstrap.rs`, "utf8");
  const block = source.slice(
    source.indexOf("generate_handler!["),
    source.indexOf("])", source.indexOf("generate_handler![")),
  );
  return new Set([...block.matchAll(/::(\w+),/g)].map((m) => m[1]!));
}

describe("contrat IPC", () => {
  it("trouve bien les commandes des deux côtés", () => {
    // Garde-fou du test lui-même : si les expressions régulières cessaient de correspondre,
    // les comparaisons suivantes passeraient sur deux ensembles vides.
    expect(rustCommands().size).toBeGreaterThan(0);
    expect(calledCommands().size).toBeGreaterThan(0);
  });

  it("collecte aussi les appels dont le type de retour est générique", () => {
    // `ipc<Page<Application>>("…")` échappait à la capture, qui s'arrêtait au premier `>` :
    // les cinq commandes paginées — le chemin de données de tous les écrans de liste —
    // étaient absentes de l'inventaire, et les comparaisons suivantes ne les voyaient pas.
    const called = calledCommands();
    for (const command of [
      "applications_list_page",
      "companies_list_page",
      "contacts_list_page",
      "documents_resume_list_page",
      "documents_cover_letters_list_page",
    ]) {
      expect(called).toContain(command);
    }
  });

  it("couvre chaque service de feature", () => {
    // Second garde-fou : une capture qui cesserait de fonctionner pour une forme d'appel
    // donnée laisserait un service entier hors du contrat sans faire échouer les autres cas.
    const services = globSync("src/features/*/services/*.ts", { cwd: root });
    for (const service of services) {
      const source = readFileSync(`${root}/${service}`, "utf8");
      const quoted = [...source.matchAll(/"([a-z][a-z0-9_]*_[a-z0-9_]+)"/g)].map((m) => m[1]!);
      const commandes = quoted.filter((name) => rustCommands().has(name));
      expect(commandes.length, `${service} : aucune commande reconnue`).toBeGreaterThan(0);
      for (const command of commandes) {
        expect(calledCommands(), `${service} : ${command} non collectée`).toContain(command);
      }
    }
  });

  it("n'appelle que des commandes qui existent côté Rust", () => {
    const rust = rustCommands();
    const unknown = [...calledCommands()].filter((name) => !rust.has(name));
    expect(unknown).toEqual([]);
  });

  it("enregistre dans l'invoke_handler toutes les commandes appelées", () => {
    // Une commande déclarée mais absente du handler est invisible à l'exécution : Tauri
    // rejette « command not found », et le frontend n'affiche qu'un bandeau d'erreur.
    const registered = registeredCommands();
    const missing = [...calledCommands()].filter((name) => !registered.has(name));
    expect(missing).toEqual([]);
  });

  it("n'enregistre pas de commande sans déclaration Rust correspondante", () => {
    const rust = rustCommands();
    const orphans = [...registeredCommands()].filter((name) => !rust.has(name));
    expect(orphans).toEqual([]);
  });

  it("déclare les commandes de l'éditeur de CV et de l'ajout de compétence", () => {
    const commandNames = rustCommands();
    expect(commandNames).toContain("documents_resume_prepare");
    expect(commandNames).toContain("documents_resume_recalculate");
    expect(commandNames).toContain("documents_resume_apply_proposal");
    expect(commandNames).toContain("documents_resume_reject_proposal");
    expect(commandNames).toContain("profile_add_skill");
  });

  it("impose snake_case aux arguments IPC, comme les DTO serde", () => {
    // Tauri convertit `page_size` en `pageSize` par défaut : le frontend envoie
    // `page_size` et la commande échoue avec « missing required key pageSize ».
    const camel = commandAttributes()
      .filter((command) => !command.attribute.includes('rename_all = "snake_case"'))
      .map((command) => command.name);
    expect(camel).toEqual([]);
  });
});
