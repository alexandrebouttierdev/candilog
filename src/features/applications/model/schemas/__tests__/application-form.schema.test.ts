import { describe, expect, it } from "vitest";
import { applicationFormSchema } from "../application-form.schema";

const BASE = {
  job_title: "Développeur Frontend",
  company_id: "11111111-1111-1111-1111-111111111111",
  contract_type: "CDI" as const,
  status: "EN_ATTENTE" as const,
  sent_date: "20-08-2026",
  job_url: "",
  notes: "",
};

describe("schéma du formulaire candidature", () => {
  it("accepte un formulaire complet", () => {
    expect(applicationFormSchema.safeParse(BASE).success).toBe(true);
  });

  it("transmet la date au format attendu par la base", () => {
    // Les filtres de période comparent des chaînes : envoyer « 20-08-2026 » ferait
    // disparaître la candidature des résultats sans qu'aucune erreur ne le signale.
    expect(applicationFormSchema.parse(BASE).sent_date).toBe("2026-08-20");
  });

  it("exige le poste", () => {
    expect(applicationFormSchema.safeParse({ ...BASE, job_title: "  " }).success).toBe(false);
  });

  it("exige l'entreprise", () => {
    // La clé étrangère est NOT NULL en base : une candidature sans entreprise serait
    // refusée par SQLite avec un message technique.
    expect(applicationFormSchema.safeParse({ ...BASE, company_id: "" }).success).toBe(false);
  });

  it("refuse une date invalide avec un message nommant le format", () => {
    const resultat = applicationFormSchema.safeParse({ ...BASE, sent_date: "32-08-2026" });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.message).toContain("JJ-MM-AAAA");
    }
  });

  it("refuse un lien d'offre non HTTP", () => {
    expect(
      applicationFormSchema.safeParse({ ...BASE, job_url: "javascript:alert(1)" }).success,
    ).toBe(false);
  });

  it("normalise les champs facultatifs vides en null", () => {
    const resultat = applicationFormSchema.parse(BASE);
    expect(resultat.job_url).toBeNull();
    expect(resultat.notes).toBeNull();
  });
});
