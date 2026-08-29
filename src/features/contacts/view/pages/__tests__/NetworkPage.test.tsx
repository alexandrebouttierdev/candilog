import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { NetworkPage } from "../NetworkPage";
import { contactService } from "../../../services/contactService";
import type { Contact } from "../../../services/contactService";
import { useUiStore } from "@/shared/lib/ui-store";

function ct(name: string): Contact {
  return {
    id: name,
    company_id: null,
    company_name: null,
    first_name: "Camille",
    name,
    job_title: null,
    tracking_role: "Recruteur",
    email: null,
    phone: null,
    linkedin: null,
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
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
  useUiStore.setState({ toasts: [] });
  vi.spyOn(contactService, "listPage").mockResolvedValue({
    items: [ct("Rivet")],
    total: 1,
    page: 1,
    page_size: 8,
    total_pages: 1,
  });
});

describe("écran Réseau — barre de filtres", () => {
  it("place la recherche et l'action dans la barre", async () => {
    render(<NetworkPage />, { wrapper });
    await waitFor(() => expect(screen.getAllByText("Camille Rivet").length).toBeGreaterThan(0));

    expect(screen.getByRole("searchbox", { name: "Rechercher…" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Filtres" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Nouveau contact" })).toBeInTheDocument();
  });

  it("transmet le rôle choisi au backend", async () => {
    const listPage = vi.spyOn(contactService, "listPage").mockResolvedValue({
      items: [ct("Rivet")],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    render(<NetworkPage />, { wrapper });
    await waitFor(() => expect(screen.getAllByText("Camille Rivet").length).toBeGreaterThan(0));

    await userEvent.click(screen.getByRole("button", { name: "Filtres" }));
    await userEvent.click(screen.getByRole("button", { name: "Manager" }));

    await waitFor(() =>
      expect(listPage).toHaveBeenLastCalledWith(
        expect.objectContaining({ tracking_role: "Manager" }),
      ),
    );
  });
});
