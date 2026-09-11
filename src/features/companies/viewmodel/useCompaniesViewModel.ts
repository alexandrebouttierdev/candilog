import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { companyService } from "../services/companyService";
import type {
  Company,
  CompanyFilter,
  CompanySize,
  NewCompany,
} from "@/shared/types/generated/companies";
import {
  applicationService,
  EMPTY_FILTER,
  type ApplicationFilter,
} from "@/features/applications";
import { COMPANIES_PAGE_SIZE, PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { useDebounce } from "@/shared/hooks/useDebounce";

/** Root des clés de cache de la feature, pour invalider d'un seul appel. */
export const COMPANIES_KEY = ["entreprises"] as const;

/**
 * Critères du répertoire, hors recherche libre.
 *
 * Trois dimensions indépendantes : l'activité de l'entreprise, sa nature et sa taille. Une
 * société peut être « ESN + PME » comme « Association + TPE ».
 */
export interface CompanyCriteria {
  readonly sector_id: string | null;
  readonly company_type_id: string | null;
  readonly company_size: CompanySize | null;
}

/** Critères vides, état par défaut de l'écran. */
export const EMPTY_CRITERIA: CompanyCriteria = {
  sector_id: null,
  company_type_id: null,
  company_size: null,
};

function applicationsForCompany(company_id: string | null): ApplicationFilter {
  return { ...EMPTY_FILTER, company_id, search: "", sort: "date", descending: true, ids: [] };
}

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
  const [search, setSearchState] = useState("");
  const searchQuery = useDebounce(search);
  const [criteria, setCriteria] = useState<CompanyCriteria>(EMPTY_CRITERIA);
  const [selected_id, setSelectedId] = useState<string | null>(null);

  /** Filtre tel qu'envoyé au backend : SQLite fait la recherche et la pagination. */
  const filter: CompanyFilter = { ...criteria, search: searchQuery };

  const list = useQuery({
    queryKey: [...COMPANIES_KEY, "page", { page, filter }],
    queryFn: () => companyService.listPage({ page, page_size: COMPANIES_PAGE_SIZE, filter }),
  });

  const items: Company[] = list.data?.items ?? [];
  // Les maquettes n'affichent jamais la colonne de droite vide : à défaut de sélection
  // explicite, la première fiche de la page est ouverte.
  const selection = items.find((item) => item.id === selected_id) ?? items[0] ?? null;
  const detailId = selection?.id ?? null;

  // Applications rattachées à la fiche ouverte : les maquettes les affichent sous le
  // bandeau d'identité. Interrogées par le filtre existant plutôt que par une commande
  // dédiée, et seulement quand une fiche est sélectionnée.
  const linked = useQuery({
    queryKey: [...COMPANIES_KEY, "candidatures", detailId],
    enabled: detailId !== null,
    queryFn: () =>
      applicationService.listPage({
        page: 1,
        page_size: PAGE_SIZE,
        filter: applicationsForCompany(detailId),
      }),
  });

  const breakdown = useQuery({
    queryKey: [...COMPANIES_KEY, "repartition", detailId],
    enabled: detailId !== null,
    queryFn: () => applicationService.breakdown(applicationsForCompany(detailId)),
  });
  const companyMetrics = {
    total:
      (breakdown.data?.pending ?? 0) +
      (breakdown.data?.followed_up ?? 0) +
      (breakdown.data?.interview ?? 0) +
      (breakdown.data?.rejected ?? 0),
    interview: breakdown.data?.interview ?? 0,
    pending: breakdown.data?.pending ?? 0,
  };

  /** Recharge toute la feature : liste, filtres et fiche sélectionnée. */
  const invalidate = useCallback(
    () => queryClient.invalidateQueries({ queryKey: COMPANIES_KEY }),
    [queryClient],
  );

  /** Présente l'échec d'une écriture sans faire disparaître le formulaire. */
  const reportFailure = useCallback(
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
      await invalidate();
      setSelectedId(company.id);
      notify({ tone: "success", title: "Entreprise enregistrée", detail: company.name });
    },
    onError: reportFailure("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NewCompany }) =>
      companyService.update(params.id, params.input),
    onSuccess: async (company) => {
      await invalidate();
      notify({ tone: "success", title: "Entreprise modifiée", detail: company.name });
    },
    onError: reportFailure("Modification impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => companyService.delete(id),
    onSuccess: async (_result, id) => {
      await invalidate();
      // La fiche affichée n'existe plus : la garder ouverte laisserait des données mortes
      // à l'écran jusqu'à la prochaine sélection.
      if (selected_id === id) setSelectedId(null);
      notify({ tone: "success", title: "Entreprise supprimée" });
    },
    onError: reportFailure("Suppression impossible"),
  });

  /** Toute recherche ou tout filtre ramène à la première page. */
  const setSearch = useCallback((value: string) => {
    setSearchState(value);
    setPage(1);
  }, []);

  const applyCriteria = useCallback((values: CompanyCriteria) => {
    setCriteria(values);
    setPage(1);
  }, []);

  const resetFilters = useCallback(() => {
    setCriteria(EMPTY_CRITERIA);
    setPage(1);
  }, []);

  /** Nombre de critères actifs, hors recherche libre, pour la pastille du bouton Filtres. */
  const activeFilterCount = [
    criteria.sector_id,
    criteria.company_type_id,
    criteria.company_size,
  ].filter(Boolean).length;

  return {
    items,
    total: list.data?.total ?? 0,
    page,
    page_size: COMPANIES_PAGE_SIZE,
    search,
    setSearch,
    criteria,
    activeFilterCount,
    selection,
    selected_id,
    /** Applications rattachées à la fiche ouverte, page la plus récente. */
    linkedApplications: linked.data?.items ?? [],
    linkedApplicationsTotal: linked.data?.total ?? 0,
    companyMetrics,
    isLoading: list.isPending,
    error: list.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setPage,
    applyCriteria,
    resetFilters,
    select: setSelectedId,
    reload: () => void list.refetch(),
    create: creation.mutateAsync,
    update: modification.mutateAsync,
    delete: suppression.mutateAsync,
  };
}
