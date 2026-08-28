import { describe, expect, it } from "vitest";
import { candidatureFormSchema } from "../candidature-form.schema";

const BASE = {
  poste: "Développeur Frontend",
  entrepriseId: "11111111-1111-1111-1111-111111111111",
  typeContrat: "CDI" as const,
  statut: "EN_ATTENTE" as const,
  dateEnvoi: "20-08-2026",
  lienOffre: "",
  notes: "",
};

describe("schéma du formulaire candidature", () => {
  it("accepte un formulaire complet", () => {
    expect(candidatureFormSchema.safeParse(BASE).success).toBe(true);
  });

  it("transmet la date au format attendu par la base", () => {
    // Les filtres de période comparent des chaînes : envoyer « 20-08-2026 » ferait
    // disparaître la candidature des résultats sans qu'aucune erreur ne le signale.
    expect(candidatureFormSchema.parse(BASE).dateEnvoi).toBe("2026-08-20");
  });

  it("exige le poste", () => {
    expect(candidatureFormSchema.safeParse({ ...BASE, poste: "  " }).success).toBe(false);
  });

  it("exige l'entreprise", () => {
    // La clé étrangère est NOT NULL en base : une candidature sans entreprise serait
    // refusée par SQLite avec un message technique.
    expect(candidatureFormSchema.safeParse({ ...BASE, entrepriseId: "" }).success).toBe(false);
  });

  it("refuse une date invalide avec un message nommant le format", () => {
    const resultat = candidatureFormSchema.safeParse({ ...BASE, dateEnvoi: "32-08-2026" });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.message).toContain("JJ-MM-AAAA");
    }
  });

  it("refuse un lien d'offre non HTTP", () => {
    expect(
      candidatureFormSchema.safeParse({ ...BASE, lienOffre: "javascript:alert(1)" }).success,
    ).toBe(false);
  });

  it("normalise les champs facultatifs vides en null", () => {
    const resultat = candidatureFormSchema.parse(BASE);
    expect(resultat.lienOffre).toBeNull();
    expect(resultat.notes).toBeNull();
  });
});
