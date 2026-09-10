import { describe, expect, it } from "vitest";
import { etatIa, iaEstConfiguree, manquants } from "../etatIa";
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

describe("iaEstConfiguree", () => {
  it("accepte une configuration cloud complète", () => {
    expect(iaEstConfiguree(llm())).toBe(true);
  });

  it("refuse un modèle vide", () => {
    expect(iaEstConfiguree(llm({ model: "  " }))).toBe(false);
  });

  it("refuse une clé API manquante pour un fournisseur distant", () => {
    expect(iaEstConfiguree(llm({ api_key_configured: false }))).toBe(false);
  });

  // La grille des fournisseurs vide `model` en sélectionnant Mistral Local, et l'écran
  // affiche MistralLocalPanel au lieu d'un champ « Modèle » : ce champ ne peut donc jamais
  // être rempli. L'exiger affichait AiRequiredModal après un enregistrement pourtant réussi.
  it("accepte Mistral Local sans nom de modèle, qu'il ne stocke pas dans llm.model", () => {
    expect(
      iaEstConfiguree(
        llm({ provider: "mistral_local", model: "", endpoint: null, api_key_configured: false }),
      ),
    ).toBe(true);
  });

  it("accepte l'IA locale Candilog sans nom de modèle dans llm.model", () => {
    expect(
      iaEstConfiguree(
        llm({ provider: "candilog_local", model: "", endpoint: null, api_key_configured: false }),
      ),
    ).toBe(true);
  });
});

describe("manquants", () => {
  it("ne réclame aucun champ pour Mistral Local", () => {
    expect(
      manquants(
        llm({ provider: "mistral_local", model: "", endpoint: null, api_key_configured: false }),
      ),
    ).toEqual([]);
  });

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
