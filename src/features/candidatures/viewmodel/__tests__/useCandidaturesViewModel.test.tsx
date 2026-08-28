import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useCandidaturesViewModel } from "../useCandidaturesViewModel";
import { candidatureService } from "../../services/candidature.service";
import type { Candidature } from "../../services/candidature.service";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

function cand(poste: string, statut: Candidature["statut"] = "EN_ATTENTE"): Candidature {
  return {
    id: poste,
    poste,
    entrepriseId: "e1",
    entrepriseNom: "Nova Digital",
    entrepriseVille: "Rennes",
    contactId: null,
    typeContrat: "CDI",
    statut,
    dateEnvoi: "2026-08-20",
    lienOffre: null,
    notes: null,
    createdAt: "2026-08-20T00:00:00Z",
    updatedAt: "2026-08-20T00:00:00Z",
  };
}

function page(items: Candidature[], total = items.length, pageSize = 32) {
  return { items, total, page: 1, pageSize, totalPages: Math.max(1, Math.ceil(total / pageSize)) };
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
  vi.spyOn(candidatureService, "repartition").mockResolvedValue({
    enAttente: 7,
    relancee: 4,
    entretien: 2,
    refus: 2,
  });
});

describe("ViewModel des candidatures", () => {
  it("expose la page renvoyée par le backend", async () => {
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(
      page([cand("Développeur"), cand("Designer")]),
    );

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.items).toHaveLength(2));
  });

  it("expose la répartition calculée sur tout le filtre, pas sur la page", async () => {
    // Les en-têtes de colonnes du Kanban en dépendent : compter les cartes affichées
    // annoncerait « 2 » sur une colonne qui contient tout le pipeline.
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")], 15));

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });

    await waitFor(() => expect(result.current.repartition.enAttente).toBe(7));
    expect(result.current.items).toHaveLength(1);
  });

  it("demande une page plus large en Kanban qu'en Liste", async () => {
    // Le Kanban montre quatre colonnes d'un coup : une page de huit lignes en laisserait
    // trois vides quel que soit le contenu réel du pipeline.
    const listerPage = vi
      .spyOn(candidatureService, "listerPage")
      .mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(listerPage).toHaveBeenCalled());
    const taillekanban = listerPage.mock.calls[0]![0].pageSize;

    act(() => result.current.setVue("liste"));

    await waitFor(() => {
      const derniere = listerPage.mock.calls.at(-1)![0].pageSize;
      expect(derniere).toBeLessThan(taillekanban);
    });
  });

  it("transmet la recherche au backend et revient en première page", async () => {
    const listerPage = vi
      .spyOn(candidatureService, "listerPage")
      .mockResolvedValue(page([cand("Développeur")], 40));

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(listerPage).toHaveBeenCalled());

    act(() => result.current.setPage(3));
    await waitFor(() => expect(result.current.page).toBe(3));

    act(() => result.current.rechercher("nova"));

    await waitFor(() => expect(result.current.page).toBe(1));
    expect(listerPage.mock.calls.at(-1)![0].filtre.search).toBe("nova");
  });

  it("inverse la direction quand on retrie la colonne courante", async () => {
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    expect(result.current.tri).toBe("date");
    expect(result.current.descendant).toBe(true);

    act(() => result.current.trierPar("date"));
    expect(result.current.descendant).toBe(false);
  });

  it("repart en descendant sur une nouvelle colonne de tri", async () => {
    // Conserver la direction précédente donnerait un premier clic dont l'effet dépend de
    // l'historique des clics sur une autre colonne.
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.trierPar("date"));
    act(() => result.current.trierPar("poste"));

    expect(result.current.tri).toBe("poste");
    expect(result.current.descendant).toBe(true);
  });

  it("compte les filtres actifs sans la recherche libre", async () => {
    // La recherche a son propre champ visible : la compter dans la pastille du bouton
    // « Filtres » ferait croire à un critère caché.
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")]));

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.rechercher("nova"));
    expect(result.current.filtresActifs).toBe(0);

    act(() =>
      result.current.appliquerFiltres({
        statut: "ENTRETIEN",
        contrat: "CDI",
        entrepriseId: null,
        ville: "",
        poste: "",
        dateDebut: null,
        dateFin: null,
      }),
    );
    expect(result.current.filtresActifs).toBe(2);
  });

  it("n'annonce pas de succès après un changement de statut", async () => {
    // Le déplacement de la carte est déjà la confirmation visible du geste : un toast à
    // chaque glisser-déposer noierait les messages qui comptent.
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(candidatureService, "changerStatut").mockResolvedValue(
      cand("Développeur", "ENTRETIEN"),
    );

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current.changerStatut({ id: "Développeur", statut: "ENTRETIEN" });
    });

    expect(useUiStore.getState().toasts).toHaveLength(0);
  });

  it("annonce l'échec d'un changement de statut", async () => {
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(candidatureService, "changerStatut").mockRejectedValue(
      new AppError({ code: "NOT_FOUND", message: "Introuvable : candidature." }),
    );

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    await act(async () => {
      await result.current
        .changerStatut({ id: "Développeur", statut: "ENTRETIEN" })
        .catch(() => undefined);
    });

    expect(useUiStore.getState().toasts.at(-1)?.tone).toBe("error");
  });

  it("referme le détail de la candidature supprimée", async () => {
    vi.spyOn(candidatureService, "listerPage").mockResolvedValue(page([cand("Développeur")]));
    vi.spyOn(candidatureService, "supprimer").mockResolvedValue(undefined);

    const { result } = renderHook(() => useCandidaturesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.items).toHaveLength(1));

    act(() => result.current.selectionner("Développeur"));
    await act(async () => {
      await result.current.supprimer("Développeur");
    });

    expect(result.current.selectedId).toBeNull();
  });
});
