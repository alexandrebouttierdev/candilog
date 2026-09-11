import { describe, expect, it } from "vitest";
import { PROVIDERS, idProvider, defaultModel, toProvider } from "../providers";

describe("fournisseurs IA", () => {
  it("reconnaît le variant personnalisé sérialisé par serde", () => {
    expect(idProvider({ custom: "maison" })).toBe("custom");
    expect(idProvider("openai")).toBe("openai");
    expect(toProvider("custom")).toEqual({ custom: "custom" });
  });

  it("ne préremplit aucun modèle par défaut", () => {
    for (const fournisseur of PROVIDERS) {
      expect(defaultModel(fournisseur.id)).toBe("");
    }
  });
});
