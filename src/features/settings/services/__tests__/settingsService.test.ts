import { beforeEach, describe, expect, it, vi } from "vitest";
import { ipc } from "@/shared/services/ipc";
import { settingsService, type Settings } from "../settingsService";

vi.mock("@/shared/services/ipc", () => ({ ipc: vi.fn() }));

const settings: Settings = {
  llm: {
    provider: "openai",
    api_key_configured: false,
    endpoint: "https://api.openai.com",
    model: "gpt-4o",
    temperature: 0.7,
    mode: "auto",
  },
  theme: "system",
  language: "fr",
};

describe("settingsService", () => {
  beforeEach(() => vi.mocked(ipc).mockReset());

  it("envoie une nouvelle clé uniquement comme argument entrant", async () => {
    vi.mocked(ipc).mockResolvedValue({
      ...settings,
      llm: { ...settings.llm, api_key_configured: true },
    });

    await settingsService.save(settings, "sk-draft");

    expect(ipc).toHaveBeenCalledWith("settings_save", {
      settings,
      api_key: "sk-draft",
    });
  });

  it("expose une commande dédiée pour supprimer la clé", async () => {
    vi.mocked(ipc).mockResolvedValue(undefined);

    await settingsService.clearApiKey();

    expect(ipc).toHaveBeenCalledWith("settings_clear_api_key");
  });

  it("retourne le résultat détaillé de la réinitialisation", async () => {
    const outcome = { data_cleared: true, secret_cleared: false };
    vi.mocked(ipc).mockResolvedValue(outcome);

    await expect(settingsService.reset()).resolves.toEqual(outcome);

    expect(ipc).toHaveBeenCalledWith("settings_reset");
  });
});
