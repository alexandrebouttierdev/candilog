import { useCallback, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { applicationService } from "../services/applicationService";
import type {
  Application,
  ApplicationFilter,
  NewApplication,
  ApplicationStatus,
} from "../services/applicationService";
import type { ApplicationFilterValues } from "../model/schemas/application-filter.schema";
import type { ApplicationSort } from "@/shared/types/generated/applications";
import { PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Root des clés de cache de la feature. */
export const APPLICATIONS_KEY = ["candidatures"] as const;

/** Mode d'affichage du suivi. */
export type TrackingView = "kanban" | "liste";

/** Filters appliqués, dans leur forme validée. */
const INITIAL_FILTER: ApplicationFilterValues = {
  status: null,
  contract: null,
  company_id: null,
  city: "",
  job_title: "",
  start_date: null,
  end_date: null,
};

/**
 * Orchestration de l'écran Tracking → Applications.
 *
 * Sert les deux vues, Kanban et List, sur la même requête : elles n'affichent pas les
 * mêmes formes mais interrogent le même filtre, et les séparer aurait dupliqué l'état des
 * filtres, du tri et de la pagination.
 */
export function useApplicationsViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [view, setView] = useState<TrackingView>("kanban");
  const [page, setPage] = useState(1);
  const [sizePage, setSizePage] = useState<number>(PAGE_SIZE);
  const [search, setSearch] = useState("");
  const [filters, setFilters] = useState<ApplicationFilterValues>(INITIAL_FILTER);
  const [sort, setSort] = useState<ApplicationSort>("date");
  const [descending, setDescending] = useState(true);
  const [selected_id, setSelectedId] = useState<string | null>(null);

  /** Filter tel qu'envoyé au backend, recherche et tri compris. */
  const filter = useMemo<ApplicationFilter>(
    () => ({
      search,
      status: filters.status,
      contract: filters.contract,
      company_id: filters.company_id,
      city: filters.city,
      job_title: filters.job_title,
      start_date: filters.start_date,
      end_date: filters.end_date,
      sort,
      descending,
    }),
    [search, filters, sort, descending],
  );

  // Le Kanban affiche les quatre colonnes d'un coup : il demande une page assez large pour
  // les remplir, là où la List s'en tient à la densité choisie dans son pied.
  const page_size = view === "kanban" ? PAGE_SIZE * 4 : sizePage;

  const list = useQuery({
    queryKey: [...APPLICATIONS_KEY, "page", { page, page_size, filter }],
    queryFn: () => applicationService.listPage({ page, page_size, filter }),
  });

  // Compteurs des en-têtes de colonnes : calculés par SQLite sur tout le filtre, pas sur la
  // page affichée — une colonne annoncerait sinon « 3 » en contenant tout le pipeline.
  const breakdown = useQuery({
    queryKey: [...APPLICATIONS_KEY, "repartition", { filter }],
    queryFn: () => applicationService.breakdown(filter),
  });

  const invalider = useCallback(
    () => queryClient.invalidateQueries({ queryKey: APPLICATIONS_KEY }),
    [queryClient],
  );

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
    mutationFn: (input: NewApplication) => applicationService.create(input),
    onSuccess: async (application) => {
      await invalider();
      setSelectedId(application.id);
      notify({
        tone: "success",
        title: "Candidature enregistrée",
        detail: `${application.job_title} — ${application.company_name ?? ""}`,
      });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NewApplication }) =>
      applicationService.update(params.id, params.input),
    onSuccess: async (application) => {
      await invalider();
      notify({ tone: "success", title: "Candidature modifiée", detail: application.job_title });
    },
    onError: signalerEchec("Modification impossible"),
  });

  const changementStatus = useMutation({
    mutationFn: (params: { id: string; status: ApplicationStatus }) =>
      applicationService.changeStatus(params.id, params.status),
    onSuccess: invalider,
    // Pas de toast en cas de succès : le déplacement de la carte est déjà la confirmation
    // visible du geste. Un échec, lui, doit être annoncé — la carte reviendra à sa place.
    onError: signalerEchec("Changement de statut impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => applicationService.delete(id),
    onSuccess: async (_result, id) => {
      await invalider();
      if (selected_id === id) setSelectedId(null);
      notify({ tone: "success", title: "Candidature supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const rechercher = useCallback((value: string) => {
    setSearch(value);
    setPage(1);
  }, []);

  const appliquerFilters = useCallback((values: ApplicationFilterValues) => {
    setFilters(values);
    setPage(1);
  }, []);

  const resetFilters = useCallback(() => {
    setFilters(INITIAL_FILTER);
    setPage(1);
  }, []);

  /** Count de critères actifs, hors recherche libre, pour la pastille du bouton Filters. */
  const filtersActifs = useMemo(
    () =>
      [
        filters.status,
        filters.contract,
        filters.company_id,
        filters.city || null,
        filters.job_title || null,
        filters.start_date,
        filters.end_date,
      ].filter(Boolean).length,
    [filters],
  );

  /** Bascule la direction si l'on retrie la colonne courante, sinon trie la nouvelle. */
  const trierPar = useCallback(
    (column: ApplicationSort) => {
      if (column === sort) {
        setDescending((value) => !value);
      } else {
        setSort(column);
        setDescending(true);
      }
      setPage(1);
    },
    [sort],
  );

  const items: Application[] = list.data?.items ?? [];
  const selection = items.find((item) => item.id === selected_id) ?? null;

  return {
    view,
    items,
    breakdown: breakdown.data ?? { pending: 0, followed_up: 0, interview: 0, rejected: 0 },
    total: list.data?.total ?? 0,
    page,
    page_size,
    search,
    filters,
    filtersActifs,
    filter,
    sort,
    descending,
    selection,
    selected_id,
    isLoading: list.isPending,
    error: list.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending,

    setView,
    setPage,
    /** Change la densité de la vue List et revient à la première page. */
    setPageSize: useCallback((size: number) => {
      setSizePage(size);
      setPage(1);
    }, []),
    rechercher,
    appliquerFilters,
    resetFilters,
    trierPar,
    selectionner: setSelectedId,
    recharger: () => void list.refetch(),
    create: creation.mutateAsync,
    update: modification.mutateAsync,
    changeStatus: changementStatus.mutateAsync,
    delete: suppression.mutateAsync,
  };
}
