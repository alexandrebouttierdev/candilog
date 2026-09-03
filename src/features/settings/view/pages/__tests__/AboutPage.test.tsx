import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { AboutPage } from "../AboutPage";
import { settingsService } from "../../../services/settingsService";
import { useUiStore } from "@/shared/lib/ui-store";
import userEvent from "@testing-library/user-event";

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
  useUiStore.setState({ onboarding: false });
  vi.spyOn(settingsService, "about").mockResolvedValue({ version: "0.3.0", name: "Candilog" });
});

describe("écran À propos", () => {
  it("présente une fiche d'identité, sans pile technique", async () => {
    render(<AboutPage />, { wrapper });

    await waitFor(() => expect(screen.getByText("0.3.0")).toBeInTheDocument());
    expect(screen.getByText("Alexandre Bouttier")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Visiter le site" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Vérifier les mises à jour" })).toBeInTheDocument();
    expect(screen.queryByText(/Tauri|React|SQLite/i)).not.toBeInTheDocument();
  });

  it("rouvre la présentation sans toucher aux données", async () => {
    render(<AboutPage />, { wrapper });

    await userEvent.click(
      await screen.findByRole("button", { name: "Revoir la présentation" }),
    );

    expect(useUiStore.getState().onboarding).toBe(true);
  });
});
