import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { CompaniesPage } from "../CompaniesPage";
import { companyService } from "../../../services/companyService";
import type { Company } from "../../../services/companyService";
import { applicationService } from "@/features/applications/services/applicationService";
import { useUiStore } from "@/shared/lib/ui-store";

function ent(name: string): Company {
  return {
    id: name,
    name,
    sector_id: null,
    sector: null,
    type: "ESN",
    website: null,
    city: null,
    address: null,
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
  vi.spyOn(companyService, "listPage").mockResolvedValue({
    items: [ent("Nova Digital")],
    total: 1,
    page: 1,
    page_size: 8,
    total_pages: 1,
  });
  vi.spyOn(companyService, "listTypes").mockResolvedValue(["ESN", "Cabinet"]);
  vi.spyOn(applicationService, "listPage").mockResolvedValue({
    items: [],
    total: 0,
    page: 1,
    page_size: 8,
    total_pages: 0,
  });
});

describe("écran Entreprises — barre de filtres", () => {
  it("place la recherche et l'action dans la barre, pas un select de type", async () => {
    render(<CompaniesPage />, { wrapper });
    await waitFor(() => expect(screen.getAllByText("Nova Digital").length).toBeGreaterThan(0));

    expect(screen.getByRole("searchbox", { name: "Rechercher…" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Filtres" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Nouvelle entreprise" })).toBeInTheDocument();
    expect(screen.queryByLabelText("Filtrer par type")).not.toBeInTheDocument();
  });

  it("transmet le type choisi au backend", async () => {
    const listPage = vi.spyOn(companyService, "listPage").mockResolvedValue({
      items: [ent("Nova Digital")],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    render(<CompaniesPage />, { wrapper });
    await waitFor(() => expect(screen.getAllByText("Nova Digital").length).toBeGreaterThan(0));

    await userEvent.click(screen.getByRole("button", { name: "Filtres" }));
    await userEvent.click(screen.getByRole("button", { name: "Cabinet" }));

    await waitFor(() =>
      expect(listPage).toHaveBeenLastCalledWith(
        expect.objectContaining({ company_type: "Cabinet" }),
      ),
    );
  });
});
