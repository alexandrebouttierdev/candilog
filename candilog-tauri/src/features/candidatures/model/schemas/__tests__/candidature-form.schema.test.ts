import { describe, expect, it } from "vitest";
import {
  candidatureFormSchema,
  versDateAffichee,
  versDateIso,
} from "../candidature-form.schema";

const BASE = {
  poste: "Développeur Frontend",
  entrepriseId: "11111111-1111-1111-1111-111111111111",
  typeContrat: "CDI" as const,
  statut: "EN_ATTENTE" as const,
  dateEnvoi: "20-08-2026",
  lienOffre: "",
  notes: "",
};

describe("conversion de date", () => {
  it("convertit le format saisi en format base", () => {
    expect(versDateIso("20-08-2026")).toBe("2026-08-20");
  });

  it("fait l'aller-retour sans perte", () => {
    expect(versDateAffichee(versDateIso("29-02-2024") as string)).toBe("29-02-2024");
  });

  it("refuse une date qui n'existe pas", () => {
    // `new Date("2026-02-31")` décale au 3 mars sans erreur : seule la comparaison de la
    // valeur relue permet de refuser un jour inexistant.
    expect(versDateIso("31-02-2026")).toBeNull();
    expect(versDateIso("29-02-2026")).toBeNull();
  });

  it("accepte le 29 février d'une année bissextile", () => {
    expect(versDateIso("29-02-2024")).toBe("2024-02-29");
  });

  it("refuse une saisie hors format", () => {
    for (const saisie of ["2026-08-20", "20/08/2026", "20 août 2026", "20-8-2026", ""]) {
      expect(versDateIso(saisie)).toBeNull();
    }
  });

  it("laisse intacte une date déjà affichable qu'on ne sait pas relire", () => {
    // Les lignes héritées portent parfois un horodatage ISO complet : mieux vaut l'afficher
    // tel quel qu'afficher « Invalid Date ».
    expect(versDateAffichee("2026-08-20T10:00:00Z")).toBe("20-08-2026");
    expect(versDateAffichee("date inconnue")).toBe("date inconnue");
  });
});

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
