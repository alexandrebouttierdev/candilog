import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import type { PropsWithChildren } from "react";
import { AiProviderRailWidget } from "../AiProviderRailWidget";
import { settingsService } from "@/features/settings/services/settingsService";
import { systemResourceService } from "@/features/settings/services/systemResourceService";
import { useAiOperationStore } from "@/features/ai/viewmodel/ai-operation-store";
import { useAiRailStatusStore } from "@/features/ai/viewmodel/ai-rail-status-store";
import type { Settings } from "@/shared/types/generated/settings";

vi.mock("@/features/settings/services/settingsService", () => ({
  settingsService: { load: vi.fn() },
}));

vi.mock("@/features/settings/services/systemResourceService", () => ({
  systemResourceService: { snapshot: vi.fn() },
  SYSTEM_RESOURCES_KEY: ["system-resources"],
}));

vi.mock("@/features/settings/services/localAiService", () => ({
  localAiService: {
    recommendation: vi.fn(),
    status: vi.fn(),
  },
}));

function llmSettings(partial: Partial<Settings["llm"]> = {}): Settings {
  return {
    theme: "system",
    language: "fr",
    llm: {
      provider: "ollama",
      model: "llama3",
      endpoint: "http://127.0.0.1:11434",
      api_key_configured: false,
      temperature: 0.2,
      mode: "auto",
      ...partial,
    },
  };
}

function Wrapper({ children }: PropsWithChildren) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return (
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={["/"]}>
        <Routes>
          <Route path="/" element={children} />
          <Route path="/settings/ai" element={<div>Réglages IA</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("AiProviderRailWidget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAiOperationStore.setState({ active: null });
    useAiRailStatusStore.setState({
      connectionTest: "idle",
      lastOperationFailed: false,
    });
    vi.mocked(systemResourceService.snapshot).mockResolvedValue({
      cpu_percent: 34,
      ram_used_percent: 23,
      ram_used_mb: 4000,
      ram_total_mb: 16000,
      vram_available: false,
      vram_used_percent: null,
      vram_used_mb: null,
      vram_total_mb: null,
    });
  });

  it("affiche smart_toy et Non configuré sans fournisseur prêt", async () => {
    vi.mocked(settingsService.load).mockResolvedValue(
      llmSettings({ model: "  ", provider: "openai", api_key_configured: false }),
    );
    render(<AiProviderRailWidget />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Non configuré/ })).toBeInTheDocument();
    });
    expect(screen.getByText("smart_toy")).toBeInTheDocument();
  });

  it("signale En cours… quand une opération IA est active", async () => {
    vi.mocked(settingsService.load).mockResolvedValue(llmSettings());
    useAiOperationStore.setState({
      active: {
        id: "1",
        kind: "analyse",
        stopping: false,
        stop: () => Promise.resolve(),
      },
    });
    render(<AiProviderRailWidget />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /En cours/ })).toBeInTheDocument();
    });
  });

  it("navigue vers les réglages IA au clic", async () => {
    vi.mocked(settingsService.load).mockResolvedValue(llmSettings());
    render(<AiProviderRailWidget />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /Ollama · llama3 — Disponible/ }),
      ).toBeInTheDocument();
    });
    await userEvent.click(screen.getByRole("button", { name: /Ollama · llama3 — Disponible/ }));
    expect(screen.getByText("Réglages IA")).toBeInTheDocument();
  });

  it("annonce fournisseur et modèle dans le tooltip", async () => {
    vi.mocked(settingsService.load).mockResolvedValue(
      llmSettings({ provider: "mistral", model: "mistral-small", api_key_configured: true }),
    );
    render(<AiProviderRailWidget />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /Mistral · mistral-small — Disponible/ }),
      ).toBeInTheDocument();
    });
  });

  it("montre un tiret pour la VRAM indisponible", async () => {
    vi.mocked(settingsService.load).mockResolvedValue(llmSettings());
    render(<AiProviderRailWidget />, { wrapper: Wrapper });
    await waitFor(() => {
      expect(screen.getByText("VRAM")).toBeInTheDocument();
    });
    const vram = screen.getByLabelText("VRAM");
    expect(vram).toHaveAttribute("aria-valuetext", "indisponible");
  });
});
