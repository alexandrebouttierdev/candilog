import { describe, expect, it } from "vitest";
import { entrepriseFormSchema } from "../entreprise-form.schema";

/** Formulaire minimal valide, à altérer champ par champ. */
const BASE = {
  nom: "Nova Digital",
  secteurId: "",
  type: "",
  siteWeb: "",
  ville: "",
  adresse: "",
  notes: "",
};

describe("schéma du formulaire entreprise", () => {
  it("accepte le nom seul, les autres champs étant facultatifs", () => {
    expect(entrepriseFormSchema.safeParse(BASE).success).toBe(true);
  });

  it("refuse un nom vide ou fait d'espaces", () => {
    expect(entrepriseFormSchema.safeParse({ ...BASE, nom: "" }).success).toBe(false);
    expect(entrepriseFormSchema.safeParse({ ...BASE, nom: "   " }).success).toBe(false);
  });

  it("normalise les champs vides en null plutôt qu'en chaîne vide", () => {
    // La base distingue NULL de '' : les `coalesce` et les `LIKE` de la recherche ne
    // traitent pas les deux de la même façon.
    const resultat = entrepriseFormSchema.parse(BASE);
    expect(resultat.ville).toBeNull();
    expect(resultat.secteurId).toBeNull();
    expect(resultat.notes).toBeNull();
  });

  it("supprime les espaces autour du nom", () => {
    expect(entrepriseFormSchema.parse({ ...BASE, nom: "  Atlas Studio  " }).nom).toBe(
      "Atlas Studio",
    );
  });

  it("accepte un site web en HTTP ou HTTPS", () => {
    expect(
      entrepriseFormSchema.safeParse({ ...BASE, siteWeb: "https://novadigital.fr" }).success,
    ).toBe(true);
  });

  it("refuse un site web dont le schéma n'est pas HTTP", () => {
    // Le champ est ouvert d'un clic depuis la fiche : un `javascript:` y ferait exécuter du
    // code au lieu d'ouvrir une page. Le backend applique la même règle.
    const resultat = entrepriseFormSchema.safeParse({
      ...BASE,
      siteWeb: "javascript:alert(1)",
    });
    expect(resultat.success).toBe(false);
  });

  it("refuse un site web mal formé", () => {
    expect(entrepriseFormSchema.safeParse({ ...BASE, siteWeb: "pas une url" }).success).toBe(
      false,
    );
  });
});
