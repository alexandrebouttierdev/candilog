import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { ApplicationsPage } from "../ApplicationsPage";
import { applicationService } from "../../../services/applicationService";
import type { Application } from "../../../services/applicationService";
import { useUiStore } from "@/shared/lib/ui-store";

function cand(job_title: string): Application {
  return {
    id: job_title,
    job_title,
    company_id: "e1",
    company_name: "Nova Digital",
    company_city: "Rennes",
    contact_id: null,
    contract_type: "CDI",
    status: "EN_ATTENTE",
    sent_date: "2026-08-20",
    job_url: null,
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
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
  vi.spyOn(applicationService, "breakdown").mockResolvedValue({
    pending: 2,
    followed_up: 0,
    interview: 0,
    rejected: 0,
  });
  vi.spyOn(applicationService, "listPage").mockResolvedValue({
    items: [cand("Développeur"), cand("Designer")],
    total: 2,
    page: 1,
    page_size: 32,
    total_pages: 1,
  });
});

describe("écran Candidatures — sélection multiple", () => {
  it("affiche les actions groupées dès qu'une carte est cochée", async () => {
    render(<ApplicationsPage />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());

    expect(screen.queryByText("1 sélectionnée")).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("checkbox", { name: "Sélectionner Développeur" }));

    expect(screen.getByText("1 sélectionnée")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Supprimer" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tout désélectionner" })).toBeInTheDocument();
  });

  it("propose de supprimer toute la page cochée en Liste", async () => {
    const supprimer = vi.spyOn(applicationService, "delete").mockResolvedValue(undefined);
    render(<ApplicationsPage />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());

    await userEvent.click(screen.getByRole("button", { name: "Liste" }));
    await userEvent.click(
      screen.getByRole("checkbox", { name: "Sélectionner les candidatures de la page" }),
    );

    expect(screen.getByText("2 sélectionnées")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Supprimer" }));

    const dialog = screen.getByRole("alertdialog", { name: "Supprimer 2 candidatures ?" });
    expect(
      within(dialog).getByText(
        "Les candidatures sélectionnées seront définitivement supprimées, ainsi que les entretiens et relances rattachés.",
      ),
    ).toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole("button", { name: "Supprimer" }));

    await waitFor(() => expect(supprimer).toHaveBeenCalledTimes(2));
  });
});
