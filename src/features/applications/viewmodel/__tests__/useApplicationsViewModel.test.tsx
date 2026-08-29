import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useApplicationsViewModel } from "../useApplicationsViewModel";
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
    company_city: "Rennes",
    contact_id: null,
    contract_type: "CDI",
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
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
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

  it("demande une page plus large en Kanban qu'en Liste", async () => {
    // Le Kanban montre quatre colonnes d'un coup : une page de huit lignes en laisserait
    // trois vides quel que soit le contenu réel du pipeline.
    const listPage = vi
      .spyOn(applicationService, "listPage")
      .mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useApplicationsViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());
    const taillekanban = listPage.mock.calls[0]![0].page_size;

    act(() => result.current.setView("liste"));

    await waitFor(() => {
      const derniere = listPage.mock.calls.at(-1)![0].page_size;
      expect(derniere).toBeLessThan(taillekanban);
    });
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
        status: ["ENTRETIEN"],
        contract: ["CDI"],
        company_id: null,
        city: "",
        job_title: "",
        start_date: null,
        end_date: null,
      }),
    );
    expect(result.current.filtersActifs).toBe(2);
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
});
