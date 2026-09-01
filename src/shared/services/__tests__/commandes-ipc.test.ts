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
const racine = process.cwd();

/** Attributs `#[tauri::command…]` et le nom de fonction qui suit. */
function attributsCommandes(): { attribut: string; nom: string }[] {
  const fichiers = globSync("src-tauri/src/features/*/presentation/commands.rs", {
    cwd: racine,
  });
  const attributs: { attribut: string; nom: string }[] = [];
  for (const fichier of fichiers) {
    const source = readFileSync(`${racine}/${fichier}`, "utf8");
    for (const correspondance of source.matchAll(
      /#\[tauri::command((?:\([^)]*\))?)\]\s*(?:pub\s+)?async\s+fn\s+(\w+)/g,
    )) {
      attributs.push({ attribut: correspondance[1] ?? "", nom: correspondance[2]! });
    }
  }
  return attributs;
}

/** Noms de commandes déclarés côté Rust par `#[tauri::command]`. */
function commandesRust(): Set<string> {
  return new Set(attributsCommandes().map((commande) => commande.nom));
}

/**
 * Noms de commandes réellement appelés par les services frontend.
 *
 * Le paramètre de type est facultatif et peut être générique : `ipc<Page<Application>>(…)`.
 * Une capture qui s'arrête au premier `>` manque toute forme imbriquée — donc les cinq
 * commandes paginées, c'est-à-dire le chemin de données de chaque écran de liste.
 */
function commandesAppelees(): Set<string> {
  const fichiers = globSync("src/features/*/services/*.ts", { cwd: racine });
  const noms = new Set<string>();
  for (const fichier of fichiers) {
    const source = readFileSync(`${racine}/${fichier}`, "utf8");
    for (const correspondance of source.matchAll(/\bipc\s*(?:<[\s\S]*?>)?\s*\(\s*"([^"]+)"/g)) {
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

  it("collecte aussi les appels dont le type de retour est générique", () => {
    // `ipc<Page<Application>>("…")` échappait à la capture, qui s'arrêtait au premier `>` :
    // les cinq commandes paginées — le chemin de données de tous les écrans de liste —
    // étaient absentes de l'inventaire, et les comparaisons suivantes ne les voyaient pas.
    const appelees = commandesAppelees();
    for (const commande of [
      "applications_list_page",
      "companies_list_page",
      "contacts_list_page",
      "documents_resume_list_page",
      "documents_cover_letters_list_page",
    ]) {
      expect(appelees).toContain(commande);
    }
  });

  it("couvre chaque service de feature", () => {
    // Second garde-fou : une capture qui cesserait de fonctionner pour une forme d'appel
    // donnée laisserait un service entier hors du contrat sans faire échouer les autres cas.
    const services = globSync("src/features/*/services/*.ts", { cwd: racine });
    for (const service of services) {
      const source = readFileSync(`${racine}/${service}`, "utf8");
      const attendus = [...source.matchAll(/"([a-z][a-z0-9_]*_[a-z0-9_]+)"/g)].map((m) => m[1]!);
      const commandes = attendus.filter((nom) => commandesRust().has(nom));
      expect(commandes.length, `${service} : aucune commande reconnue`).toBeGreaterThan(0);
      for (const commande of commandes) {
        expect(commandesAppelees(), `${service} : ${commande} non collectée`).toContain(commande);
      }
    }
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

  it("déclare les commandes de l'éditeur de CV et de l'ajout de compétence", () => {
    const commandNames = commandesRust();
    expect(commandNames).toContain("documents_resume_prepare");
    expect(commandNames).toContain("documents_resume_recalculate");
    expect(commandNames).toContain("documents_resume_apply_proposal");
    expect(commandNames).toContain("documents_resume_reject_proposal");
    expect(commandNames).toContain("profile_add_skill");
  });

  it("impose snake_case aux arguments IPC, comme les DTO serde", () => {
    // Tauri convertit `page_size` en `pageSize` par défaut : le frontend envoie
    // `page_size` et la commande échoue avec « missing required key pageSize ».
    const camel = attributsCommandes()
      .filter((commande) => !commande.attribut.includes('rename_all = "snake_case"'))
      .map((commande) => commande.nom);
    expect(camel).toEqual([]);
  });
});
