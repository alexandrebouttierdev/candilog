import { describe, expect, it } from "vitest";
import { letterDateLine, letterSignature, letterSubject } from "../letterLayout";

describe("composition d'une lettre", () => {
  it("compose l'objet comme l'export PDF", () => {
    expect(letterSubject("Développeur", "Astek")).toBe(
      "Objet : candidature au poste de Développeur — Astek",
    );
    expect(letterSubject("Développeur", "")).toBe("Objet : candidature au poste de Développeur");
    expect(letterSubject(null, "Astek")).toBe("Objet : candidature");
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
