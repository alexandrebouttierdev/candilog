import { describe, expect, it } from "vitest";
import type { Settings } from "@/shared/types/generated/settings";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { assignmentOf, mainLabel } from "../taskRouting";

function settings(overrides: Partial<Settings> = {}): Settings {
  return {
    llm: { provider: "candilog_local", api_key_configured: false, endpoint: null, model: "", temperature: 0.7, mode: "auto" },
    llm_presets: {},
    ai_routes: {},
    theme: "system",
    language: "fr",
    ...overrides,
  };
}

const MINISTRAL = {
  definition: { ollama_tag: "ministral-3:3b", display_name: "Ministral 3 · 3B" },
  installed: true,
  active: true,
} as unknown as ManagedModelStatus;

describe("routage des tâches", () => {
  it("fait suivre le fournisseur principal à une tâche sans route", () => {
    expect(assignmentOf("write_letter", settings(), [MINISTRAL])).toEqual({
      label: "Ministral 3 · 3B",
      locality: "local",
      isDefault: true,
    });
  });

  it("nomme la route distante et la marque comme envoi distant", () => {
    const routed = settings({ ai_routes: { analyze_resume: { provider: "claude", model: "claude-sonnet" } } });
    expect(assignmentOf("analyze_resume", routed, [MINISTRAL])).toEqual({
      label: "Claude · claude-sonnet",
      locality: "remote",
      isDefault: false,
    });
  });

  it("distingue une tâche désactivée d'une tâche sans route", () => {
    const off = settings({ ai_routes: { extract_offer: null } });
    expect(assignmentOf("extract_offer", off, []).locality).toBe("off");
    expect(assignmentOf("import_resume", off, []).isDefault).toBe(true);
  });

  it("nomme un modèle local routé par son libellé du catalogue", () => {
    const routed = settings({ ai_routes: { generate_resume: { provider: "candilog_local", model: "ministral-3:3b" } } });
    expect(assignmentOf("generate_resume", routed, [MINISTRAL]).label).toBe("Ministral 3 · 3B");
  });

  it("décrit un fournisseur principal distant avec son modèle", () => {
    expect(mainLabel(settings({ llm: { ...settings().llm, provider: "openai", model: "gpt-4o" } }), [])).toBe("OpenAI · gpt-4o");
  });
});
