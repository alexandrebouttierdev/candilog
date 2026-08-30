import { describe, expect, it } from "vitest";
import {
  applicationFilterSchema,
  type ApplicationFilterInput,
} from "../application-filter.schema";

/** Filtre vide tel que saisi : les bornes sont des chaînes avant validation. */
const VIDE: ApplicationFilterInput = {
  status: [],
  application_type: [],
  contract_type_code: [],
  professional_domain_id: [],
  company_type_id: [],
  company_size: [],
  sector_id: [],
  weekly_work_schedule: [],
  min_weekly_hours: "",
  max_weekly_hours: "",
  company_id: "",
  city: "",
  job_title: "",
  start_date: "",
  end_date: "",
};

describe("schéma des filtres de candidatures", () => {
  it("accepte un filtre entièrement vide", () => {
    // C'est l'état par défaut de l'écran : le refuser rendrait la liste inaccessible.
    expect(applicationFilterSchema.safeParse(VIDE).success).toBe(true);
  });

  it("convertit les bornes de période au format base", () => {
    const resultat = applicationFilterSchema.parse({
      ...VIDE,
      start_date: "01-08-2026",
      end_date: "31-08-2026",
    });
    expect(resultat.start_date).toBe("2026-08-01");
    expect(resultat.end_date).toBe("2026-08-31");
  });

  it("refuse une borne de période mal formée", () => {
    expect(
      applicationFilterSchema.safeParse({ ...VIDE, start_date: "01/08/2026" }).success,
    ).toBe(false);
  });

  it("refuse une période inversée", () => {
    // Une période inversée ne renvoie jamais rien : l'écran afficherait un état vide
    // indiscernable d'une absence réelle de candidatures.
    const resultat = applicationFilterSchema.safeParse({
      ...VIDE,
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
        ...VIDE,
        start_date: "20-08-2026",
        end_date: "20-08-2026",
      }).success,
    ).toBe(true);
  });

  it("normalise l'entreprise non choisie en null", () => {
    expect(applicationFilterSchema.parse({ ...VIDE, company_id: "" }).company_id).toBeNull();
  });

  it("accepte plusieurs valeurs par référentiel", () => {
    const resultat = applicationFilterSchema.parse({
      ...VIDE,
      status: ["ENTRETIEN", "REFUS"],
      contract_type_code: ["CDI", "MIS"],
      professional_domain_id: ["M18"],
      company_type_id: ["IT_SERVICES_COMPANY", "FINAL_CLIENT"],
      company_size: ["PME", "ETI"],
      application_type: ["SPONTANEE"],
      weekly_work_schedule: ["PART_TIME"],
    });
    expect(resultat.status).toEqual(["ENTRETIEN", "REFUS"]);
    expect(resultat.contract_type_code).toEqual(["CDI", "MIS"]);
    expect(resultat.company_type_id).toEqual(["IT_SERVICES_COMPANY", "FINAL_CLIENT"]);
    expect(resultat.company_size).toEqual(["PME", "ETI"]);
  });

  describe("amplitude horaire", () => {
    it("convertit les bornes saisies en nombres", () => {
      const resultat = applicationFilterSchema.parse({
        ...VIDE,
        min_weekly_hours: "17,5",
        max_weekly_hours: "35",
      });
      expect(resultat.min_weekly_hours).toBe(17.5);
      expect(resultat.max_weekly_hours).toBe(35);
    });

    it("refuse une amplitude inversée", () => {
      const resultat = applicationFilterSchema.safeParse({
        ...VIDE,
        min_weekly_hours: "35",
        max_weekly_hours: "20",
      });
      expect(resultat.success).toBe(false);
      if (!resultat.success) {
        expect(resultat.error.issues[0]?.path).toEqual(["max_weekly_hours"]);
      }
    });

    it("refuse une borne non numérique ou hors limites", () => {
      for (const min_weekly_hours of ["abc", "0", "-2", "500"]) {
        expect(
          applicationFilterSchema.safeParse({ ...VIDE, min_weekly_hours }).success,
          `« ${min_weekly_hours} » aurait dû être refusé`,
        ).toBe(false);
      }
    });
  });
});
