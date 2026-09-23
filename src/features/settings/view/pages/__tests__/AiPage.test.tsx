import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { AiPage } from "../AiPage";
import { settingsService } from "../../../services/settingsService";
import type { LlmForm, Settings } from "@/shared/types/generated/settings";
import { AppError } from "@/shared/types/app-error";
import { openExternal } from "@/shared/services/external-link";

vi.mock("@/shared/services/external-link", () => ({ openExternal: vi.fn() }));

vi.mock("../../../viewmodel/useManagedOllamaViewModel", () => ({
  useManagedOllamaViewModel: () => ({
    runtimeState: "not_installed",
    status: {
      runtime_state: "not_installed",
      runtime_version: null,
      port: null,
      models_disk_bytes: 0,
      active_model: null,
      models: [],
      last_error: null,
    },
    progress: null,
    error: null,
    isInstalling: false,
    isRemoving: false,
    isActivating: false,
    install: vi.fn(),
    cancel: vi.fn(),
    remove: vi.fn(),
    activate: vi.fn(),
    activateAsync: vi.fn().mockResolvedValue(undefined),
    reload: vi.fn(),
  }),
}));

function reglages(llm: Partial<LlmForm> = {}): Settings {
  return {
    llm: {
      provider: "openai",
      api_key_configured: true,
      endpoint: "https://api.openai.com",
      model: "gpt-4o",
      temperature: 0.7,
      mode: "auto",
      ...llm,
    },
    llm_presets: {},
    ai_routes: {},
    theme: "system",
    language: "fr",
  };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return (
    <QueryClientProvider client={client}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(settingsService, "load").mockResolvedValue(reglages());
});

describe("écran Intelligence artificielle", () => {
  it("propose l'aide canirun.ai pour Ollama, sans clé API à saisir", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(reglages({ provider: "ollama", model: "llama3.2:3b" }));

    render(<AiPage />, { wrapper });

    // Ollama est le fournisseur principal : l'écran s'ouvre sur lui.
    expect(await screen.findByRole("tab", { name: /^Ollama/ })).toHaveAttribute("aria-selected", "true");
    expect(await screen.findByText("Modèle local : aucune clé, aucune connexion")).toBeInTheDocument();
    expect(screen.queryByLabelText(/^Clé API/)).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "canirun.ai" }));
    expect(openExternal).toHaveBeenCalledWith("https://www.canirun.ai/");
  });

  it("enregistre le nouveau modèle Ollama choisi", async () => {
    const initial = reglages({
      provider: "ollama",
      api_key_configured: false,
      endpoint: "http://localhost:11434",
      model: "LiquidAI/lfm2.5-1.2b-instruct:latest",
    });
    vi.spyOn(settingsService, "load").mockResolvedValue(initial);
    const listModels = vi.spyOn(settingsService, "listModels").mockResolvedValue([
      "LiquidAI/lfm2.5-1.2b-instruct:latest",
      "maternion/lfm2.5:350m",
    ]);
    const save = vi.spyOn(settingsService, "save").mockResolvedValue(initial);

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));
    await userEvent.click(await screen.findByRole("button", { name: "Actualiser" }));
    await waitFor(() => expect(listModels).toHaveBeenCalledOnce());
    await userEvent.click(await screen.findByRole("radio", { name: "maternion/lfm2.5:350m" }));
    await userEvent.click(screen.getByRole("button", { name: "Enregistrer" }));

    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    expect(save.mock.calls[0]?.[0].llm.model).toBe("maternion/lfm2.5:350m");
    expect(save.mock.calls[0]?.[1]).toBeNull();
  });

  it("annonce les sections fournisseur sans bandeau d'état en tête", async () => {
    render(<AiPage />, { wrapper });

    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));
    expect(screen.getByRole("navigation", { name: "Fournisseurs" })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Configuration" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tester la connexion" })).toBeInTheDocument();
    expect(screen.getByDisplayValue("gpt-4o")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /IA locale/ })).toBeInTheDocument();
    // Un fournisseur distant le dit : les données lui sont envoyées.
    expect(screen.getByText("Envoyé à OpenAI")).toBeInTheDocument();
  });

  it("ouvre l'onglet IA locale sans bandeau d'état en tête", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(
      reglages({ provider: "candilog_local", model: "", api_key_configured: false, endpoint: "" }),
    );

    render(<AiPage />, { wrapper });

    expect(await screen.findByRole("tab", { name: /IA locale/ })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Tester l'IA" })).not.toBeInTheDocument();
    expect(screen.queryByText("Non configuré")).not.toBeInTheDocument();
  });

  it("affiche un squelette pendant le chargement des réglages", () => {
    vi.spyOn(settingsService, "load").mockReturnValue(new Promise(() => undefined));

    render(<AiPage />, { wrapper });

    expect(
      screen.getByRole("status", { name: "Chargement des réglages" }),
    ).toBeInTheDocument();
  });

  it("remplace le squelette par une erreur quand le chargement initial échoue", async () => {
    vi.spyOn(settingsService, "load").mockRejectedValue(
      new AppError({ code: "DATABASE_ERROR", message: "Réglages inaccessibles." }),
    );

    render(<AiPage />, { wrapper });

    expect(await screen.findByText("Réglages inaccessibles.")).toBeInTheDocument();
    expect(
      screen.queryByRole("status", { name: "Chargement des réglages" }),
    ).not.toBeInTheDocument();
  });

  it("montre le champ clé API à renseigner quand aucune clé n'est configurée", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(
      reglages({ api_key_configured: false }),
    );

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));

    expect(await screen.findByLabelText(/^Clé API/)).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Saisir la clé API")).toBeInTheDocument();
  });

  it("confirme la connexion sous Configuration quand le test aboutit", async () => {
    vi.spyOn(settingsService, "testConnection").mockResolvedValue(undefined);

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));
    await userEvent.click(await screen.findByRole("button", { name: "Tester la connexion" }));

    expect(await screen.findByText("Connexion établie.")).toBeInTheDocument();
  });

  it("affiche l'erreur du test sous Configuration", async () => {
    vi.spyOn(settingsService, "testConnection").mockRejectedValue(
      new AppError({ code: "PROVIDER_ERROR", message: "Clé refusée par le fournisseur." }),
    );

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));
    await userEvent.click(await screen.findByRole("button", { name: "Tester la connexion" }));

    expect(await screen.findByText("Clé refusée par le fournisseur.")).toBeInTheDocument();
  });

  it("n'affiche jamais la clé API en clair", async () => {
    render(<AiPage />, { wrapper });

    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));
    const champ = await screen.findByLabelText(/^Clé API/);
    expect(champ).toHaveAttribute("type", "password");
    expect(champ).toHaveValue("");
    await waitFor(() =>
      expect(screen.getByPlaceholderText("Clé configurée")).toBeInTheDocument(),
    );
  });

  it("conserve la config d'un fournisseur quand on en sélectionne un autre", async () => {
    const initial = reglages({
      provider: "openai",
      model: "gpt-4o-mini",
      endpoint: "https://api.openai.com",
      api_key_configured: true,
    });
    initial.llm_presets = {
      openai: {
        endpoint: "https://api.openai.com",
        model: "gpt-4o-mini",
        temperature: 0.7,
        mode: "auto",
        api_key_configured: true,
      },
      mistral: {
        endpoint: "https://api.mistral.ai",
        model: "mistral-small-latest",
        temperature: 0.4,
        mode: "standard",
        api_key_configured: true,
      },
    };
    vi.spyOn(settingsService, "load").mockResolvedValue(initial);

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("tab", { name: /^OpenAI/ }));

    expect(await screen.findByDisplayValue("gpt-4o-mini")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("tab", { name: /^Mistral/ }));
    expect(await screen.findByDisplayValue("mistral-small-latest")).toBeInTheDocument();
    expect(screen.getByDisplayValue("https://api.mistral.ai")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("tab", { name: /^OpenAI/ }));
    expect(await screen.findByDisplayValue("gpt-4o-mini")).toBeInTheDocument();
    expect(screen.getByDisplayValue("https://api.openai.com")).toBeInTheDocument();
  });
});

