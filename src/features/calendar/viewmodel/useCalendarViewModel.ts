import { useCallback, useMemo, useState } from "react";
import { useMutation, useQueries, useQueryClient } from "@tanstack/react-query";
import { interviewService } from "@/features/interviews/services/interviewService";
import type { Interview, NewInterview } from "@/features/interviews/services/interviewService";
import { followUpService } from "@/features/followups/services/followUpService";
import type { NewFollowUp, FollowUp } from "@/features/followups/services/followUpService";
import { APPLICATIONS_KEY } from "@/features/applications";
import {
  gridBounds,
  decalerMonth,
  gridDuMonth,
  monthLabel,
} from "../model/month";
import {
  fromInterview,
  fromFollowUp,
  groupByDay,
  type CalendarEvent,
} from "../model/event";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Racines de cache des deux entités affichées. */
export const INTERVIEWS_KEY = ["entretiens"] as const;
export const FOLLOW_UPS_KEY = ["relances"] as const;

/**
 * Orchestration de l'écran Tracking → Calendar.
 *
 * Les deux entités sont chargées par `useQueries` sur **les bornes de la grille** et non du
 * mois : la grille déborde sur les mois voisins, et interroger le seul mois laisserait ces
 * cases vides alors qu'elles portent des événements.
 */
export function useCalendarViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);

  // Année et mois dans un seul état : ils changent toujours ensemble, et les séparer
  // obligerait à lire l'un pour calculer l'autre, ce qui fige leurs valeurs dans la
  // fermeture du gestionnaire.
  const [period, setPeriod] = useState(() => {
    const now = new Date();
    return { year: now.getFullYear(), month: now.getMonth() };
  });
  const { year, month } = period;

  const bounds = useMemo(() => gridBounds(year, month), [year, month]);
  const cells = useMemo(() => gridDuMonth(year, month), [year, month]);

  const [interviews, follow_ups] = useQueries({
    queries: [
      {
        queryKey: [...INTERVIEWS_KEY, "entre", bounds],
        // Les entretiens portent une heure : les bornes doivent couvrir la journée entière,
        // sans quoi un entretien de 14 h le dernier jour affiché serait exclu.
        queryFn: () =>
          interviewService.listBetween(`${bounds.from}T00:00:00`, `${bounds.to}T23:59:59`),
      },
      {
        queryKey: [...FOLLOW_UPS_KEY, "entre", bounds],
        queryFn: () => followUpService.listBetween(bounds.from, bounds.to),
      },
    ],
  });

  const events = useMemo<CalendarEvent[]>(
    () => [
      ...(interviews.data ?? []).map(fromInterview),
      ...(follow_ups.data ?? []).map(fromFollowUp),
    ],
    [interviews.data, follow_ups.data],
  );

  const parDay = useMemo(() => groupByDay(events), [events]);

  // Les compteurs d'en-tête ne comptent que le mois affiché, pas la grille entière : ils
  // sont posés à côté de « août 2026 », et y inclure les débordements sur juillet et
  // septembre ferait mentir le chiffre sur le mois qu'il annonce.
  const prefixe_du_mois = `${year}-${String(month + 1).padStart(2, "0")}`;
  const in_month = useMemo(
    () => events.filter((event) => event.day.startsWith(prefixe_du_mois)),
    [events, prefixe_du_mois],
  );

  const invalider = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: INTERVIEWS_KEY });
    await queryClient.invalidateQueries({ queryKey: FOLLOW_UPS_KEY });
    // Save un entretien fait avancer la candidature : sans cette invalidation, le
    // Kanban resterait sur le statut précédent jusqu'au prochain rechargement.
    await queryClient.invalidateQueries({ queryKey: APPLICATIONS_KEY });
  }, [queryClient]);

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

  const saveInterview = useMutation({
    mutationFn: (params: { id: string | null; input: NewInterview }) =>
      interviewService.save(params.id, params.input),
    onSuccess: async (interview) => {
      await invalider();
      notify({
        tone: "success",
        title: "Entretien enregistré",
        detail: `${interview.application_job_title ?? ""} — la candidature passe en « Entretien ».`,
      });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const deleteInterview = useMutation({
    mutationFn: (id: string) => interviewService.delete(id),
    onSuccess: async () => {
      await invalider();
      notify({ tone: "success", title: "Entretien supprimé" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const saveFollowUp = useMutation({
    mutationFn: (params: { id: string | null; input: NewFollowUp }) =>
      params.id === null
        ? followUpService.create(params.input)
        : followUpService.update(params.id, params.input),
    onSuccess: async () => {
      await invalider();
      notify({ tone: "success", title: "Relance enregistrée" });
    },
    onError: signalerEchec("Enregistrement impossible"),
  });

  const deleteFollowUp = useMutation({
    mutationFn: (id: string) => followUpService.delete(id),
    onSuccess: async () => {
      await invalider();
      notify({ tone: "success", title: "Relance supprimée" });
    },
    onError: signalerEchec("Suppression impossible"),
  });

  const naviguer = useCallback((pas: number) => {
    setPeriod((courante) => decalerMonth(courante.year, courante.month, pas));
  }, []);

  const revenirToday = useCallback(() => {
    const today = new Date();
    setPeriod({ year: today.getFullYear(), month: today.getMonth() });
  }, []);

  const allerA = useCallback((cibleYear: number, cibleMonth: number) => {
    setPeriod({ year: cibleYear, month: cibleMonth });
  }, []);

  /** Interview ou relance derrière un événement, pour rouvrir la bonne modale. */
  const interviewDe = useCallback(
    (id: string): Interview | null => interviews.data?.find((item) => item.id === id) ?? null,
    [interviews.data],
  );
  const followUpDe = useCallback(
    (id: string): FollowUp | null => follow_ups.data?.find((item) => item.id === id) ?? null,
    [follow_ups.data],
  );

  return {
    year,
    month,
    label: monthLabel(year, month),
    cells,
    parDay,
    countInterviews: in_month.filter((e) => e.kind === "entretien").length,
    countFollowUps: in_month.filter((e) => e.kind === "relance").length,
    isLoading: interviews.isPending || follow_ups.isPending,
    error: interviews.error ?? follow_ups.error,
    isSaving: saveInterview.isPending || saveFollowUp.isPending,
    isDeleting: deleteInterview.isPending || deleteFollowUp.isPending,

    naviguer,
    allerA,
    revenirToday,
    recharger: () => {
      void interviews.refetch();
      void follow_ups.refetch();
    },
    interviewDe,
    followUpDe,
    saveInterview: saveInterview.mutateAsync,
    deleteInterview: deleteInterview.mutateAsync,
    saveFollowUp: saveFollowUp.mutateAsync,
    deleteFollowUp: deleteFollowUp.mutateAsync,
  };
}
