import { describe, expect, it } from "vitest";
import { candidatureFilterSchema, FILTRE_VIDE } from "../candidature-filter.schema";

describe("schéma des filtres de candidatures", () => {
  it("accepte un filtre entièrement vide", () => {
    // C'est l'état par défaut de l'écran : le refuser rendrait la liste inaccessible.
    expect(candidatureFilterSchema.safeParse(FILTRE_VIDE).success).toBe(true);
  });

  it("convertit les bornes de période au format base", () => {
    const resultat = candidatureFilterSchema.parse({
      ...FILTRE_VIDE,
      dateDebut: "01-08-2026",
      dateFin: "31-08-2026",
    });
    expect(resultat.dateDebut).toBe("2026-08-01");
    expect(resultat.dateFin).toBe("2026-08-31");
  });

  it("refuse une borne de période mal formée", () => {
    expect(
      candidatureFilterSchema.safeParse({ ...FILTRE_VIDE, dateDebut: "01/08/2026" }).success,
    ).toBe(false);
  });

  it("refuse une période inversée", () => {
    // Une période inversée ne renvoie jamais rien : l'écran afficherait un état vide
    // indiscernable d'une absence réelle de candidatures.
    const resultat = candidatureFilterSchema.safeParse({
      ...FILTRE_VIDE,
      dateDebut: "31-08-2026",
      dateFin: "01-08-2026",
    });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.path).toEqual(["dateFin"]);
    }
  });

  it("accepte une période d'un seul jour", () => {
    expect(
      candidatureFilterSchema.safeParse({
        ...FILTRE_VIDE,
        dateDebut: "20-08-2026",
        dateFin: "20-08-2026",
      }).success,
    ).toBe(true);
  });

  it("normalise l'entreprise non choisie en null", () => {
    expect(candidatureFilterSchema.parse({ ...FILTRE_VIDE, entrepriseId: "" }).entrepriseId).toBeNull();
  });
});
