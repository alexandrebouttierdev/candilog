import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { applicationService } from "../services/applicationService";
import type { Application,
  ApplicationFilter,
  NewApplication,
  ApplicationStatus, } from "@/shared/types/generated/applications";
import {
  EMPTY_FILTER,
  type ApplicationFilterValues,
} from "../model/schemas/application-filter.schema";
import type { ApplicationSort } from "@/shared/types/generated/applications";
import { PAGE_SIZE, type Page } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { Statuses } from "../model/statuses";
import { formatReference } from "../model/presentation";

/** Root des clés de cache de la feature. */
export const APPLICATIONS_KEY = ["candidatures"] as const;

/** Mode d'affichage du suivi. */
export type TrackingView = "kanban" | "list";

const INITIAL_KANBAN_PAGES: Record<ApplicationStatus, number> = {
  EN_ATTENTE: 1,
  RELANCEE: 1,
  ENTRETIEN: 1,
  REFUS: 1,
};

/** Lignes chargées par groupe de la liste, puis à chaque « Afficher plus ». */
export const GROUP_STEP = 50;

const INITIAL_GROUP_LIMITS: Record<ApplicationStatus, number> = {
  EN_ATTENTE: GROUP_STEP,
  RELANCEE: GROUP_STEP,
  ENTRETIEN: GROUP_STEP,
  REFUS: GROUP_STEP,
};

/**
 * Orchestration de l'écran Candidatures.
 *
 * Les deux vues interrogent **chaque statut séparément**, sur le même filtre : SQLite
 * applique le statut avant LIMIT/OFFSET, sans charger le pipeline complet. La liste v2
 * est groupée par statut — chaque groupe charge ses 50 premières lignes, puis 50 de plus
 * à la demande ; le Kanban pagine chaque colonne indépendamment.
 */
