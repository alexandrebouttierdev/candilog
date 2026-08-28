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
      title: "",
      resume: "",
      linkedin: "",
      github: "",
      website: "",
    });

    expect(resultat.first_name).toBe("Camille");
    expect(resultat.phone).toBeNull();
    expect(resultat.website).toBeNull();
  });
});
