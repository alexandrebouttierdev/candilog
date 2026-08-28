import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { companyService } from "../services/companyService";
import type { Company, NewCompany } from "../services/companyService";
import { applicationService } from "@/features/applications/services/applicationService";
import { PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Root des clés de cache de la feature, pour invalider d'un seul appel. */
export const COMPANIES_KEY = ["entreprises"] as const;

/**
 * Orchestration de l'écran Companies : liste paginée, recherche, filtre, sélection,
 * création, modification et suppression.
 *
 * La pagination et la recherche sont **des paramètres de requête**, pas un filtrage local :
 * la clé de cache les inclut, et chaque changement déclenche un appel au backend qui ne
 * renvoie qu'une page. Filtrer côté React aurait exigé de charger tout le répertoire.
 */
export function useCompaniesViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const [company_type, setCompanyType] = useState<string | null>(null);
  const [selected_id, setSelectedId] = useState<string | null>(null);

  const list = useQuery({
    queryKey: [...COMPANIES_KEY, "page", { page, search, company_type }],
    queryFn: () =>
      companyService.listPage({ page, page_size: PAGE_SIZE, search, company_type }),
  });

  const types = useQuery({
    queryKey: [...COMPANIES_KEY, "types"],
    queryFn: companyService.listTypes,
  });

  const items: Company[] = list.data?.items ?? [];
  // Les maquettes n'affichent jamais la colonne de droite vide : à défaut de sélection
  // explicite, la première fiche de la page est ouverte.
  const selection = items.find((item) => item.id === selected_id) ?? items[0] ?? null;
  const ficheId = selection?.id ?? null;

  // Applications rattachées à la fiche ouverte : les maquettes les affichent sous le
  // bandeau d'identité. Interrogées par le filtre existant plutôt que par une commande
  // dédiée, et seulement quand une fiche est sélectionnée.
  const liees = useQuery({
    queryKey: [...COMPANIES_KEY, "candidatures", ficheId],
    enabled: ficheId !== null,
    queryFn: () =>
      applicationService.listPage({
        page: 1,
        page_size: PAGE_SIZE,
        filter: {
          search: "",
          status: null,
          contract: null,
          company_id: ficheId,
          city: "",
          job_title: "",
          start_date: null,
          end_date: null,
          sort: "date",
          descending: true,
        },
      }),
  });

  /** Recharge toute la feature : liste, filtres et fiche sélectionnée. */
  const invalider = useCallback(
    () => queryClient.invalidateQueries({ queryKey: COMPANIES_KEY }),
    [queryClient],
  );

  /** Présente l'échec d'une écriture sans faire disparaître le formulaire. */
  const signalerEchec = useCallback(
    (title: string) => (error: unknown) => {
      notify({
        tone: "error",
        title: title,
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
    [notify],
  );

  const creation = useMutation({
    mutationFn: (input: NewCompany) => companyService.create(input),
    onSuccess: async (company) => {
      await invalider();
      setSelectedId(company.id);
      notify({ tone: "success", title: "Entreprise enregistrée", detail: company.name });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NewCompany }) =>
      companyService.update(params.id, params.input),
    onSuccess: async (company) => {
      await invalider();
      notify({ tone: "success", title: "Entreprise modifiée", detail: company.name });
    },
    onError: signalerEchec("Modification impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => companyService.delete(id),
    onSuccess: async (_result, id) => {
      await invalider();
      // La fiche affichée n'existe plus : la garder ouverte laisserait des données mortes
      // à l'écran jusqu'à la prochaine sélection.
      if (selected_id === id) setSelectedId(null);
      notify({ tone: "success", title: "Entreprise supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  /** Toute recherche ou tout filtre ramène à la première page. */
  const rechercher = useCallback((value: string) => {
    setSearch(value);
    setPage(1);
  }, []);

  const filtrerParType = useCallback((value: string | null) => {
    setCompanyType(value);
    setPage(1);
  }, []);

  return {
    items,
    total: list.data?.total ?? 0,
    page,
    page_size: PAGE_SIZE,
    search,
    company_type,
    types: types.data ?? [],
    selection,
    selected_id,
    /** Applications rattachées à la fiche ouverte, page la plus récente. */
    applicationsLiees: liees.data?.items ?? [],
    totalApplicationsLiees: liees.data?.total ?? 0,
    isLoading: list.isPending,
    error: list.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setPage,
    rechercher,
    filtrerParType,
    selectionner: setSelectedId,
    recharger: () => void list.refetch(),
    create: creation.mutateAsync,
    update: modification.mutateAsync,
    delete: suppression.mutateAsync,
  };
}
