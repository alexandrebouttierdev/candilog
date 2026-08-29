import { describe, expect, it } from "vitest";
import { interviewFormSchema } from "../interview-form.schema";

const BASE = {
  application_id: "11111111-1111-1111-1111-111111111111",
  contact_id: "",
  date: "25-08-2026",
  time: "14:00",
  type: "Visio" as const,
  location: "",
  notes: "",
  minutes: "",
};

describe("schéma du formulaire entretien", () => {
  it("compose un horodatage RFC 3339 à partir de la date et de l'heure", () => {
    const resultat = interviewFormSchema.parse(BASE);
    expect(resultat.interview_date).toMatch(/^2026-08-25T14:00:00[+-]\d{2}:\d{2}$/);
  });

  it("refuse une date inexistante", () => {
    const resultat = interviewFormSchema.safeParse({ ...BASE, date: "31-02-2026" });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.message).toContain("JJ-MM-AAAA");
    }
  });

  it("refuse une heure hors 24 h", () => {
    const resultat = interviewFormSchema.safeParse({ ...BASE, time: "24:00" });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.message).toContain("HH:MM");
    }
  });

  it("accepte une date issue du calendrier natif", () => {
    expect(
      interviewFormSchema.safeParse({ ...BASE, date: "25-08-2026", time: "09:05" }).success,
    ).toBe(true);
  });
});
