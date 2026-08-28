import { describe, expect, it } from "vitest";
import { companyFormSchema } from "../company-form.schema";

/** Formulaire minimal valide, à altérer champ par champ. */
const BASE = {
  name: "Nova Digital",
  sector_id: "",
  type: "",
  website: "",
  city: "",
  address: "",
  notes: "",
};

describe("schéma du formulaire entreprise", () => {
  it("accepte le nom seul, les autres champs étant facultatifs", () => {
    expect(companyFormSchema.safeParse(BASE).success).toBe(true);
  });

  it("refuse un nom vide ou fait d'espaces", () => {
    expect(companyFormSchema.safeParse({ ...BASE, name: "" }).success).toBe(false);
    expect(companyFormSchema.safeParse({ ...BASE, name: "   " }).success).toBe(false);
  });

  it("normalise les champs vides en null plutôt qu'en chaîne vide", () => {
    // La base distingue NULL de '' : les `coalesce` et les `LIKE` de la recherche ne
    // traitent pas les deux de la même façon.
    const resultat = companyFormSchema.parse(BASE);
    expect(resultat.city).toBeNull();
    expect(resultat.sector_id).toBeNull();
    expect(resultat.notes).toBeNull();
  });

  it("supprime les espaces autour du nom", () => {
    expect(companyFormSchema.parse({ ...BASE, name: "  Atlas Studio  " }).name).toBe(
      "Atlas Studio",
    );
  });

  it("accepte un site web en HTTP ou HTTPS", () => {
    expect(
      companyFormSchema.safeParse({ ...BASE, website: "https://novadigital.fr" }).success,
    ).toBe(true);
  });

  it("refuse un site web dont le schéma n'est pas HTTP", () => {
    // Le champ est ouvert d'un clic depuis la fiche : un `javascript:` y ferait exécuter du
    // code au lieu d'ouvrir une page. Le backend applique la même règle.
    const resultat = companyFormSchema.safeParse({
      ...BASE,
      website: "javascript:alert(1)",
    });
    expect(resultat.success).toBe(false);
  });

  it("refuse un site web mal formé", () => {
    expect(companyFormSchema.safeParse({ ...BASE, website: "pas une url" }).success).toBe(
      false,
    );
  });
});
