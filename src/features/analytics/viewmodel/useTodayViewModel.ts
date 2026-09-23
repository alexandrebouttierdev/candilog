import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { analyticsService } from "../services/analyticsService";
import { ANALYTICS_KEY } from "./analyticsKeys";
import { APPLICATIONS_KEY, EMPTY_FILTER, applicationService } from "@/features/applications";
import type { ApplicationFilter } from "@/features/applications";
import { FOLLOW_UPS_KEY, followUpService } from "@/features/followups";
import type { FollowUp, NewFollowUp } from "@/features/followups";
import type { AgendaItem } from "@/shared/types/generated/analytics";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { splitAgenda } from "../model/agenda";
import { useAgenda } from "./useAgenda";

/** Toutes les candidatures : la répartition de la colonne Situation est un total. */
const TOUTES: ApplicationFilter = { ...EMPTY_FILTER, search: "", sort: "date", descending: true, ids: [] };

/**
 * Écran Aujourd'hui : les échéances en trois horizons, et la colonne Situation (répartition
 * des statuts, 30 derniers jours, candidatures sans réponse).
 *
 * « Faire » marque une relance envoyée. Toutes les requêtes sont rangées sous les clés
 * racines des features concernées, pour qu'une écriture ailleurs mette l'écran à jour.
 */
export function useTodayViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const agenda = useAgenda();
  const today = agenda.today;

  const dashboard = useQuery({ queryKey: [...ANALYTICS_KEY, "tableau-de-bord"], queryFn: analyticsService.dashboard });
  const analytics = useQuery({ queryKey: [...ANALYTICS_KEY, "tout"], queryFn: () => analyticsService.load("tout") });
  const breakdown = useQuery({
    queryKey: [...APPLICATIONS_KEY, "navigation"],
    queryFn: () => applicationService.breakdown(TOUTES),
  });

  const invalidate = () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ANALYTICS_KEY }),
      queryClient.invalidateQueries({ queryKey: FOLLOW_UPS_KEY }),
      queryClient.invalidateQueries({ queryKey: APPLICATIONS_KEY }),
    ]);

  const done = useMutation({
    mutationFn: (id: string) => followUpService.setDone(id, true),
    onSuccess: async (followUp: FollowUp) => {
      await invalidate();
      notify({ tone: "success", title: "Relance marquée faite", detail: followUp.application_job_title ?? undefined });
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Relance impossible à marquer", detail: error instanceof AppError ? error.message : undefined }),
  });

  const reschedule = useMutation({
    mutationFn: ({ id, input }: { id: string; input: NewFollowUp }) => followUpService.update(id, input),
    onSuccess: async (followUp: FollowUp) => {
      await invalidate();
      const date = followUp.follow_up_date;
      notify({ tone: "success", title: `Relance reportée au ${date.slice(8, 10)}-${date.slice(5, 7)}` });
    },
    onError: (error: unknown) =>
      notify({ tone: "error", title: "Report impossible", detail: error instanceof AppError ? error.message : undefined }),
  });

  /** Relance complète d'une échéance : le report la relit pour ne pas écraser ses notes. */
  const loadFollowUp = async (item: AgendaItem): Promise<FollowUp | null> => {
    const day = item.date.slice(0, 10);
    const found = await followUpService.listBetween(day, day);
    return found.find((followUp) => followUp.id === item.id) ?? null;
  };

  const items = agenda.data ?? [];
  const b = breakdown.data;
  return {
    today,
    groups: splitAgenda(items, today),
    total: items.length,
    metrics: dashboard.data?.metrics ?? null,
    silent: analytics.data?.to_follow_up ?? [],
    breakdown: b ?? null,
    applicationsTotal: b ? b.pending + b.followed_up + b.interview + b.rejected : null,
    isLoading: agenda.isPending,
    error: agenda.error,
    reload: () => void invalidate(),
    markDone: done.mutateAsync,
    isMarking: done.isPending,
    loadFollowUp,
    reschedule: (id: string, input: NewFollowUp) => reschedule.mutateAsync({ id, input }).then(() => undefined),
    isRescheduling: reschedule.isPending,
  };
}
