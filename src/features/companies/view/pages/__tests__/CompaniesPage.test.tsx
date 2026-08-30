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
import { referentialService } from "@/features/referentials/services/referentialService";
import { REFERENTIELS_DE_TEST } from "@/shared/lib/test-utils";

function ent(name: string): Company {
  return {
    id: name,
    name,
    sector_id: null,
    sector_name: null,
    company_type_id: "IT_SERVICES_COMPANY",
    company_type_name: "ESN / Société de services numériques",
    company_size: "PME",
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
  vi.spyOn(referentialService, "load").mockResolvedValue(REFERENTIELS_DE_TEST);
  vi.spyOn(applicationService, "listPage").mockResolvedValue({
    items: [],
    total: 0,
    page: 1,
    page_size: 8,
    total_pages: 0,
  });
  vi.spyOn(applicationService, "breakdown").mockResolvedValue({
    pending: 0,
    followed_up: 0,
    interview: 0,
    rejected: 0,
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
    await userEvent.click(await screen.findByRole("button", { name: "Client final" }));

    // L'assertion porte sur l'argument réellement transmis : le code est persisté, pas le
    // libellé français affiché dans le menu.
    await waitFor(() =>
      expect(listPage.mock.lastCall?.[0].filter.company_type_id).toBe("FINAL_CLIENT"),
    );
  });
});
