import { describe, expect, it } from "vitest";
import { letterDateLine, letterHeadline, letterSignature } from "../letterLayout";

describe("composition d'une lettre", () => {
  it("compose l'intitulé de candidature sans préfixe Objet", () => {
    expect(letterHeadline("Développeur")).toBe("Candidature au poste de Développeur");
    expect(letterHeadline("")).toBeNull();
    expect(letterHeadline(null)).toBeNull();
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
