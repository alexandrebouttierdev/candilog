import { beforeEach, describe, expect, it, vi } from "vitest";
import type { MockInstance } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { RelationsPage } from "../RelationsPage";
import { companyService } from "@/features/companies";
import type { Company } from "@/features/companies";
import { contactService } from "@/features/contacts";
import type { Contact } from "@/features/contacts";
import { applicationService } from "@/features/applications";
import { referentialService } from "@/features/referentials";
import * as externalLink from "@/shared/services/external-link";
import { ChromeProvider } from "@/shared/lib/chrome";
import { AppError } from "@/shared/types/app-error";
import { REFERENTIELS_DE_TEST, createTestQueryClient } from "@/shared/lib/test-utils";
import { useUiStore } from "@/shared/lib/ui-store";
import type { Page } from "@/shared/types/page";

function entreprise(name: string, overrides: Partial<Company> = {}): Company {
  return {
    id: name,
    name,
    sector_id: null,
    sector_name: "Services informatiques",
    company_type_id: null,
    company_type_name: null,
    company_size: "PME",
    website: "https://vallis-conseil.fr",
    city: "Rennes (35)",
    address: null,
    notes: null,
    created_at: "2026-08-01T00:00:00Z",
    updated_at: "2026-08-01T00:00:00Z",
    activity: { open_applications: 1, applications: 1, contacts: 2, last_reference_number: 133, last_sent_date: "2026-08-29" },
    ...overrides,
  };
}

function contact(name: string, overrides: Partial<Contact> = {}): Contact {
  return {
    id: name,
    company_id: "Vallis Conseil",
    company_name: "Vallis Conseil",
    first_name: "Claire",
    name,
    job_title: "Chargée de recrutement",
    tracking_role: "Recruteur",
    email: "c.menard@vallis-conseil.fr",
    phone: null,
    linkedin: "https://linkedin.com/in/claire",
    notes: null,
    created_at: "2026-08-01T00:00:00Z",
    updated_at: "2026-08-01T00:00:00Z",
    activity: { applications: 1, last_reference_number: 133, last_sent_date: "2026-08-29" },
    ...overrides,
  };
}

function page<T>(items: T[]): Page<T> {
  return { items, total: items.length, page: 1, page_size: 50, total_pages: 1 };
}

function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={createTestQueryClient()}>
      <MemoryRouter>
        <ChromeProvider>{children}</ChromeProvider>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

let listCompanies: MockInstance<typeof companyService.listPage>;
let listContacts: MockInstance<typeof contactService.listPage>;

/** Critères reçus par le backend lors des appels successifs à la liste des entreprises. */
function companyFilters() {
  return listCompanies.mock.calls.map(([params]) => params.filter);
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(referentialService, "load").mockResolvedValue(REFERENTIELS_DE_TEST);
  listCompanies = vi.spyOn(companyService, "listPage").mockImplementation(({ filter }) =>
    Promise.resolve(
      filter.relation_state === "active"
        ? page([entreprise("Vallis Conseil")])
        : filter.relation_state === "watch"
          ? page([entreprise("Sémaphore IT", { activity: { open_applications: 0, applications: 0, contacts: 1, last_reference_number: null, last_sent_date: null } })])
          : filter.relation_state === null
            ? { ...page([entreprise("x")]), total: 2 }
            : page([]),
    ),
  );
  listContacts = vi.spyOn(contactService, "listPage").mockImplementation(({ linked }) =>
    Promise.resolve(
      linked === true
        ? page([contact("Ménard")])
        : linked === false
          ? page([contact("Cozic", { first_name: "Yann", company_id: null, company_name: null, activity: { applications: 0, last_reference_number: null, last_sent_date: null } })])
          : { ...page([contact("x")]), total: 2 },
    ),
  );
  vi.spyOn(applicationService, "listPage").mockResolvedValue(page([]));
});