describe("écran Intelligence artificielle — qui fait quoi", () => {
  it("montre les cinq tâches, qui suivent le fournisseur principal par défaut", async () => {
    render(<AiPage />, { wrapper });

    const section = await screen.findByRole("region", { name: "Qui fait quoi" });
    for (const tache of ["Générer un CV ciblé", "Rédiger une lettre", "Analyser un CV", "Extraire une offre d'emploi", "Lire un CV importé"]) {
      expect(within(section).getByRole("button", { name: new RegExp(`^${tache}, modèle OpenAI · gpt-4o, distant`) })).toBeInTheDocument();
    }
  });

  it("désactive une tâche et enregistre aussitôt le routage", async () => {
    const save = vi.spyOn(settingsService, "save").mockImplementation((settings) => Promise.resolve(settings));
    render(<AiPage />, { wrapper });

    const section = await screen.findByRole("region", { name: "Qui fait quoi" });
    await userEvent.click(within(section).getByRole("button", { name: /^Extraire une offre d'emploi/ }));
    await userEvent.click(await screen.findByRole("menuitemradio", { name: "Aucun — désactiver cette tâche" }));

    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    expect(save.mock.calls[0]?.[0].ai_routes).toEqual({ extract_offer: null });
    // Le fournisseur principal n'est pas touché par un choix de routage.
    expect(save.mock.calls[0]?.[0].llm.provider).toBe("openai");
  });

  it("route une tâche vers un fournisseur distant configuré", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue({
      ...reglages(),
      llm_presets: {
        mistral: { endpoint: null, model: "mistral-small-latest", temperature: 0.4, mode: "auto", api_key_configured: true },
      },
    });
    const save = vi.spyOn(settingsService, "save").mockImplementation((settings) => Promise.resolve(settings));
    render(<AiPage />, { wrapper });

    const section = await screen.findByRole("region", { name: "Qui fait quoi" });
    await userEvent.click(within(section).getByRole("button", { name: /^Analyser un CV/ }));
    await userEvent.click(await screen.findByRole("menuitemradio", { name: "Mistral · mistral-small-latest" }));

    await waitFor(() =>
      expect(save.mock.calls[0]?.[0].ai_routes).toEqual({
        analyze_resume: { provider: "mistral", model: "mistral-small-latest" },
      }),
    );
  });
});
