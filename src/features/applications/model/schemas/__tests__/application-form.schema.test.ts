import { describe, expect, it } from "vitest";
import { applicationFormSchema } from "../application-form.schema";

const BASE = {
  job_title: "Développeur Frontend",
  company_id: "11111111-1111-1111-1111-111111111111",
  contact_id: "",
  application_type: "OFFRE" as const,
  contract_type_code: "CDI",
  weekly_work_schedule: "UNSPECIFIED" as const,
  weekly_hours: "",
  professional_domain_id: "",
  city: "",
  address: "",
  company_type_id: "",
  status: "EN_ATTENTE" as const,
  sent_date: "20-08-2026",
  job_url: "https://example.org/offre",
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

  it("exige un type de contrat", () => {
    expect(
      applicationFormSchema.safeParse({ ...BASE, contract_type_code: "" }).success,
    ).toBe(false);
  });

  it("refuse une date invalide avec un message nommant le format", () => {
    const resultat = applicationFormSchema.safeParse({ ...BASE, sent_date: "32-08-2026" });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.message).toContain("JJ-MM-AAAA");
    }
  });

  describe("lien de l'offre", () => {
    it("est obligatoire pour une candidature à une offre", () => {
      const resultat = applicationFormSchema.safeParse({ ...BASE, job_url: "" });
      expect(resultat.success).toBe(false);
      if (!resultat.success) {
        expect(resultat.error.issues[0]?.path).toEqual(["job_url"]);
      }
    });

    it("refuse un schéma autre que HTTP(S)", () => {
      // Le lien est ouvert d'un clic depuis la fiche : un `javascript:` y ferait exécuter
      // du code au lieu d'ouvrir une page.
      expect(
        applicationFormSchema.safeParse({ ...BASE, job_url: "javascript:alert(1)" }).success,
      ).toBe(false);
    });

    it("est effacé pour une candidature spontanée", () => {
      const resultat = applicationFormSchema.parse({
        ...BASE,
        application_type: "SPONTANEE",
        job_url: "https://example.org/offre",
      });
      expect(resultat.job_url).toBeNull();
    });

    it("n'est pas exigé pour une candidature spontanée", () => {
      expect(
        applicationFormSchema.safeParse({
          ...BASE,
          application_type: "SPONTANEE",
          job_url: "",
        }).success,
      ).toBe(true);
    });
  });

  describe("nombre d'heures par semaine", () => {
    it("accepte un champ vide et le normalise en null", () => {
      expect(applicationFormSchema.parse(BASE).weekly_hours).toBeNull();
    });

    it("accepte la virgule décimale française", () => {
      expect(applicationFormSchema.parse({ ...BASE, weekly_hours: "17,5" }).weekly_hours).toBe(
        17.5,
      );
    });

    it("refuse une valeur nulle, négative ou irréaliste", () => {
      for (const weekly_hours of ["0", "-3", "200", "abc"]) {
        expect(
          applicationFormSchema.safeParse({ ...BASE, weekly_hours }).success,
          `« ${weekly_hours} » aurait dû être refusé`,
        ).toBe(false);
      }
    });
  });

  it("normalise les surcharges vides en null plutôt que d'y recopier l'héritage", () => {
    // Une surcharge vide signifie « hériter de l'entreprise » : y écrire la valeur héritée
    // la figerait, et un changement d'entreprise laisserait la ville de la précédente.
    const resultat = applicationFormSchema.parse(BASE);
    expect(resultat.city).toBeNull();
    expect(resultat.address).toBeNull();
    expect(resultat.company_type_id).toBeNull();
    expect(resultat.professional_domain_id).toBeNull();
    expect(resultat.notes).toBeNull();
  });
});
