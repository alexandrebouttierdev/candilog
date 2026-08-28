import { experienceSchema, identiteSchema } from "../profil.schemas";

describe("schémas du profil", () => {
  it("refuse une date de fin pour un poste actuel", () => {
    const resultat = experienceSchema.safeParse({
      intitule: "Développeuse",
      entreprise: "Nova",
      lieu: "",
      dateDebut: "2024-01",
      dateFin: "2026-08",
      posteActuel: true,
      description: "",
    });

    expect(resultat.success).toBe(false);
  });

  it("transforme les champs facultatifs vides en null", () => {
    const resultat = identiteSchema.parse({
      prenom: " Camille ",
      nom: "Rivet",
      email: "",
      telephone: "",
      ville: "",
      titre: "",
      resume: "",
      linkedin: "",
      github: "",
      siteWeb: "",
    });

    expect(resultat.prenom).toBe("Camille");
    expect(resultat.telephone).toBeNull();
    expect(resultat.siteWeb).toBeNull();
  });
});
