import { describe, expect, it } from "vitest";
import {
  PROVIDERS,
  idProvider,
  defaultModel,
  llmFromPreset,
  presetFromLlm,
  toProvider,
} from "../providers";

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

  it("restaure un preset fournisseur sans écraser les autres", () => {
    const openai = llmFromPreset("openai", {
      endpoint: "https://api.openai.com",
      model: "gpt-4o",
      temperature: 0.2,
      mode: "advanced",
      api_key_configured: true,
    });
    const mistral = llmFromPreset("mistral", {
      endpoint: "https://api.mistral.ai",
      model: "mistral-small",
      temperature: 0.9,
      mode: "auto",
      api_key_configured: true,
    });
    expect(openai.model).toBe("gpt-4o");
    expect(mistral.model).toBe("mistral-small");
    expect(presetFromLlm(openai).model).toBe("gpt-4o");
    expect(llmFromPreset("claude", undefined).model).toBe("");
    expect(llmFromPreset("claude", undefined).endpoint).toBe("https://api.anthropic.com");
  });
});
