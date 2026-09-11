import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useUiStore } from "@/shared/lib/ui-store";
import { settingsService } from "../../services/settingsService";
import type { Settings } from "@/shared/types/generated/settings";
import { useAboutViewModel } from "../useAboutViewModel";
import { useThemePreference } from "../useSettingsViewModel";

function reglages(theme: Settings["theme"] = "system"): Settings {
  return {
    llm: {
      provider: "openai",
      api_key_configured: true,
      endpoint: "https://api.openai.com",
      model: "gpt-4o",
      temperature: 0.7,
      mode: "auto",
    },
    theme,
    language: "fr",
  };
}

function setup() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return { wrapper, client };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ theme: "system" });
});

describe("useAboutViewModel", () => {
  it("expose la version produit", async () => {
    vi.spyOn(settingsService, "about").mockResolvedValue({ version: "0.4.0", name: "Candilog" });
    const { wrapper } = setup();
    const { result } = renderHook(() => useAboutViewModel(), { wrapper });

    await waitFor(() => expect(result.current.version).toBe("0.4.0"));
  });
});

describe("useThemePreference", () => {
  it("applique et persiste le thème sans toast", async () => {
    const initial = reglages("light");
    vi.spyOn(settingsService, "load").mockResolvedValue(initial);
    const save = vi.spyOn(settingsService, "save").mockResolvedValue(reglages("dark"));
    const { wrapper } = setup();
    const { result } = renderHook(() => useThemePreference(), { wrapper });

    await act(async () => {
      await result.current.saveTheme("dark");
    });

    expect(useUiStore.getState().theme).toBe("dark");
    expect(save).toHaveBeenCalledWith({ ...initial, theme: "dark" }, null);
  });
});
