import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { applicationService } from "../services/applicationService";
import type {
  Application,
  ApplicationFilter,
  NewApplication,
  ApplicationStatus,
} from "../services/applicationService";
import {
  FILTER_VIDE,
  type ApplicationFilterValues,
} from "../model/schemas/application-filter.schema";
import type { ApplicationSort } from "@/shared/types/generated/applications";
import { KANBAN_PAGE_SIZE, PAGE_SIZE } from "@/shared/types/page";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { useDebounce } from "@/shared/hooks/useDebounce";

/** Root des clés de cache de la feature. */
export const APPLICATIONS_KEY = ["candidatures"] as const;

/** Mode d'affichage du suivi. */
export type TrackingView = "kanban" | "liste";

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

  const [searchParams, setSearchParams] = useSearchParams();
  const [view, setVue] = useState<TrackingView>("kanban");
  const [page, setPage] = useState(1);
  const [sizePage, setSizePage] = useState<number>(PAGE_SIZE);
  const [search, setSearch] = useState("");
  const searchQuery = useDebounce(search);
  const [filters, setFilters] = useState<ApplicationFilterValues>(FILTER_VIDE);
  const [sort, setSort] = useState<ApplicationSort>("date");
  const [descending, setDescending] = useState(true);

  // La fiche ouverte vit dans l'URL, pas dans un état local : le Dashboard ouvre une
  // candidature par `?fiche=<id>`, et le panneau survit ainsi à un rechargement comme à un
  // retour arrière. Aucune fiche n'est sélectionnée tant que le paramètre est absent.
  const selected_id = searchParams.get("fiche");

  const selectionner = useCallback(
    (id: string | null) => {
      setSearchParams(
        (actuel) => {
          const suivant = new URLSearchParams(actuel);
          if (id === null) suivant.delete("fiche");
          else suivant.set("fiche", id);
          return suivant;
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

  // Le Kanban affiche les quatre colonnes d'un coup : une page de 32 lignes tronquait
  // silencieusement le pipeline dès ~30 candidatures.
  const page_size = view === "kanban" ? KANBAN_PAGE_SIZE : sizePage;

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

  // Le détail est chargé par son identifiant, pas cherché dans `items` : une fiche ouverte
  // depuis le Dashboard, ou restée sélectionnée après un changement de page, de filtre ou
  // de tri, n'appartient pas forcément à la page affichée.
  const detail = useQuery({
    queryKey: [...APPLICATIONS_KEY, "detail", selected_id],
    queryFn: () => applicationService.get(selected_id as string),
    enabled: selected_id !== null,
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
      selectionner(application.id);
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
      if (selected_id === id) selectionner(null);
      await invalider();
      notify({ tone: "success", title: "Candidature supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const suppressionMultiple = useMutation({
    mutationFn: async (ids: readonly string[]) => {
      for (const id of ids) {
        await applicationService.delete(id);
      }
      return ids.length;
    },
    onSuccess: async (count, ids) => {
      if (selected_id !== null && ids.includes(selected_id)) selectionner(null);
      await invalider();
      notify({
        tone: "success",
        title: count === 1 ? "Candidature supprimée" : `${count} candidatures supprimées`,
      });
    },
    onError: signalerEchec("Suppression impossible"),
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
    onError: signalerEchec("Export impossible"),
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
    setFilters(FILTER_VIDE);
    setPage(1);
  }, []);

  /** Nombre de critères actifs, hors recherche libre, pour la pastille du bouton Filtres. */
  const filtersActifs = useMemo(
    () =>
      filters.status.length +
      filters.application_type.length +
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
  const selection = selected_id === null ? null : (detail.data ?? null);

  // Un `?fiche=` pointant sur une candidature supprimée ou inconnue ne doit pas laisser
  // l'URL mentir : le paramètre est retiré et l'échec annoncé une seule fois.
  const detailError = detail.error;
  useEffect(() => {
    if (detailError === null) return;
    selectionner(null);
    notify({
      tone: "error",
      title: "Candidature introuvable",
      detail: detailError instanceof AppError ? detailError.message : undefined,
    });
  }, [detailError, selectionner, notify]);

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
    isLoadingDetail: selected_id !== null && detail.isPending,
    error: list.error,
    isSaving: creation.isPending || modification.isPending,
    isDeleting: suppression.isPending || suppressionMultiple.isPending,
    isExporting: exportCsv.isPending,

    /** Change de vue et revient à la première page : le Kanban charge tout le pipeline. */
    setView: useCallback((suivante: TrackingView) => {
      setVue(suivante);
      setPage(1);
    }, []),
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
    selectionner,
    /** Recharge la liste, les compteurs et la fiche ouverte, pas seulement la page. */
    recharger: () => void invalider(),
    create: creation.mutateAsync,
    update: modification.mutateAsync,
    changeStatus: changementStatus.mutateAsync,
    delete: suppression.mutateAsync,
    deleteMany: suppressionMultiple.mutateAsync,
    exportCsv: exportCsv.mutateAsync,
  };
}
