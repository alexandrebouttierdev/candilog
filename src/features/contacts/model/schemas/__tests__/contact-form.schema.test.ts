import { describe, expect, it } from "vitest";
import { contactFormSchema } from "../contact-form.schema";

const BASE = {
  prenom: "Camille",
  nom: "Rivet",
  email: "",
  telephone: "",
  entrepriseId: "",
  poste: "",
  roleSuivi: "",
  linkedin: "",
  notes: "",
};

describe("schéma du formulaire contact", () => {
  it("accepte prénom et nom seuls", () => {
    expect(contactFormSchema.safeParse(BASE).success).toBe(true);
  });

  it("exige le prénom et le nom", () => {
    expect(contactFormSchema.safeParse({ ...BASE, prenom: "" }).success).toBe(false);
    expect(contactFormSchema.safeParse({ ...BASE, nom: "  " }).success).toBe(false);
  });

  it("accepte une adresse e-mail valide", () => {
    expect(
      contactFormSchema.safeParse({ ...BASE, email: "camille.rivet@novadigital.fr" }).success,
    ).toBe(true);
  });

  it("refuse une adresse e-mail mal formée", () => {
    expect(contactFormSchema.safeParse({ ...BASE, email: "camille@" }).success).toBe(false);
  });

  it("laisse l'e-mail facultatif", () => {
    // Un contact peut n'être connu que par téléphone : imposer l'e-mail empêcherait de
    // l'enregistrer du tout.
    expect(contactFormSchema.parse(BASE).email).toBeNull();
  });

  it("refuse un profil LinkedIn dont le schéma n'est pas HTTP", () => {
    expect(
      contactFormSchema.safeParse({ ...BASE, linkedin: "javascript:alert(1)" }).success,
    ).toBe(false);
  });

  it("normalise l'entreprise non choisie en null", () => {
    // Le `select` renvoie "" pour « Aucune » ; le backend attend une absence de valeur, pas
    // une chaîne vide qui échouerait la contrainte de clé étrangère.
    expect(contactFormSchema.parse(BASE).entrepriseId).toBeNull();
  });

  it("conserve le rôle dans le suivi lorsqu'il est choisi", () => {
    expect(contactFormSchema.parse({ ...BASE, roleSuivi: "Recruteur" }).roleSuivi).toBe(
      "Recruteur",
    );
  });
});
