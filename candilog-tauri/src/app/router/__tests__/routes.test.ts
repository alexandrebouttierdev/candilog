import { describe, expect, it } from "vitest";
import { SECTIONS, sectionForPath } from "../routes";

describe("carte de navigation", () => {
  it("reprend les sept sections du rail des maquettes", () => {
    expect(SECTIONS.map((section) => section.key)).toEqual([
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
    for (const section of SECTIONS) {
      expect(section.routes.length).toBeGreaterThan(0);
    }
  });

  it("n'attribue jamais deux fois le même chemin", () => {
    const paths = SECTIONS.flatMap((section) => section.routes.map((route) => route.path));
    expect(new Set(paths).size).toBe(paths.length);
  });

  it("rattache un chemin à sa section pour l'état sélectionné du rail", () => {
    expect(sectionForPath("/suivi/calendrier").key).toBe("suivi");
    expect(sectionForPath("/documents/generer-cv").key).toBe("documents");
    expect(sectionForPath("/reglages/a-propos").key).toBe("reglages");
  });

  it("ne fait correspondre la racine qu'à elle-même", () => {
    // `startsWith("/")` serait vrai de tous les chemins : la racine, seule route dont le
    // préfixe est contenu dans tous les autres, doit être comparée à l'identique.
    expect(sectionForPath("/").key).toBe("accueil");
    expect(sectionForPath("/relations/entreprises").key).toBe("relations");
  });

  it("retombe sur l'accueil pour un chemin inconnu", () => {
    expect(sectionForPath("/inexistant").key).toBe("accueil");
  });
});
