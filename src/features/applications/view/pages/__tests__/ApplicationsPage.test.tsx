import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { ApplicationsPage } from "../ApplicationsPage";
import { applicationService } from "../../../services/applicationService";
import type { Application } from "@/shared/types/generated/applications";
import { companyService } from "@/features/companies";
import { useUiStore } from "@/shared/lib/ui-store";

function cand(job_title: string, reference_number = 142): Application {
  return {
    id: job_title,
    job_title,
    company_id: "e1",
    company_name: "Nova Digital",
    company_size: "PME",
    contact_id: null,
    application_type: "OFFRE",
    channel: "OFFER",
    reference_number,
    next_follow_up_date: null,
    next_interview_at: null,
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
  vi.spyOn(applicationService, "listPage").mockImplementation(({ page, page_size, filter }) => {
    const all = [cand("Développeur", 142), cand("Designer", 139)];
    const items =
      filter.status.length === 0
        ? all
        : all.filter((application) => filter.status.includes(application.status));
    return Promise.resolve({
      items,
      total: items.length,
      page,
      page_size,
      total_pages: 1,
    });
  });
  vi.spyOn(companyService, "listPage").mockResolvedValue({
    items: [],
    total: 0,
    page: 1,
    page_size: 8,
    total_pages: 0,
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

describe("écran Candidatures — liste groupée", () => {
  it("regroupe les candidatures par statut, avec le décompte du filtre", async () => {
    render(<ApplicationsPage view="list" />, { wrapper });
    const liste = await screen.findByRole("listbox", { name: "Candidatures" });

    expect(within(liste).getByRole("region", { name: "En attente, 2" })).toBeInTheDocument();
    expect(await within(liste).findByText("CAN-142")).toBeInTheDocument();
    expect(within(liste).getByText("CAN-139")).toBeInTheDocument();
  });

  it("replie le groupe Refusée par défaut, et le déplie à la demande", async () => {
    render(<ApplicationsPage view="list" />, { wrapper });
    const bouton = await screen.findByRole("button", { name: "Déplier Refusée" });
    expect(bouton).toHaveAttribute("aria-expanded", "false");
    await userEvent.click(bouton);
    expect(screen.getByRole("button", { name: "Replier Refusée" })).toBeInTheDocument();
  });

  it("ouvre la création au clavier avec N", async () => {
    render(<ApplicationsPage view="list" />, { wrapper });
    await screen.findByText("CAN-142");
    await userEvent.keyboard("n");
    expect(screen.getByRole("dialog", { name: "Nouvelle candidature" })).toBeInTheDocument();
  });

  it("n'autorise la création qu'une fois l'intitulé et l'entreprise renseignés", async () => {
    render(<ApplicationsPage view="list" />, { wrapper });
    await screen.findByText("CAN-142");
    await userEvent.keyboard("n");
    const dialog = screen.getByRole("dialog", { name: "Nouvelle candidature" });
    expect(within(dialog).getByRole("button", { name: /Créer la candidature/ })).toBeDisabled();
    expect(within(dialog).getAllByText("obligatoire").length).toBeGreaterThanOrEqual(2);
  });
});

describe("écran Candidatures — sélection multiple", () => {
  it("propose de supprimer les candidatures cochées", async () => {
    const supprimer = vi.spyOn(applicationService, "delete").mockResolvedValue(undefined);
    render(<ApplicationsPage view="list" />, { wrapper });
    await screen.findByText("CAN-142");

    await userEvent.click(screen.getByRole("checkbox", { name: "Cocher CAN-142" }));
    await userEvent.click(screen.getByRole("checkbox", { name: "Cocher CAN-139" }));
    expect(screen.getByText("2 cochées")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Supprimer" }));
    const dialog = screen.getByRole("alertdialog", { name: "Supprimer 2 candidatures ?" });
    await userEvent.click(within(dialog).getByRole("button", { name: /Supprimer/ }));

    await waitFor(() => expect(supprimer).toHaveBeenCalledTimes(2));
  });
});

describe("écran Candidatures — création depuis le Kanban", () => {
  it("préremplit le statut de la colonne dont on a cliqué le plus", async () => {
    render(<ApplicationsPage view="kanban" />, { wrapper });
    await waitFor(() => expect(screen.getByText("Développeur")).toBeInTheDocument());

    await userEvent.click(
      screen.getByRole("button", { name: "Nouvelle candidature au statut Entretien" }),
    );

    const dialog = screen.getByRole("dialog", { name: "Nouvelle candidature" });
    await waitFor(() =>
      expect(within(dialog).getByRole("radio", { name: /Entretien/ })).toHaveAttribute(
        "aria-checked",
        "true",
      ),
    );
  });
});

describe("écran Candidatures — inspecteur", () => {
  it("n'affiche aucune fiche tant qu'aucune candidature n'est sélectionnée", async () => {
    render(<ApplicationsPage view="list" />, { wrapper });
    await screen.findByText("CAN-142");
    expect(screen.queryByRole("complementary")).not.toBeInTheDocument();
  });

  it("charge la fiche par son identifiant au clic sur une ligne, puis la referme", async () => {
    const get = vi.spyOn(applicationService, "get").mockResolvedValue(cand("Développeur"));
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([]);
    render(<ApplicationsPage view="list" />, { wrapper });
    await userEvent.click(await screen.findByText("Développeur"));

    const fiche = await screen.findByRole("complementary", { name: "Fiche CAN-142" });
    expect(get).toHaveBeenCalledWith("Développeur");

    await userEvent.click(within(fiche).getByRole("button", { name: "Fermer la fiche" }));
    await waitFor(() => expect(screen.queryByRole("complementary")).not.toBeInTheDocument());
  });

  it("ouvre la fiche demandée par l'URL même si elle est absente des groupes chargés", async () => {
    const get = vi.spyOn(applicationService, "get").mockResolvedValue(cand("Data Analyst", 7));
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([]);
    render(<ApplicationsPage view="list" />, { wrapper: wrapperSur("/applications?id=Data%20Analyst") });

    expect(await screen.findByRole("complementary", { name: "Fiche CAN-007" })).toBeInTheDocument();
    expect(get).toHaveBeenCalledWith("Data Analyst");
  });

  it("énumère les conséquences avant de supprimer, puis referme la fiche", async () => {
    vi.spyOn(applicationService, "get").mockResolvedValue(cand("Développeur"));
    vi.spyOn(applicationService, "statusHistory").mockResolvedValue([]);
    vi.spyOn(applicationService, "deletionImpact").mockResolvedValue({
      follow_ups: 1,
      interviews: 2,
      status_changes: 3,
    });
    const supprimer = vi.spyOn(applicationService, "delete").mockResolvedValue(undefined);
    render(<ApplicationsPage view="list" />, { wrapper: wrapperSur("/applications?id=Développeur") });

    const fiche = await screen.findByRole("complementary", { name: "Fiche CAN-142" });
    await userEvent.click(within(fiche).getByRole("button", { name: "Actions sur CAN-142" }));
    await userEvent.click(screen.getByRole("menuitem", { name: /Supprimer…/ }));

    const dialog = screen.getByRole("alertdialog", { name: "Supprimer CAN-142 ?" });
    expect(await within(dialog).findByText("Entretiens")).toBeInTheDocument();
    expect(within(dialog).getByText("L'entreprise et le contact associés sont conservés.")).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole("button", { name: /Supprimer/ }));

    await waitFor(() => expect(supprimer).toHaveBeenCalledWith("Développeur"));
    await waitFor(() => expect(screen.queryByRole("complementary")).not.toBeInTheDocument());
  });
});
