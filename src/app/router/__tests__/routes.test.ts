import { describe, expect, it } from "vitest";
import { Sections, sectionForPath } from "../routes";

describe("carte de navigation", () => {
  it("reprend les sept sections du rail des maquettes", () => {
    expect(Sections.map((section) => section.key)).toEqual([
      "accueil",
      "suivi",
      "relations",
      "documents",
      "analyses",
      "profil",
      "reglages",
    ]);
  });

  it("donne à chaque section au moins un écran", () => {
    for (const section of Sections) {
      expect(section.routes.length).toBeGreaterThan(0);
    }
  });

  it("n'attribue jamais deux fois le même chemin", () => {
    const paths = Sections.flatMap((section) => section.routes.map((route) => route.path));
    expect(new Set(paths).size).toBe(paths.length);
  });

  it("rattache un chemin à sa section pour l'état sélectionné du rail", () => {
    expect(sectionForPath("/tracking/calendar").key).toBe("suivi");
    expect(sectionForPath("/documents/generate-resume").key).toBe("documents");
    expect(sectionForPath("/settings/about").key).toBe("reglages");
  });

  it("ne fait correspondre la racine qu'à elle-même", () => {
    // `startsWith("/")` serait vrai de tous les chemins : la racine, seule route dont le
    // préfixe est contenu dans tous les autres, doit être comparée à l'identique.
    expect(sectionForPath("/").key).toBe("accueil");
    expect(sectionForPath("/relations/companies").key).toBe("relations");
  });

  it("retombe sur l'accueil pour un chemin inconnu", () => {
    expect(sectionForPath("/inexistant").key).toBe("accueil");
  });
});

describe("écrans migrés", () => {
  it("couvre tous les chemins du rail, sans jalon restant", async () => {
    const { MIGRATED_PATHS } = await import("../AppRouter");
    const paths = Sections.flatMap((section) => section.routes.map((route) => route.path));
    expect([...MIGRATED_PATHS].sort()).toEqual([...paths].sort());
  });
});
