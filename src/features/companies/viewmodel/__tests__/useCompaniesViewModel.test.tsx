import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { CRITERES_VIDES, useCompaniesViewModel } from "../useCompaniesViewModel";
import { companyService } from "../../services/companyService";
import type { Company } from "../../services/companyService";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { applicationService } from "@/features/applications/services/applicationService";

/** Entreprise minimale, pour n'écrire que ce que chaque test observe. */
function ent(name: string, id = name): Company {
  return {
    id,
    name,
    sector_id: null,
    sector_name: null,
    company_type_id: null,
    company_type_name: null,
    company_size: "UNKNOWN",
    website: null,
    city: null,
    address: null,
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  };
}

function page(items: Company[], total = items.length) {
  return { items, total, page: 1, page_size: 8, total_pages: Math.max(1, Math.ceil(total / 8)) };
}

function wrapper({ children }: { children: ReactNode }) {
  // `retry: false` : sans cela, une erreur simulée serait réessayée et le test attendrait
  // les temporisations de TanStack Query.
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(applicationService, "listPage").mockResolvedValue({
    items: [],
    total: 8,
    page: 1,
    page_size: 8,
    total_pages: 1,
  });
  vi.spyOn(applicationService, "breakdown").mockResolvedValue({
    pending: 0,
    followed_up: 0,
    interview: 0,
    rejected: 0,
  });
});

describe("ViewModel des entreprises", () => {
  it("expose la page renvoyée par le backend", async () => {
    vi.spyOn(companyService, "listPage").mockResolvedValue(
      page([ent("Nova Digital"), ent("Atlas Studio")]),
    );

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.items).toHaveLength(2));
    expect(result.current.total).toBe(2);
  });

  it("calcule les KPI de la fiche sur tout le pipeline", async () => {
    vi.spyOn(companyService, "listPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(applicationService, "breakdown").mockResolvedValue({
      pending: 9,
      followed_up: 4,
      interview: 7,
      rejected: 3,
    });

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.companyMetrics.total).toBe(23));
    expect(result.current.companyMetrics.interview).toBe(7);
    expect(result.current.companyMetrics.pending).toBe(9);
  });

  it("transmet la recherche au backend au lieu de filtrer localement", async () => {
    // C'est la garantie de la pagination : filtrer ici obligerait à charger tout le
    // répertoire, et l'ordre comme les compteurs seraient faux dès la seconde page.
    const listPage = vi
      .spyOn(companyService, "listPage")
      .mockResolvedValue(page([ent("Nova Digital")]));

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());

    act(() => result.current.rechercher("nova"));

    await waitFor(() => expect(listPage.mock.lastCall?.[0].filter.search).toBe("nova"));
  });

  it("revient à la première page à chaque nouvelle recherche", async () => {
    // Rester en page 3 après avoir restreint la recherche afficherait une liste vide alors
    // que des résultats existent.
    const listPage = vi
      .spyOn(companyService, "listPage")
      .mockResolvedValue(page([ent("Nova Digital")], 40));

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(listPage).toHaveBeenCalled());

    act(() => result.current.setPage(3));
    await waitFor(() => expect(result.current.page).toBe(3));

    act(() => result.current.rechercher("nova"));

    await waitFor(() => expect(result.current.page).toBe(1));
  });

  it("sélectionne l'entreprise créée pour ouvrir sa fiche", async () => {
    vi.spyOn(companyService, "listPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(companyService, "create").mockResolvedValue(ent("Nova Digital"));

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.create({
        name: "Nova Digital",
        sector_id: null,
        company_type_id: null,
        company_size: "UNKNOWN",
        website: null,
        city: null,
        address: null,
        notes: null,
      });
    });

    expect(result.current.selected_id).toBe("Nova Digital");
  });

  it("referme la fiche de l'entreprise supprimée", async () => {
    // La garder ouverte laisserait des données mortes à l'écran jusqu'à la prochaine
    // sélection.
    vi.spyOn(companyService, "listPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(companyService, "delete").mockResolvedValue(undefined);

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.selectionner("Nova Digital"));
    await act(async () => {
      await result.current.delete("Nova Digital");
    });

    expect(result.current.selected_id).toBeNull();
  });

  it("présente le message du backend lorsqu'une suppression est refusée", async () => {
    vi.spyOn(companyService, "listPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(companyService, "delete").mockRejectedValue(
      new AppError({
        code: "VALIDATION_ERROR",
        message: "Suppression impossible : des candidatures sont liées à cette entreprise",
      }),
    );

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.delete("Nova Digital").catch(() => undefined);
    });

    const toast = useUiStore.getState().toasts.at(-1);
    expect(toast?.tone).toBe("error");
    expect(toast?.detail).toContain("des candidatures sont liées");
  });

  it("remonte l'erreur de chargement au lieu d'afficher une liste vide", async () => {
    // Une liste vide et une liste en panne se ressemblent à l'écran : la vue a besoin de
    // l'erreur pour afficher un bandeau et un bouton « Réessayer ».
    vi.spyOn(companyService, "listPage").mockRejectedValue(
      new AppError({ code: "DATABASE_ERROR", message: "Fichier illisible." }),
    );

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.error).toBeInstanceOf(AppError));
    expect(result.current.items).toEqual([]);
  });

  it("compte les critères actifs sans la recherche libre", async () => {
    vi.spyOn(companyService, "listPage").mockResolvedValue(page([ent("Nova Digital")]));

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.rechercher("nova"));
    expect(result.current.filtersActifs).toBe(0);

    // Type et taille sont deux axes distincts : les cumuler donne bien deux critères.
    act(() =>
      result.current.appliquerCriteres({
        ...CRITERES_VIDES,
        company_type_id: "IT_SERVICES_COMPANY",
        company_size: "PME",
      }),
    );
    expect(result.current.filtersActifs).toBe(2);
  });

  it("transmet les critères au backend plutôt que de filtrer en mémoire", async () => {
    const listPage = vi
      .spyOn(companyService, "listPage")
      .mockResolvedValue(page([ent("Nova Digital")]));

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() =>
      result.current.appliquerCriteres({
        ...CRITERES_VIDES,
        company_type_id: "IT_SERVICES_COMPANY",
      }),
    );

    await waitFor(() =>
      expect(listPage.mock.lastCall?.[0].filter.company_type_id).toBe("IT_SERVICES_COMPANY"),
    );
  });

  it("ôte les critères au reset et revient à la première page", async () => {
    vi.spyOn(companyService, "listPage").mockResolvedValue(page([ent("Nova Digital")], 40));

    const { result } = renderHook(() => useCompaniesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.setPage(3));
    act(() =>
      result.current.appliquerCriteres({ ...CRITERES_VIDES, company_size: "PME" }),
    );
    await waitFor(() => expect(result.current.page).toBe(1));

    act(() => result.current.setPage(2));
    act(() => result.current.resetFilters());

    expect(result.current.criteres).toEqual(CRITERES_VIDES);
    expect(result.current.page).toBe(1);
  });
});