describe("Relations — entreprises", () => {
  it("groupe les entreprises en cours et repérées, avec leurs décomptes", async () => {
    render(<RelationsPage kind="companies" />, { wrapper });

    const enCours = await screen.findByRole("region", { name: "En cours, 1" });
    expect(within(enCours).getByText("Vallis Conseil")).toBeInTheDocument();
    expect(within(enCours).getByText("2 contacts")).toBeInTheDocument();
    expect(within(enCours).getByText("CAN-133")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Repérées, 1" })).toHaveTextContent("Sémaphore IT");
    // Un groupe vide n'est pas affiché.
    expect(screen.queryByRole("region", { name: /Clôturées/ })).not.toBeInTheDocument();
  });

  it("transmet la recherche et le type choisi au backend plutôt que de filtrer en mémoire", async () => {
    render(<RelationsPage kind="companies" />, { wrapper });
    await screen.findByRole("region", { name: "En cours, 1" });

    await userEvent.type(screen.getByRole("searchbox", { name: "Rechercher une entreprise" }), "vallis");
    await waitFor(() => expect(companyFilters().some((filter) => filter.search === "vallis")).toBe(true));

    await userEvent.click(screen.getByRole("button", { name: /Filtre/ }));
    await userEvent.click(await screen.findByRole("menuitem", { name: /Type d'entreprise/ }));
    await userEvent.click(await screen.findByRole("menuitemradio", { name: REFERENTIELS_DE_TEST.company_types[0]!.name }));
    const code = REFERENTIELS_DE_TEST.company_types[0]!.code;
    await waitFor(() => expect(companyFilters().some((filter) => filter.company_type_id === code)).toBe(true));
  });

  it("ouvre le site de l'entreprise via openExternal", async () => {
    const ouvrir = vi.spyOn(externalLink, "openExternal").mockResolvedValue(undefined);
    render(<RelationsPage kind="companies" />, { wrapper });

    const fiche = await screen.findByRole("complementary", { name: "Fiche Vallis Conseil" });
    await userEvent.click(within(fiche).getByRole("button", { name: "Site web" }));

    expect(ouvrir).toHaveBeenCalledWith("https://vallis-conseil.fr");
  });

  it("présente le message du backend quand une suppression est refusée", async () => {
    vi.spyOn(companyService, "delete").mockRejectedValue(
      new AppError({ code: "VALIDATION", message: "Suppression impossible : des candidatures y sont rattachées." }),
    );
    render(<RelationsPage kind="companies" />, { wrapper });

    const fiche = await screen.findByRole("complementary", { name: "Fiche Vallis Conseil" });
    await userEvent.click(within(fiche).getByRole("button", { name: "Actions sur Vallis Conseil" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "Supprimer…" }));
    const dialogue = await screen.findByRole("alertdialog");
    await userEvent.click(within(dialogue).getByRole("button", { name: /Supprimer/ }));

    await waitFor(() =>
      expect(useUiStore.getState().toasts.map((toast) => toast.detail)).toContain(
        "Suppression impossible : des candidatures y sont rattachées.",
      ),
    );
  });
});

describe("Relations — contacts", () => {
  it("sépare les recruteurs rattachés du réseau", async () => {
    render(<RelationsPage kind="contacts" />, { wrapper });

    expect(await screen.findByRole("region", { name: "Recruteurs et managers, 1" })).toHaveTextContent("Claire Ménard");
    expect(screen.getByRole("region", { name: "Réseau, 1" })).toHaveTextContent("Yann Cozic");
  });

  it("transmet le rôle choisi au backend", async () => {
    render(<RelationsPage kind="contacts" />, { wrapper });
    await screen.findByRole("region", { name: "Recruteurs et managers, 1" });

    await userEvent.keyboard("f");
    await userEvent.click(await screen.findByRole("menuitem", { name: /Rôle/ }));
    await userEvent.click(await screen.findByRole("menuitemradio", { name: "Manager" }));

    await waitFor(() => expect(listContacts).toHaveBeenCalledWith(expect.objectContaining({ tracking_role: "Manager" })));
  });

  it("garde un lien natif pour le courriel et ouvre LinkedIn via openExternal", async () => {
    const ouvrir = vi.spyOn(externalLink, "openExternal").mockResolvedValue(undefined);
    render(<RelationsPage kind="contacts" />, { wrapper });

    const fiche = await screen.findByRole("complementary", { name: "Fiche Claire Ménard" });
    expect(within(fiche).getByRole("link", { name: "c.menard@vallis-conseil.fr" })).toHaveAttribute(
      "href",
      "mailto:c.menard@vallis-conseil.fr",
    );
    await userEvent.click(within(fiche).getByRole("button", { name: "linkedin.com/in/claire" }));
    expect(ouvrir).toHaveBeenCalledWith("https://linkedin.com/in/claire");
  });

  it("sélectionne le contact créé pour ouvrir sa fiche", async () => {
    const cree = contact("Hamon", { first_name: "Marie" });
    vi.spyOn(contactService, "create").mockResolvedValue(cree);
    render(<RelationsPage kind="contacts" />, { wrapper });
    await screen.findByRole("region", { name: "Recruteurs et managers, 1" });

    await userEvent.keyboard("n");
    const dialogue = await screen.findByRole("dialog");
    await userEvent.type(within(dialogue).getByLabelText(/Prénom/), "Marie");
    await userEvent.type(within(dialogue).getByLabelText(/^Nom/), "Hamon");
    await userEvent.keyboard("{Meta>}{Enter}{/Meta}");

    await waitFor(() => expect(contactService.create).toHaveBeenCalled());
  });
});
