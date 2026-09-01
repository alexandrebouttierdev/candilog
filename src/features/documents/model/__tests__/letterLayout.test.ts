import { describe, expect, it } from "vitest";
import {
  elider,
  letterDateLine,
  letterHeadline,
  letterJobTitleFromHeadline,
  letterSignature,
} from "../letterLayout";

describe("composition d'une lettre", () => {
  it("compose l'intitulé de candidature sans préfixe Objet", () => {
    expect(letterHeadline("Développeur")).toBe("Candidature au poste de Développeur");
    expect(letterHeadline("")).toBeNull();
    expect(letterHeadline(null)).toBeNull();
  });

  /** « Candidature au poste de Administrateur » : la feuille écrivait la faute en gras. */
  it("élide la préposition devant une voyelle", () => {
    expect(letterHeadline("Administrateur système")).toBe(
      "Candidature au poste d’Administrateur système",
    );
    expect(letterHeadline("Ingénieure réseaux")).toBe("Candidature au poste d’Ingénieure réseaux");
    expect(letterHeadline("Épicière")).toBe("Candidature au poste d’Épicière");
  });

  it("retire l'amorce de l'intitulé saisi, élidée ou non", () => {
    expect(letterJobTitleFromHeadline("Candidature au poste de Technicien")).toBe("Technicien");
    expect(letterJobTitleFromHeadline("Candidature au poste d’Administrateur")).toBe(
      "Administrateur",
    );
    expect(letterJobTitleFromHeadline("Technicien")).toBe("Technicien");
  });

  it("n'élide pas devant une consonne", () => {
    expect(elider("de", "Technicien")).toBe("de Technicien");
    expect(elider("de", "Astek")).toBe("d’Astek");
    expect(elider("de", "  Hôtellerie  ")).toBe("d’Hôtellerie");
  });

  it("date la lettre en français, avec la ville quand elle est connue", () => {
    const jour = new Date(2026, 7, 31);

    expect(letterDateLine("Rennes", jour)).toBe("Rennes, le 31 août 2026");
    expect(letterDateLine(null, jour)).toBe("Le 31 août 2026");
  });

  it("retombe sur Candilog quand le profil n'a pas de nom", () => {
    expect(letterSignature("Alex", "Exemple")).toBe("Alex Exemple");
    expect(letterSignature("", "  ")).toBe("Candilog");
  });
});
