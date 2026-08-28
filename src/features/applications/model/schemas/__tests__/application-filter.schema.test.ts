import { describe, expect, it } from "vitest";
import { applicationFilterSchema, FILTER_VIDE } from "../application-filter.schema";

describe("schéma des filtres de candidatures", () => {
  it("accepte un filtre entièrement vide", () => {
    // C'est l'état par défaut de l'écran : le refuser rendrait la liste inaccessible.
    expect(applicationFilterSchema.safeParse(FILTER_VIDE).success).toBe(true);
  });

  it("convertit les bornes de période au format base", () => {
    const resultat = applicationFilterSchema.parse({
      ...FILTER_VIDE,
      start_date: "01-08-2026",
      end_date: "31-08-2026",
    });
    expect(resultat.start_date).toBe("2026-08-01");
    expect(resultat.end_date).toBe("2026-08-31");
  });

  it("refuse une borne de période mal formée", () => {
    expect(
      applicationFilterSchema.safeParse({ ...FILTER_VIDE, start_date: "01/08/2026" }).success,
    ).toBe(false);
  });

  it("refuse une période inversée", () => {
    // Une période inversée ne renvoie jamais rien : l'écran afficherait un état vide
    // indiscernable d'une absence réelle de candidatures.
    const resultat = applicationFilterSchema.safeParse({
      ...FILTER_VIDE,
      start_date: "31-08-2026",
      end_date: "01-08-2026",
    });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.path).toEqual(["end_date"]);
    }
  });

  it("accepte une période d'un seul jour", () => {
    expect(
      applicationFilterSchema.safeParse({
        ...FILTER_VIDE,
        start_date: "20-08-2026",
        end_date: "20-08-2026",
      }).success,
    ).toBe(true);
  });

  it("normalise l'entreprise non choisie en null", () => {
    expect(applicationFilterSchema.parse({ ...FILTER_VIDE, company_id: "" }).company_id).toBeNull();
  });
});
