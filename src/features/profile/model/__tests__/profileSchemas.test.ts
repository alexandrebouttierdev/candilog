import { experienceSchema, identitySchema } from "../profileSchemas";

describe("schémas du profil", () => {
  it("refuse une date de fin pour un poste actuel", () => {
    const resultat = experienceSchema.safeParse({
      title: "Développeuse",
      company: "Nova",
      location: "",
      start_date: "2024-01",
      end_date: "2026-08",
      current: true,
      description: "",
    });

    expect(resultat.success).toBe(false);
  });

  it("transforme les champs facultatifs vides en null", () => {
    const resultat = identitySchema.parse({
      first_name: " Camille ",
      name: "Rivet",
      email: "",
      phone: "",
      city: "",
      address: "",
      title: "",
      resume: "",
      birth_date: "",
      age: "",
      availability: "",
      desired_contracts: "",
      linkedin: "",
      github: "",
      website: "",
    });

    expect(resultat.first_name).toBe("Camille");
    expect(resultat.phone).toBeNull();
    expect(resultat.address).toBeNull();
    expect(resultat.birth_date).toBeNull();
    expect(resultat.age).toBeNull();
    expect(resultat.availability).toBeNull();
    expect(resultat.desired_contracts).toBeNull();
    expect(resultat.website).toBeNull();
  });

  it("accepte un âge numérique facultatif", () => {
    const resultat = identitySchema.parse({
      first_name: "Camille",
      name: "Rivet",
      email: "",
      phone: "",
      city: "",
      address: "",
      title: "",
      resume: "",
      birth_date: "14 avril 1992",
      age: "34 ans",
      availability: "Sous 1 mois",
      desired_contracts: "CDI",
      linkedin: "",
      github: "",
      website: "",
    });
    expect(resultat.age).toBe(34);
    expect(resultat.availability).toBe("Sous 1 mois");
  });
});