export function useApplicationsViewModel(controlledView?: TrackingView) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  const [searchParams, setSearchParams] = useSearchParams();
  const [viewState, setViewState] = useState<TrackingView>("kanban");
  // La vue suit l'onglet de la barre de titre quand l'écran est monté par une route (v2) ;
  // l'état local ne sert plus qu'aux écrans montés sans onglet.
  const view = controlledView ?? viewState;
  const [kanbanPages, setKanbanPages] = useState(INITIAL_KANBAN_PAGES);
  const [groupLimits, setGroupLimits] = useState(INITIAL_GROUP_LIMITS);
  const [search, setSearchState] = useState("");
  const searchQuery = useDebounce(search);
  const [filters, setFilters] = useState<ApplicationFilterValues>(EMPTY_FILTER);
  // Ordre dans un groupe ou une colonne : les plus récentes d'abord. La liste v2 n'a plus
  // d'en-têtes de colonnes triables — le groupement par statut les remplace.
  const sort: ApplicationSort = "date";
  const descending = true;

  // La fiche ouverte vit dans l'URL, pas dans un état local : le Dashboard ouvre une
  // candidature par `?id=<uuid>`, et le panneau survit ainsi à un rechargement comme à un
  // retour arrière. Aucune fiche n'est sélectionnée tant que le paramètre est absent.
  const selected_id = searchParams.get("id");

  const select = useCallback(
    (id: string | null) => {
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current);
          if (id === null) next.delete("id");
          else next.set("id", id);
          return next;
        },
        { replace: true },
      );
    },
    [setSearchParams],
  );

  /** Filtre tel qu'envoyé au backend, recherche et tri compris. */
  const filter = useMemo<ApplicationFilter>(
    () => ({ ...filters, search: searchQuery, sort, descending, ids: [] }),
    [searchQuery, filters, sort, descending],
  );

  // Une requête par statut : SQLite applique le statut avant LIMIT/OFFSET, ce qui évite
  // de charger le pipeline complet et permet à chaque groupe d'avancer à son propre rythme.
  const kanbanQueries = useQueries({
    queries: Statuses.map((status) => {
      const bornes =
        view === "kanban"
          ? { page: kanbanPages[status.value], page_size: PAGE_SIZE }
          : { page: 1, page_size: groupLimits[status.value] };
      return {
        queryKey: [...APPLICATIONS_KEY, view, status.value, { ...bornes, filter }],
        queryFn: () =>
          applicationService.listPage({ ...bornes, filter: { ...filter, status: [status.value] } }),
        enabled: filter.status.length === 0 || filter.status.includes(status.value),
      };
    }),
  });

  const kanbanColumns = useMemo<Record<ApplicationStatus, Page<Application>>>(() => {
    const columns = {} as Record<ApplicationStatus, Page<Application>>;
    Statuses.forEach((status, index) => {
      const query = kanbanQueries[index];
      columns[status.value] =
        query?.data ?? {
          items: [],
          total: 0,
          page: view === "kanban" ? kanbanPages[status.value] : 1,
          page_size: view === "kanban" ? PAGE_SIZE : groupLimits[status.value],
          total_pages: 1,
        };
    });
    return columns;
  }, [kanbanPages, groupLimits, kanbanQueries, view]);

  // Compteurs des en-têtes de colonnes : calculés par SQLite sur tout le filtre, pas sur la
  // page affichée — une colonne annoncerait sinon « 3 » en contenant tout le pipeline.
  const breakdown = useQuery({
    queryKey: [...APPLICATIONS_KEY, "repartition", { filter }],
    queryFn: () => applicationService.breakdown(filter),
  });
  // Total sans aucun critère, pour le décompte « 3 / 24 » de la barre d'outils. Même clé
  // que le décompte de la navigation : une seule requête pour les deux.
  const overall = useQuery({
    queryKey: [...APPLICATIONS_KEY, "navigation"],
    queryFn: () =>
      applicationService.breakdown({ ...EMPTY_FILTER, search: "", sort: "date", descending: true, ids: [] }),
  });

  // Le détail est chargé par son identifiant, pas cherché dans `items` : une fiche ouverte
  // depuis le Dashboard, ou restée sélectionnée après un changement de page, de filtre ou
  // de tri, n'appartient pas forcément à la page affichée.
  const detail = useQuery({
    queryKey: [...APPLICATIONS_KEY, "detail", selected_id],
    queryFn: () => applicationService.get(selected_id as string),
    enabled: selected_id !== null,
  });

  const invalidate = useCallback(
    () => queryClient.invalidateQueries({ queryKey: APPLICATIONS_KEY }),
    [queryClient],
  );

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
    mutationFn: (input: NewApplication) => applicationService.create(input),
    onSuccess: async (application) => {
      await invalidate();
      select(application.id);
      notify({
        tone: "success",
        title: "Candidature enregistrée",
        detail: `${application.job_title} — ${application.company_name ?? ""}`,
      });
    },
    onError: reportFailure("Enregistrement impossible"),
  });

  const modification = useMutation({
    mutationFn: (params: { id: string; input: NewApplication }) =>
      applicationService.update(params.id, params.input),
    onSuccess: async (application) => {
      await invalidate();
      notify({ tone: "success", title: "Candidature modifiée", detail: application.job_title });
    },
    onError: reportFailure("Modification impossible"),
  });

  const changementStatus = useMutation({
    mutationFn: (params: { id: string; status: ApplicationStatus }) =>
      applicationService.changeStatus(params.id, params.status),
    // Toast court `CAN-142 → Entretien` (`INTERACTIONS.md` §3.2) : le geste peut venir
    // d'une glisse, du menu ou du clavier, et la ligne peut changer de groupe hors écran.
    onSuccess: async (application: Application) => {
      await invalidate();
      const label = Statuses.find((status) => status.value === application.status)?.label ?? application.status;
      notify({ tone: "success", title: `${formatReference(application.reference_number)} → ${label}` });
    },
    onError: reportFailure("Changement de statut impossible"),
  });

  const changementStatusMultiple = useMutation({
    mutationFn: async (params: { ids: readonly string[]; status: ApplicationStatus }) => {
      for (const id of params.ids) {
        await applicationService.changeStatus(id, params.status);
      }
      return params.ids.length;
    },
    onSuccess: async (count, params) => {
      await invalidate();
      const label = Statuses.find((status) => status.value === params.status)?.label ?? params.status;
      notify({ tone: "success", title: `${count} candidature${count > 1 ? "s" : ""} → ${label}` });
    },
    onError: reportFailure("Changement de statut impossible"),
  });

  const suppression = useMutation({
    mutationFn: (id: string) => applicationService.delete(id),
    onSuccess: async (_result, id) => {
      if (selected_id === id) select(null);
      await invalidate();
      notify({ tone: "success", title: "Candidature supprimée" });
    },
    onError: reportFailure("Suppression impossible"),
  });

  const duplication = useMutation({
    mutationFn: (id: string) => applicationService.duplicate(id),
    onSuccess: async (application) => {
      await invalidate();
      select(application.id);
      notify({
        tone: "success",
        title: `${formatReference(application.reference_number)} créée par duplication`,
      });
    },
    onError: reportFailure("Duplication impossible"),
  });

  const suppressionMultiple = useMutation({
    mutationFn: async (ids: readonly string[]) => {
      for (const id of ids) {
        await applicationService.delete(id);
      }
      return ids.length;
    },
    onSuccess: async (count, ids) => {
      if (selected_id !== null && ids.includes(selected_id)) select(null);
      await invalidate();
      notify({
        tone: "success",
        title: count === 1 ? "Candidature supprimée" : `${count} candidatures supprimées`,
      });
    },
    onError: reportFailure("Suppression impossible"),
  });

  const exportCsv = useMutation({
    mutationFn: (exportFilter: ApplicationFilter) => applicationService.exportCsv(exportFilter),
    onSuccess: (rows) => {
      if (rows === null) return;
      notify({
        tone: "success",
        title: "Export terminé",
        detail: `${rows} candidature${rows > 1 ? "s" : ""} exportée${rows > 1 ? "s" : ""}.`,
      });
    },
    onError: reportFailure("Export impossible"),
  });

  const resetPaging = useCallback(() => {
    setKanbanPages(INITIAL_KANBAN_PAGES);
    setGroupLimits(INITIAL_GROUP_LIMITS);
  }, []);

  const setSearch = useCallback(
    (value: string) => {
      setSearchState(value);
      resetPaging();
    },
    [resetPaging],
  );

  const applyFilters = useCallback(
    (values: ApplicationFilterValues) => {
      setFilters(values);
      resetPaging();
    },
    [resetPaging],
  );

  const resetFilters = useCallback(() => {
    setFilters(EMPTY_FILTER);
    resetPaging();
  }, [resetPaging]);

  /** Nombre de critères actifs, hors recherche libre, pour la pastille du bouton Filtres. */
  const activeFilterCount = useMemo(
    () =>
      filters.status.length +
      filters.application_type.length +
      filters.channel.length +
      filters.contract_type_code.length +
      filters.professional_domain_id.length +
      filters.company_type_id.length +
      filters.company_size.length +
      filters.sector_id.length +
      filters.weekly_work_schedule.length +
      [
        filters.company_id,
        filters.city || null,
        filters.job_title || null,
        filters.start_date,
        filters.end_date,
        filters.min_weekly_hours,
        filters.max_weekly_hours,
      ].filter((critere) => critere !== null && critere !== undefined).length,
    [filters],
  );

  const items: Application[] = Array.from(
    new Map(
      Statuses.flatMap((status) => kanbanColumns[status.value].items).map((item) => [
        item.id,
        item,
      ]),
    ).values(),
  );
  const selection = selected_id === null ? null : (detail.data ?? null);
  const kanbanError = kanbanQueries.find((query) => query.error !== null)?.error ?? null;
  const totalKanban = breakdown.data
    ? breakdown.data.pending +
      breakdown.data.followed_up +
      breakdown.data.interview +
      breakdown.data.rejected
    : 0;

  // Un `?id=` pointant sur une candidature supprimée ou inconnue ne doit pas laisser
  // l'URL mentir : le paramètre est retiré et l'échec annoncé une seule fois.
  const detailError = detail.error;
  useEffect(() => {
    if (detailError === null) return;
    select(null);
    notify({
      tone: "error",
      title: "Candidature introuvable",
      detail: detailError instanceof AppError ? detailError.message : undefined,
    });
  }, [detailError, select, notify]);

  return {
    view,
    items,
    breakdown: breakdown.data ?? { pending: 0, followed_up: 0, interview: 0, rejected: 0 },
    kanbanColumns,
    kanbanPages,
    total: totalKanban,
    overallTotal: overall.data
      ? overall.data.pending + overall.data.followed_up + overall.data.interview + overall.data.rejected
      : null,
    groupLimits,
    search,
    setSearch,
    filters,
    activeFilterCount,
    filter,
    sort,
    descending,
    selection,
    selected_id,
    isLoading: breakdown.isPending || kanbanQueries.some((query) => query.isPending),
    isLoadingDetail: selected_id !== null && detail.isPending,
    error: breakdown.error ?? kanbanError,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending || suppressionMultiple.isPending,
    isExporting: exportCsv.isPending,

    setView: setViewState,
    setKanbanPage: useCallback((status: ApplicationStatus, nextPage: number) => {
      setKanbanPages((current) => ({ ...current, [status]: nextPage }));
    }, []),
    /** Charge 50 lignes de plus dans un groupe de la liste. */
    showMore: useCallback((status: ApplicationStatus) => {
      setGroupLimits((current) => ({ ...current, [status]: current[status] + GROUP_STEP }));
    }, []),
    applyFilters,
    resetFilters,
    select,
    /** Recharge la liste, les compteurs et la fiche ouverte, pas seulement la page. */
    reload: () => void invalidate(),
    create: creation.mutateAsync,
    update: modification.mutateAsync,
    changeStatus: changementStatus.mutateAsync,
    changeStatusMany: changementStatusMultiple.mutateAsync,
    delete: suppression.mutateAsync,
    duplicate: duplication.mutateAsync,
    deleteMany: suppressionMultiple.mutateAsync,
    exportCsv: exportCsv.mutateAsync,
  };
}
