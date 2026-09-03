import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
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

    expect(await screen.findByText("Modèle local : aucune clé, aucune connexion")).toBeInTheDocument();
    expect(screen.queryByLabelText(/^Clé API/)).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "canirun.ai" }));
    expect(openExternal).toHaveBeenCalledWith("https://www.canirun.ai/");
  });

  it("annonce l'état du fournisseur sans attendre un test", async () => {
    render(<AiPage />, { wrapper });

    expect(await screen.findByText("Configuré")).toBeInTheDocument();
    expect(screen.getByText("gpt-4o")).toBeInTheDocument();
    expect(screen.getByText("Fournisseur")).toBeInTheDocument();
    expect(screen.getByText("Configuration")).toBeInTheDocument();
    expect(screen.getByText("Apparence")).toBeInTheDocument();
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

  it("signale un fournisseur incomplet et ce qu'il manque", async () => {
    vi.spyOn(settingsService, "load").mockResolvedValue(
      reglages({ api_key_configured: false }),
    );

    render(<AiPage />, { wrapper });

    expect(await screen.findByText("Non configuré")).toBeInTheDocument();
    expect(
      screen.getByText("Renseignez la clé API pour utiliser l'assistance."),
    ).toBeInTheDocument();
  });

  it("passe à « Disponible » quand le test aboutit", async () => {
    vi.spyOn(settingsService, "testConnection").mockResolvedValue(undefined);

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Tester la connexion" }));

    expect(await screen.findByText("Disponible")).toBeInTheDocument();
  });

  it("affiche l'erreur du test sans envahir l'écran", async () => {
    vi.spyOn(settingsService, "testConnection").mockRejectedValue(
      new AppError({ code: "PROVIDER_ERROR", message: "Clé refusée par le fournisseur." }),
    );

    render(<AiPage />, { wrapper });
    await userEvent.click(await screen.findByRole("button", { name: "Tester la connexion" }));

    expect(await screen.findByText("Erreur")).toBeInTheDocument();
    expect(screen.getByText("Clé refusée par le fournisseur.")).toBeInTheDocument();
  });

  it("n'affiche jamais la clé API en clair", async () => {
    render(<AiPage />, { wrapper });

    const champ = await screen.findByLabelText(/^Clé API/);
    expect(champ).toHaveAttribute("type", "password");
    expect(champ).toHaveValue("");
    await waitFor(() =>
      expect(screen.getByPlaceholderText("Clé configurée")).toBeInTheDocument(),
    );
  });
});
