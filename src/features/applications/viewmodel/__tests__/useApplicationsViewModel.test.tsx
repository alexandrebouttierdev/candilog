import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { useApplicationsViewModel } from "../useApplicationsViewModel";
import { FILTER_VIDE } from "../../model/schemas/application-filter.schema";
import { applicationService } from "../../services/applicationService";
import type { Application } from "../../services/applicationService";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

function cand(job_title: string, status: Application["status"] = "EN_ATTENTE"): Application {
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
    status,
    sent_date: "2026-08-20",
    job_url: null,
    notes: null,
    created_at: "2026-08-20T00:00:00Z",
    updated_at: "2026-08-20T00:00:00Z",
  };
}

function page(items: Application[], total = items.length, page_size = 32) {
  return { items, total, page: 1, page_size, total_pages: Math.max(1, Math.ceil(total / page_size)) };
}

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  // La fiche ouverte est un paramètre d'URL : le ViewModel a besoin d'un routeur.
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
    pending: 7,
    followed_up: 4,
    interview: 2,
    rejected: 2,
  });
});

describe("ViewModel des candidatures", () => {
  it("expose la page renvoyée par le backend", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(
      page([cand("Développeur"), cand("Designer")]),
    );

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });

    await waitFor(() => expect(result.current.items).toHaveLength(2));
  });

  it("expose la répartition calculée sur tout le filtre, pas sur la page", async () => {
    // Les en-têtes de colonnes du Kanban en dépendent : compter les cartes affichées
    // annoncerait « 2 » sur une colonne qui contient tout le pipeline.
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")], 15));

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });

    await waitFor(() => expect(result.current.breakdown.pending).toBe(7));
    expect(result.current.items).toHaveLength(1);
  });

  it("demande une page backend indépendante pour chaque colonne du Kanban", async () => {
    // Une page globale répartie ensuite côté React tronque certaines colonnes et oblige à
    // charger tout le pipeline. Chaque statut doit donc être paginé directement par SQLite.
    const listPage = vi
      .spyOn(applicationService, "listPage")
      .mockResolvedValue(page([cand("Développeur")]));

    renderHook(() => useApplicationsViewModel(), { wrapper });

    await waitFor(() => expect(listPage).toHaveBeenCalledTimes(4));
    expect(listPage.mock.calls.map(([params]) => params.filter.status)).toEqual([
      ["EN_ATTENTE"],
      ["RELANCEE"],
      ["ENTRETIEN"],
      ["REFUS"],
    ]);
    expect(listPage.mock.calls.every(([params]) => params.page_size === 8)).toBe(true);
  });

  it("change la page d'une seule colonne du Kanban", async () => {
    const listPage = vi.spyOn(applicationService, "listPage").mockImplementation((params) =>
      Promise.resolve({
        items: [],
        total: params.filter.status[0] === "REFUS" ? 12 : 0,
        page: params.page,
        page_size: params.page_size,
        total_pages: params.filter.status[0] === "REFUS" ? 2 : 1,
      }),
    );
    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalledTimes(4));
    listPage.mockClear();

    act(() => result.current.setKanbanPage("REFUS", 2));

    await waitFor(() =>
      expect(
        listPage.mock.calls.some(
          ([params]) =>
            params.page === 2 &&
            params.page_size === 8 &&
            params.filter.status.length === 1 &&
            params.filter.status[0] === "REFUS",
        ),
      ).toBe(true),
    );
    expect(result.current.kanbanPages).toEqual({
      EN_ATTENTE: 1,
      RELANCEE: 1,
      ENTRETIEN: 1,
      REFUS: 2,
    });
    expect(
      listPage.mock.calls.some(
        ([params]) => params.page === 2 && params.filter.status[0] !== "REFUS",
      ),
    ).toBe(false);
  });

  it("transmet la recherche au backend et revient en première page", async () => {
    const listPage = vi
      .spyOn(applicationService, "listPage")
      .mockResolvedValue(page([cand("Développeur")], 40));

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());

    act(() => result.current.setPage(3));
    await waitFor(() => expect(result.current.page).toBe(3));

    act(() => result.current.rechercher("nova"));

    await waitFor(() => expect(result.current.page).toBe(1));
    await waitFor(() => expect(listPage.mock.calls.at(-1)?.[0].filter.search).toBe("nova"));
  });

  it("inverse la direction quand on retrie la colonne courante", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    expect(result.current.sort).toBe("date");
    expect(result.current.descending).toBe(true);

    act(() => result.current.trierPar("date"));
    expect(result.current.descending).toBe(false);
  });

  it("repart en descendant sur une nouvelle colonne de tri", async () => {
    // Conserver la direction précédente donnerait un premier clic dont l'effet dépend de
    // l'historique des clics sur une autre colonne.
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.trierPar("date"));
    act(() => result.current.trierPar("job_title"));

    expect(result.current.sort).toBe("job_title");
    expect(result.current.descending).toBe(true);
  });

  it("compte les filtres actifs sans la recherche libre", async () => {
    // La recherche a son propre champ visible : la compter dans la pastille du bouton
    // « Filtres » ferait croire à un critère caché.
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.rechercher("nova"));
    expect(result.current.filtersActifs).toBe(0);

    act(() =>
      result.current.appliquerFilters({
        ...FILTER_VIDE,
        status: ["ENTRETIEN"],
        contract_type_code: ["CDI"],
        company_size: ["PME"],
        min_weekly_hours: 20,
      }),
    );
    expect(result.current.filtersActifs).toBe(4);
  });

  it("n'annonce pas de succès après un changement de statut", async () => {
    // Le déplacement de la carte est déjà la confirmation visible du geste : un toast à
    // chaque glisser-déposer noierait les messages qui comptent.
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(applicationService, "changeStatus").mockResolvedValue(
      cand("Développeur", "ENTRETIEN"),
    );

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.changeStatus({ id: "Développeur", status: "ENTRETIEN" });
    });

    expect(useUiStore.getState().toasts).toHaveLength(0);
  });

  it("annonce l'échec d'un changement de statut", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(applicationService, "changeStatus").mockRejectedValue(
      new AppError({ code: "NOT_FOUND", message: "Introuvable : candidature." }),
    );

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current
        .changeStatus({ id: "Développeur", status: "ENTRETIEN" })
        .catch(() => undefined);
    });

    expect(useUiStore.getState().toasts.at(-1)?.tone).toBe("error");
  });

  it("n'envoie pas la sélection dans la requête de liste", async () => {
    // La sélection sert l'export et la suppression groupée : la mettre dans la clé de
    // liste rechargerait la page à chaque case cochée.
    const listPage = vi
      .spyOn(applicationService, "listPage")
      .mockResolvedValue(page([cand("Développeur")]));

    renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());

    expect(listPage.mock.calls[0]![0].filter.ids).toEqual([]);
  });

  it("supprime chaque identifiant d'une sélection et referme le détail", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(
      page([cand("Développeur"), cand("Designer")]),
    );
    const supprimer = vi.spyOn(applicationService, "delete").mockResolvedValue(undefined);

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(2));

    act(() => result.current.selectionner("Développeur"));
    await act(async () => {
      await result.current.deleteMany(["Développeur", "Designer"]);
    });

    expect(supprimer).toHaveBeenCalledTimes(2);
    expect(result.current.selected_id).toBeNull();
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("2 candidatures supprimées");
  });

  it("referme le détail de la candidature supprimée", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(applicationService, "delete").mockResolvedValue(undefined);

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.selectionner("Développeur"));
    await act(async () => {
      await result.current.delete("Développeur");
    });

    expect(result.current.selected_id).toBeNull();
  });

  it("exporte le filtre reçu et annonce le nombre de candidatures", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));
    const exportCsv = vi.spyOn(applicationService, "exportCsv").mockResolvedValue(2);
    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => { await result.current.exportCsv(result.current.filter); });

    expect(exportCsv).toHaveBeenCalledWith(result.current.filter);
    expect(useUiStore.getState().toasts.at(-1)).toMatchObject({
      tone: "success",
      title: "Export terminé",
      detail: "2 candidatures exportées.",
    });
  });

  it("n'annonce rien quand l'export est annulé", async () => {
    vi.spyOn(applicationService, "listPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(applicationService, "exportCsv").mockResolvedValue(null);
    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => { await result.current.exportCsv(result.current.filter); });

    expect(useUiStore.getState().toasts).toHaveLength(0);
  });
});
