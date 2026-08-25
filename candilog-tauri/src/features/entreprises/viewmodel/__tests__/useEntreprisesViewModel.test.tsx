import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useEntreprisesViewModel } from "../useEntreprisesViewModel";
import { entrepriseService } from "../../services/entreprise.service";
import type { Entreprise } from "../../services/entreprise.service";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Entreprise minimale, pour n'écrire que ce que chaque test observe. */
function ent(nom: string, id = nom): Entreprise {
  return {
    id,
    nom,
    secteurId: null,
    secteur: null,
    type: null,
    siteWeb: null,
    ville: null,
    adresse: null,
    notes: null,
    createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z",
  };
}

function page(items: Entreprise[], total = items.length) {
  return { items, total, page: 1, pageSize: 8, totalPages: Math.max(1, Math.ceil(total / 8)) };
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
  vi.spyOn(entrepriseService, "listerTypes").mockResolvedValue([]);
});

describe("ViewModel des entreprises", () => {
  it("expose la page renvoyée par le backend", async () => {
    vi.spyOn(entrepriseService, "listerPage").mockResolvedValue(
      page([ent("Nova Digital"), ent("Atlas Studio")]),
    );

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.items).toHaveLength(2));
    expect(result.current.total).toBe(2);
  });

  it("transmet la recherche au backend au lieu de filtrer localement", async () => {
    // C'est la garantie de la pagination : filtrer ici obligerait à charger tout le
    // répertoire, et l'ordre comme les compteurs seraient faux dès la seconde page.
    const listerPage = vi
      .spyOn(entrepriseService, "listerPage")
      .mockResolvedValue(page([ent("Nova Digital")]));

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });
    await waitFor(() => expect(listerPage).toHaveBeenCalled());

    act(() => result.current.rechercher("nova"));

    await waitFor(() =>
      expect(listerPage).toHaveBeenLastCalledWith(
        expect.objectContaining({ search: "nova" }),
      ),
    );
  });

  it("revient à la première page à chaque nouvelle recherche", async () => {
    // Rester en page 3 après avoir restreint la recherche afficherait une liste vide alors
    // que des résultats existent.
    const listerPage = vi
      .spyOn(entrepriseService, "listerPage")
      .mockResolvedValue(page([ent("Nova Digital")], 40));

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });
    await waitFor(() => expect(listerPage).toHaveBeenCalled());

    act(() => result.current.setPage(3));
    await waitFor(() => expect(result.current.page).toBe(3));

    act(() => result.current.rechercher("nova"));

    await waitFor(() => expect(result.current.page).toBe(1));
  });

  it("sélectionne l'entreprise créée pour ouvrir sa fiche", async () => {
    vi.spyOn(entrepriseService, "listerPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(entrepriseService, "creer").mockResolvedValue(ent("Nova Digital"));

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.creer({
        nom: "Nova Digital",
        secteurId: null,
        secteur: null,
        type: null,
        siteWeb: null,
        ville: null,
        adresse: null,
        notes: null,
      });
    });

    expect(result.current.selectedId).toBe("Nova Digital");
  });

  it("referme la fiche de l'entreprise supprimée", async () => {
    // La garder ouverte laisserait des données mortes à l'écran jusqu'à la prochaine
    // sélection.
    vi.spyOn(entrepriseService, "listerPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(entrepriseService, "supprimer").mockResolvedValue(undefined);

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.selectionner("Nova Digital"));
    await act(async () => {
      await result.current.supprimer("Nova Digital");
    });

    expect(result.current.selectedId).toBeNull();
  });

  it("présente le message du backend lorsqu'une suppression est refusée", async () => {
    vi.spyOn(entrepriseService, "listerPage").mockResolvedValue(page([ent("Nova Digital")]));
    vi.spyOn(entrepriseService, "supprimer").mockRejectedValue(
      new AppError({
        code: "VALIDATION_ERROR",
        message: "Suppression impossible : des candidatures sont liées à cette entreprise",
      }),
    );

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.supprimer("Nova Digital").catch(() => undefined);
    });

    const toast = useUiStore.getState().toasts.at(-1);
    expect(toast?.tone).toBe("error");
    expect(toast?.detail).toContain("des candidatures sont liées");
  });

  it("remonte l'erreur de chargement au lieu d'afficher une liste vide", async () => {
    // Une liste vide et une liste en panne se ressemblent à l'écran : la vue a besoin de
    // l'erreur pour afficher un bandeau et un bouton « Réessayer ».
    vi.spyOn(entrepriseService, "listerPage").mockRejectedValue(
      new AppError({ code: "DATABASE_ERROR", message: "Fichier illisible." }),
    );

    const { result } = renderHook(() => useEntreprisesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.error).toBeInstanceOf(AppError));
    expect(result.current.items).toEqual([]);
  });
});
