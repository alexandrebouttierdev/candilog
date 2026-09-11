import { describe, expect, it } from "vitest";
import { aiStatus, isAiConfigured, manquants } from "../aiStatus";
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

describe("aiStatus", () => {
  it("annonce « Configuré » quand tout est renseigné, avant tout test", () => {
    expect(aiStatus(llm(), "idle")).toMatchObject({ label: "Configuré", tone: "accent" });
  });

  it("réclame la clé API manquante d'un fournisseur distant", () => {
    const etat = aiStatus(llm({ api_key_configured: false }), "idle");

    expect(etat.label).toBe("Non configuré");
    expect(etat.hint).toBe("Renseignez la clé API pour utiliser l'assistance.");
  });

  it("n'exige aucune clé pour Ollama, qui tourne en local", () => {
    expect(
      aiStatus(llm({ provider: "ollama", api_key_configured: false }), "idle").label,
    ).toBe("Configuré");
  });

  it("réclame l'endpoint d'un fournisseur personnalisé", () => {
    const etat = aiStatus(
      llm({ provider: { custom: "custom" }, api_key_configured: false, endpoint: "" }),
      "idle",
    );

    expect(etat.label).toBe("Non configuré");
    expect(etat.hint).toBe("Renseignez l'endpoint pour utiliser l'assistance.");
  });

  it("cumule les champs manquants dans un seul message", () => {
    const etat = aiStatus(llm({ model: "  ", api_key_configured: false }), "idle");

    expect(etat.hint).toBe("Renseignez le modèle et la clé API pour utiliser l'assistance.");
  });

  it("laisse le résultat du test primer sur la configuration", () => {
    expect(aiStatus(llm(), "pending").label).toBe("Connexion en cours");
    expect(aiStatus(llm(), "ok")).toMatchObject({ label: "Disponible", tone: "success" });
    expect(aiStatus(llm({ api_key_configured: false }), "error")).toMatchObject({
      label: "Erreur",
      tone: "danger",
    });
  });
});

describe("isAiConfigured", () => {
  it("accepte une configuration cloud complète", () => {
    expect(isAiConfigured(llm())).toBe(true);
  });

  it("refuse un modèle vide", () => {
    expect(isAiConfigured(llm({ model: "  " }))).toBe(false);
  });

  it("refuse une clé API manquante pour un fournisseur distant", () => {
    expect(isAiConfigured(llm({ api_key_configured: false }))).toBe(false);
  });

  it("accepte l'IA locale Candilog sans nom de modèle dans llm.model", () => {
    expect(
      isAiConfigured(
        llm({ provider: "candilog_local", model: "", endpoint: null, api_key_configured: false }),
      ),
    ).toBe(true);
  });
});

describe("manquants", () => {
  it("ne réclame aucun champ pour l'IA locale Candilog", () => {
    expect(
      manquants(
        llm({ provider: "candilog_local", model: "", endpoint: null, api_key_configured: false }),
      ),
    ).toEqual([]);
  });

  it("réclame toujours le modèle des autres fournisseurs locaux", () => {
    expect(manquants(llm({ provider: "ollama", model: "", api_key_configured: false }))).toEqual([
      "le modèle",
    ]);
  });
});
