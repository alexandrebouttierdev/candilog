import { describe, expect, it } from "vitest";
import { etatIa } from "../etatIa";
import type { LlmForm } from "@/shared/types/generated/settings";

function llm(patch: Partial<LlmForm> = {}): LlmForm {
  return {
    provider: "openai",
    api_key_configured: true,
    endpoint: "https://api.openai.com",
    model: "gpt-4o",
    temperature: 0.7,
    mode: "auto",
    ...patch,
  };
}

describe("etatIa", () => {
  it("annonce « Configuré » quand tout est renseigné, avant tout test", () => {
    expect(etatIa(llm(), "idle")).toMatchObject({ label: "Configuré", tone: "accent" });
  });

  it("réclame la clé API manquante d'un fournisseur distant", () => {
    const etat = etatIa(llm({ api_key_configured: false }), "idle");

    expect(etat.label).toBe("Non configuré");
    expect(etat.hint).toBe("Renseignez la clé API pour utiliser l'assistance.");
  });

  it("n'exige aucune clé pour Ollama, qui tourne en local", () => {
    expect(
      etatIa(llm({ provider: "ollama", api_key_configured: false }), "idle").label,
    ).toBe("Configuré");
  });

  it("réclame l'endpoint d'un fournisseur personnalisé", () => {
    const etat = etatIa(
      llm({ provider: { custom: "custom" }, api_key_configured: false, endpoint: "" }),
      "idle",
    );

    expect(etat.label).toBe("Non configuré");
    expect(etat.hint).toBe("Renseignez l'endpoint pour utiliser l'assistance.");
  });

  it("cumule les champs manquants dans un seul message", () => {
    const etat = etatIa(llm({ model: "  ", api_key_configured: false }), "idle");

    expect(etat.hint).toBe("Renseignez le modèle et la clé API pour utiliser l'assistance.");
  });

  it("laisse le résultat du test primer sur la configuration", () => {
    expect(etatIa(llm(), "pending").label).toBe("Connexion en cours");
    expect(etatIa(llm(), "ok")).toMatchObject({ label: "Disponible", tone: "success" });
    expect(etatIa(llm({ api_key_configured: false }), "error")).toMatchObject({
      label: "Erreur",
      tone: "danger",
    });
  });
});
