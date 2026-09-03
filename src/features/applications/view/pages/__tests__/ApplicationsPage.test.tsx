import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { ApplicationsPage } from "../ApplicationsPage";
import { applicationService } from "../../../services/applicationService";
import type { Application } from "../../../services/applicationService";
import { companyService } from "@/features/companies/services/companyService";
import { useUiStore } from "@/shared/lib/ui-store";

function cand(job_title: string): Application {
  return {
    id: job_title,
    job_title,
    company_id: "e1",
    company_name: "Nova Digital",
    company_size: "PME",
    contact_id: null,
    application_type: "OFFRE",
    contract_type_code: "CDI",
    contract_type_name: "CDI",
    weekly_work_schedule: "FULL_TIME",
    weekly_hours: 35,
    professional_domain_id: "M18",
    professional_domain_name: "Informatique / Télécommunication",
    city: null,
    address: null,
    company_type_id: null,
    effective_city: "Rennes",
    effective_address: null,
    effective_company_type_id: "IT_SERVICES_COMPANY",
    effective_company_type_name: "ESN / Société de services numériques",
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
  vi.spyOn(companyService, "listPage").mockResolvedValue({
    items: [],
    total: 0,
    page: 1,
    page_size: 8,
    total_pages: 0,
  });
});

describe("écran Candidatures — sélection multiple", () => {
  it("affiche les actions groupées dès qu'une carte est cochée", async () => {
    render(<ApplicationsPage />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());
    expect(screen.getByText("2 candidatures")).toBeInTheDocument();

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

describe("écran Candidatures — création depuis le Kanban", () => {
  it("préremplit le statut de la colonne dont on a cliqué le plus", async () => {
    render(<ApplicationsPage />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());

    await userEvent.click(
      screen.getByRole("button", { name: "Nouvelle candidature au statut Entretien" }),
    );

    expect(screen.getByRole("dialog", { name: "Nouvelle candidature" })).toBeInTheDocument();
    await waitFor(() => expect(screen.getByLabelText("Statut")).toHaveValue("ENTRETIEN"));
  });
});

/** Rendu de la page sur une URL donnée, pour les tests de lien profond. */
function wrapperSur(url: string) {
  return function WrapperSur({ children }: { children: ReactNode }) {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    return (
      <QueryClientProvider client={client}>
        <MemoryRouter initialEntries={[url]}>{children}</MemoryRouter>
      </QueryClientProvider>
    );
  };
}

describe("écran Candidatures — panneau de détail", () => {
  it("n'affiche aucun panneau tant qu'aucune candidature n'est sélectionnée", async () => {
    render(<ApplicationsPage />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());
    await userEvent.click(screen.getByRole("button", { name: "Liste" }));

    expect(screen.queryByText("Aucune sélection")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Fermer l'inspecteur" }),
    ).not.toBeInTheDocument();
  });

  it("charge la fiche par son identifiant au clic sur une ligne, puis la referme", async () => {
    const get = vi.spyOn(applicationService, "get").mockResolvedValue(cand("Développeur"));
    render(<ApplicationsPage />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());
    await userEvent.click(screen.getByRole("button", { name: "Liste" }));

    await userEvent.click(screen.getByText("Développeur"));

    expect(
      await screen.findByRole("complementary", { name: "Développeur" }),
    ).toBeInTheDocument();
    expect(get).toHaveBeenCalledWith("Développeur");

    await userEvent.click(screen.getByRole("button", { name: "Fermer l'inspecteur" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("complementary", { name: "Développeur" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("ouvre la fiche demandée par l'URL même si elle est absente de la page courante", async () => {
    const get = vi.spyOn(applicationService, "get").mockResolvedValue(cand("Data Analyst"));
    render(<ApplicationsPage />, { wrapper: wrapperSur("/candidatures?fiche=Data%20Analyst") });

    expect(
      await screen.findByRole("complementary", { name: "Data Analyst" }),
    ).toBeInTheDocument();
    expect(get).toHaveBeenCalledWith("Data Analyst");
  });

  it("referme le panneau quand la candidature affichée est supprimée", async () => {
    vi.spyOn(applicationService, "get").mockResolvedValue(cand("Développeur"));
    vi.spyOn(applicationService, "delete").mockResolvedValue(undefined);
    render(<ApplicationsPage />, { wrapper: wrapperSur("/candidatures?fiche=Développeur") });

    const panneau = await screen.findByRole("complementary", { name: "Développeur" });
    await userEvent.click(within(panneau).getByRole("button", { name: "Supprimer" }));

    const dialog = screen.getByRole("alertdialog", { name: "Supprimer cette candidature ?" });
    await userEvent.click(within(dialog).getByRole("button", { name: "Supprimer" }));

    await waitFor(() =>
      expect(
        screen.queryByRole("complementary", { name: "Développeur" }),
      ).not.toBeInTheDocument(),
    );
  });
});
