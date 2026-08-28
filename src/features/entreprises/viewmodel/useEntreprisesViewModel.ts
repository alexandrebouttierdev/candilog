import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { entrepriseService } from "../services/entreprise.service";
import type { Entreprise, NouvelleEntreprise } from "../services/entreprise.service";
import { PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Racine des clés de cache de la feature, pour invalider d'un seul appel. */
export const ENTREPRISES_KEY = ["entreprises"] as const;

/**
 * Orchestration de l'écran Entreprises : liste paginée, recherche, filtre, sélection,
 * création, modification et suppression.
 *
 * La pagination et la recherche sont **des paramètres de requête**, pas un filtrage local :
 * la clé de cache les inclut, et chaque changement déclenche un appel au backend qui ne
 * renvoie qu'une page. Filtrer côté React aurait exigé de charger tout le répertoire.
 */
export function useEntreprisesViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const [companyType, setCompanyType] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const liste = useQuery({
    queryKey: [...ENTREPRISES_KEY, "page", { page, search, companyType }],
    queryFn: () =>
      entrepriseService.listerPage({ page, pageSize: PAGE_SIZE, search, companyType }),
  });

  const types = useQuery({
    queryKey: [...ENTREPRISES_KEY, "types"],
    queryFn: entrepriseService.listerTypes,
  });

  /** Recharge toute la feature : liste, filtres et fiche sélectionnée. */
  const invalider = useCallback(
    () => queryClient.invalidateQueries({ queryKey: ENTREPRISES_KEY }),
    [queryClient],
  );

  /** Présente l'échec d'une écriture sans faire disparaître le formulaire. */
  const signalerEchec = useCallback(
    (titre: string) => (error: unknown) => {
      notify({
        tone: "error",
        title: titre,
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
    [notify],
  );

  const creation = useMutation({
    mutationFn: (input: NouvelleEntreprise) => entrepriseService.creer(input),
    onSuccess: async (entreprise) => {
      await invalider();
      setSelectedId(entreprise.id);
      notify({ tone: "success", title: "Entreprise enregistrée", detail: entreprise.nom });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NouvelleEntreprise }) =>
      entrepriseService.modifier(params.id, params.input),
    onSuccess: async (entreprise) => {
      await invalider();
      notify({ tone: "success", title: "Entreprise modifiée", detail: entreprise.nom });
    },
    onError: signalerEchec("Modification impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => entrepriseService.supprimer(id),
    onSuccess: async (_result, id) => {
      await invalider();
      // La fiche affichée n'existe plus : la garder ouverte laisserait des données mortes
      // à l'écran jusqu'à la prochaine sélection.
      if (selectedId === id) setSelectedId(null);
      notify({ tone: "success", title: "Entreprise supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  /** Toute recherche ou tout filtre ramène à la première page. */
  const rechercher = useCallback((valeur: string) => {
    setSearch(valeur);
    setPage(1);
  }, []);

  const filtrerParType = useCallback((valeur: string | null) => {
    setCompanyType(valeur);
    setPage(1);
  }, []);

  const items: Entreprise[] = liste.data?.items ?? [];
  const selection = items.find((item) => item.id === selectedId) ?? null;

  return {
    items,
    total: liste.data?.total ?? 0,
    page,
    pageSize: PAGE_SIZE,
    search,
    companyType,
    types: types.data ?? [],
    selection,
    selectedId,
    isLoading: liste.isPending,
    error: liste.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setPage,
    rechercher,
    filtrerParType,
    selectionner: setSelectedId,
    recharger: () => void liste.refetch(),
    creer: creation.mutateAsync,
    modifier: modification.mutateAsync,
    supprimer: suppression.mutateAsync,
  };
}
