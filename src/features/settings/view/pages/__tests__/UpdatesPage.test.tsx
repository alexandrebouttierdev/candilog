import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { UpdatesPage } from "../UpdatesPage";
import { settingsService } from "../../../services/settingsService";
import { AppError } from "@/shared/types/app-error";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(settingsService, "about").mockResolvedValue({ version: "1.2.0", name: "Candilog" });
});

describe("écran Mise à jour", () => {
  it("ne contient plus le bloc « Installation maîtrisée »", async () => {
    render(<UpdatesPage />, { wrapper });

    await waitFor(() => expect(screen.getByText("1.2.0")).toBeInTheDocument());
    expect(screen.queryByText("Installation maîtrisée")).not.toBeInTheDocument();
    expect(screen.queryByText("Aucune installation silencieuse")).not.toBeInTheDocument();
  });

  it("présente la version installée et l'action de vérification", async () => {
    render(<UpdatesPage />, { wrapper });

    expect(await screen.findByText("Version installée")).toBeInTheDocument();
    expect(await screen.findByText("1.2.0")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rechercher une mise à jour" })).toBeInTheDocument();
  });

  it("annonce que l'application est à jour", async () => {
    vi.spyOn(settingsService, "checkUpdate").mockResolvedValue(null);

    render(<UpdatesPage />, { wrapper });
    await userEvent.click(
      await screen.findByRole("button", { name: "Rechercher une mise à jour" }),
    );

    expect(await screen.findByText("Candilog est à jour")).toBeInTheDocument();
    expect(
      screen.getByText("Vous utilisez la dernière version disponible."),
    ).toBeInTheDocument();
  });

  it("présente la nouvelle version et l'action de mise à jour", async () => {
    vi.spyOn(settingsService, "checkUpdate").mockResolvedValue({
      version: "1.3.0",
      notes: "Corrections diverses.",
      page_url: "https://github.com/alexandrebouttierdev/candilog/releases/tag/v1.3.0",
      asset: { name: "candilog.AppImage", url: "https://example.test/candilog.AppImage" },
    });

    render(<UpdatesPage />, { wrapper });
    await userEvent.click(
      await screen.findByRole("button", { name: "Rechercher une mise à jour" }),
    );

    expect(await screen.findByText("Une nouvelle version est disponible")).toBeInTheDocument();
    expect(screen.getByText("Nouvelle version")).toBeInTheDocument();
    expect(screen.getByText("1.3.0")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Mettre à jour" })).toBeInTheDocument();
  });

  it("lance le téléchargement et l'annonce", async () => {
    vi.spyOn(settingsService, "checkUpdate").mockResolvedValue({
      version: "1.3.0",
      notes: "",
      page_url: "https://github.com/alexandrebouttierdev/candilog/releases/tag/v1.3.0",
      asset: { name: "candilog.AppImage", url: "https://example.test/candilog.AppImage" },
    });
    const telecharger = vi
      .spyOn(settingsService, "downloadUpdate")
      .mockReturnValue(new Promise(() => undefined));

    render(<UpdatesPage />, { wrapper });
    await userEvent.click(
      await screen.findByRole("button", { name: "Rechercher une mise à jour" }),
    );
    await userEvent.click(await screen.findByRole("button", { name: "Mettre à jour" }));

    await waitFor(() => expect(telecharger).toHaveBeenCalledTimes(1));
    expect(await screen.findByText(/Téléchargement en cours/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Mettre à jour" })).toBeDisabled();
  });

  it("affiche l'échec de la vérification sans masquer la version installée", async () => {
    vi.spyOn(settingsService, "checkUpdate").mockRejectedValue(
      new AppError({ code: "HTTP_ERROR", message: "Serveur injoignable." }),
    );

    render(<UpdatesPage />, { wrapper });
    await userEvent.click(
      await screen.findByRole("button", { name: "Rechercher une mise à jour" }),
    );

    expect(await screen.findByText("Serveur injoignable.")).toBeInTheDocument();
    expect(screen.getByText("1.2.0")).toBeInTheDocument();
  });
});
