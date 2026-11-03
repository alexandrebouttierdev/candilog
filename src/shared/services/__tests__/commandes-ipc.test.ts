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

/**
 * Constantes `const NOM … = "valeur"` d'un fichier, pour résoudre un nom d'événement passé
 * par identifiant plutôt qu'en littéral (`MANAGED_DOWNLOAD_PROGRESS_EVENT`).
 */
function stringConstants(source: string): Map<string, string> {
  const constants = new Map<string, string>();
  for (const match of source.matchAll(/const\s+(\w+)\s*(?::\s*&?'?\w+)?\s*=\s*"([^"]+)"/g)) {
    constants.set(match[1]!, match[2]!);
  }
  return constants;
}

/** Premier argument d'un appel `nom(...)`, résolu en littéral quand c'est possible. */
function firstArguments(source: string, callee: string): string[] {
  const constants = stringConstants(source);
  const appels = source.matchAll(
    new RegExp(`\\b${callee}\\s*(?:<[\\s\\S]*?>)?\\s*\\(\\s*(?:"([^"]+)"|(\\w+))`, "g"),
  );
  return [...appels].flatMap((match) => {
    if (match[1] !== undefined) return [match[1]];
    const resolu = constants.get(match[2]!);
    return resolu === undefined ? [] : [resolu];
  });
}

/**
 * Contrat des **événements** Tauri, jumeau de celui des commandes ci-dessus.
 *
 * Même angle mort, et il a déjà mordu : un nom d'événement n'est vérifié ni par Rust ni par
 * TypeScript. Un `listen` posé sur un nom que plus personne n'émet ne se déclenche jamais et
 * ne lève aucune erreur — la documentation a longtemps annoncé des `local-ai://*` que le
 * backend n'émettait pas. Les deux inventaires sont donc comparés.
 */
describe("contrat des événements", () => {
  /** Noms réellement émis par la couche `presentation` des features. */
  function rustEvents(): Set<string> {
    const files = globSync("src-tauri/src/features/*/presentation/*.rs", { cwd: root });
    const names = new Set<string>();
    for (const file of files) {
      const source = readFileSync(`${root}/${file}`, "utf8");
      for (const name of firstArguments(source, "emit")) names.add(name);
    }
    return names;
  }

  /** Noms réellement écoutés par le frontend. */
  function listenedEvents(): Set<string> {
    const files = globSync("src/{app,features,shared}/**/*.{ts,tsx}", { cwd: root }).filter(
      (file) => !file.includes("__tests__"),
    );
    const names = new Set<string>();
    for (const file of files) {
      const source = readFileSync(`${root}/${file}`, "utf8");
      if (!source.includes("@tauri-apps/api/event")) continue;
      for (const name of firstArguments(source, "listen")) names.add(name);
    }
    return names;
  }

  it("trouve bien les événements des deux côtés", () => {
    // Garde-fou du test : sans lui, deux ensembles vides rendraient les comparaisons vraies.
    expect(rustEvents().size).toBeGreaterThan(0);
    expect(listenedEvents().size).toBeGreaterThan(0);
  });

  it("résout un nom passé par constante autant qu'un littéral", () => {
    // `managed-ollama://download-progress` est émis et écouté via une constante de module :
    // une capture limitée aux littéraux l'aurait manqué des deux côtés à la fois.
    expect(rustEvents()).toContain("managed-ollama://download-progress");
    expect(listenedEvents()).toContain("managed-ollama://download-progress");
  });

  it("n'écoute que des événements réellement émis", () => {
    const emis = rustEvents();
    const fantomes = [...listenedEvents()].filter((name) => !emis.has(name));
    expect(fantomes).toEqual([]);
  });

  it("documente la liste complète des événements dans ARCHITECTURE.md", () => {
    // La table de `docs/ARCHITECTURE.md` se donne pour exhaustive : qu'elle le reste.
    const architecture = readFileSync(`${root}/docs/ARCHITECTURE.md`, "utf8");
    for (const name of rustEvents()) {
      expect(architecture, `${name} absent de docs/ARCHITECTURE.md`).toContain(`\`${name}\``);
    }
  });
});

/** Découpe au premier niveau d'imbrication (`<>`, `()`, `{}`, `[]`) sur les virgules. */
function splitTopLevel(text: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  for (const char of text) {
    if ("<({[".includes(char)) depth += 1;
    if (">)}]".includes(char)) depth -= 1;
    if (char === "," && depth === 0) {
      parts.push(current);
      current = "";
    } else {
      current += char;
    }
  }
  parts.push(current);
  return parts.map((part) => part.trim()).filter((part) => part !== "");
}

/** Contenu entre la parenthèse (ou accolade) ouvrante à `start` et sa fermante. */
function balanced(source: string, start: number): string {
  const open = source[start]!;
  const close = open === "(" ? ")" : "}";
  let depth = 0;
  for (let index = start; index < source.length; index += 1) {
    if (source[index] === open) depth += 1;
    if (source[index] === close) depth -= 1;
    if (depth === 0) return source.slice(start + 1, index);
  }
  throw new Error(`délimiteur non refermé à ${start}`);
}

/** Types injectés par Tauri, que le frontend ne transmet jamais. */
const INJECTED = /^(?:tauri::)?(?:State|AppHandle|Window|WebviewWindow|Webview)\b/;

/**
 * Paramètres attendus par chaque commande Rust : nom → caractère facultatif (`Option<…>`).
 */
function rustParameters(): Map<string, Map<string, boolean>> {
  const files = globSync("src-tauri/src/features/*/presentation/*.rs", { cwd: root });
  const commands = new Map<string, Map<string, boolean>>();
  for (const file of files) {
    const source = readFileSync(`${root}/${file}`, "utf8");
    for (const match of source.matchAll(
      /#\[tauri::command(?:\([^)]*\))?\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)(?:<[^>]*>)?\s*\(/g,
    )) {
      const open = match.index + match[0].length - 1;
      const parameters = new Map<string, boolean>();
      for (const parameter of splitTopLevel(balanced(source, open))) {
        const [name, type] = parameter.split(/:(.*)/s).map((part) => part.trim());
        if (!name || !type || INJECTED.test(type)) continue;
        parameters.set(name.replace(/^_/, ""), type.startsWith("Option<"));
      }
      commands.set(match[1]!, parameters);
    }
  }
  return commands;
}

/**
 * Clés d'arguments transmises par chaque appel `ipc("commande", { … })` des services.
 * `null` quand l'objet n'est pas littéral (variable, décomposition) : non vérifiable ici.
 */
function calledArguments(): { command: string; file: string; keys: string[] | null }[] {
  const files = globSync("src/{features/*/services,shared/services}/*.ts", { cwd: root });
  const calls: { command: string; file: string; keys: string[] | null }[] = [];
  for (const file of files) {
    const source = readFileSync(`${root}/${file}`, "utf8");
    for (const match of source.matchAll(/\bipc\s*(?:<[\s\S]*?>)?\s*\(\s*"([^"]+)"\s*(,?)\s*/g)) {
      const command = match[1]!;
      const after = match.index + match[0].length;
      if (match[2] === "" || source[after] === ")") {
        calls.push({ command, file, keys: [] });
        continue;
      }
      if (source[after] !== "{") {
        calls.push({ command, file, keys: null });
        continue;
      }
      const entries = splitTopLevel(balanced(source, after));
      if (entries.some((entry) => entry.startsWith("..."))) {
        calls.push({ command, file, keys: null });
        continue;
      }
      calls.push({ command, file, keys: entries.map((entry) => entry.split(":")[0]!.trim().replace(/"/g, "")) });
    }
  }
  return calls;
}

/**
 * Noms des **arguments**, troisième angle mort après les noms de commandes et
 * d'événements : `ipc("companies_get", { identifiant })` compile des deux côtés et ne
 * rejette « missing required key id » qu'à l'exécution. Le type de retour, lui, reste
 * affirmé par l'appelant, mais il s'appuie sur les types générés par ts-rs.
 */
describe("contrat des arguments IPC", () => {
  it("lit les paramètres des deux côtés", () => {
    const rust = rustParameters();
    expect(rust.get("companies_update")).toEqual(
      new Map([
        ["id", false],
        ["input", false],
      ]),
    );
    expect(rust.get("ai_select_resume_file")?.size).toBe(0);
    const calls = calledArguments();
    expect(calls.filter((call) => call.keys !== null).length).toBeGreaterThan(80);
  });

  it("transmet exactement les arguments attendus par chaque commande", () => {
    const rust = rustParameters();
    const ecarts: string[] = [];
    for (const call of calledArguments()) {
      const expected = rust.get(call.command);
      if (!expected || call.keys === null) continue;
      const unknown = call.keys.filter((key) => !expected.has(key));
      const missing = [...expected]
        .filter(([name, optional]) => !optional && !call.keys!.includes(name))
        .map(([name]) => name);
      if (unknown.length > 0 || missing.length > 0) {
        ecarts.push(`${call.file} ${call.command} : inconnus [${unknown.join(", ")}], manquants [${missing.join(", ")}]`);
      }
    }
    expect(ecarts).toEqual([]);
  });

  it("ne laisse hors contrôle que les appels paginés, dont l'objet est construit à part", () => {
    const opaques = calledArguments()
      .filter((call) => call.keys === null)
      .map((call) => call.command)
      .sort();
    expect(opaques).toEqual(
      [
        "applications_list_page",
        "companies_list_page",
        "contacts_list_page",
        "documents_cover_letters_list_page",
        "documents_resume_list_page",
      ].filter((command) => opaques.includes(command)),
    );
  });
});
