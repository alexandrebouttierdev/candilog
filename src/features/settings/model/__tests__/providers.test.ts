import { describe, expect, it } from "vitest";
import { FOURNISSEURS, idProvider, modelDefaut, versProvider } from "../providers";

describe("fournisseurs IA", () => {
  it("reconnaît le variant personnalisé sérialisé par serde", () => {
    expect(idProvider({ custom: "maison" })).toBe("custom");
    expect(idProvider("openai")).toBe("openai");
    expect(versProvider("custom")).toEqual({ custom: "custom" });
  });

  it("ne préremplit aucun modèle par défaut", () => {
    for (const fournisseur of FOURNISSEURS) {
      expect(modelDefaut(fournisseur.id)).toBe("");
    }
  });
});
