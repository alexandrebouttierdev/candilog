import { describe, expect, it } from "vitest";
import type { Settings } from "@/shared/types/generated/settings";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { assignmentOf, mainLabel, remoteSendToConfirm, taskDestination, tasksSentTo } from "../taskRouting";

function settings(overrides: Partial<Settings> = {}): Settings {
  return {
    llm: { provider: "candilog_local", api_key_configured: false, endpoint: null, model: "", temperature: 0.7, mode: "auto" },
    llm_presets: {},
    ai_routes: {},
    remote_send_consents: [],
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
      label: "Anthropic · claude-sonnet",
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

describe("destination d'une tâche et premier envoi distant", () => {
  const claude = { provider: "claude" as const, model: "claude-sonnet-5" };
  const preset = (endpoint: string) => ({ endpoint, model: "llama3.2", temperature: 0.7, mode: "auto" as const, api_key_configured: false });

  it("nomme qui reçoit les données et demande une seule fois par fournisseur", () => {
    const routed = settings({ ai_routes: { analyze_resume: claude } });

    expect(taskDestination("analyze_resume", routed)).toEqual({ providerId: "claude", recipient: "Anthropic", remote: true });
    expect(remoteSendToConfirm("analyze_resume", routed)?.recipient).toBe("Anthropic");
    expect(remoteSendToConfirm("analyze_resume", { ...routed, remote_send_consents: ["claude"] })).toBeNull();
    // La génération reste sur l'IA locale : rien à demander.
    expect(remoteSendToConfirm("generate_resume", routed)).toBeNull();
    expect(tasksSentTo("claude", routed)).toBe(1);
  });

  it("traite un Ollama d'une autre machine comme un envoi distant, et une tâche désactivée comme rien", () => {
    const lan = settings({
      ai_routes: { write_letter: { provider: "ollama", model: "llama3.2" }, extract_offer: null },
      llm_presets: { ollama: preset("http://192.168.1.20:11434") },
    });

    expect(taskDestination("write_letter", lan)).toEqual({ providerId: "ollama", recipient: "192.168.1.20", remote: true });
    expect(assignmentOf("write_letter", lan, []).locality).toBe("remote");
    expect(taskDestination("extract_offer", lan)).toBeNull();
    const local = settings({ ai_routes: { write_letter: { provider: "ollama", model: "llama3.2" } } });
    expect(taskDestination("write_letter", local)?.remote).toBe(false);
  });
});
