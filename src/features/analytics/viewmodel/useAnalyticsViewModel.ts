import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { analyticsService } from "../services/analyticsService";
import { FOLLOW_UPS_KEY, followUpService, type NewFollowUp } from "@/features/followups";
import type { Period } from "@/shared/types/generated/analytics";
import { ANALYTICS_KEY } from "./useDashboardViewModel";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Orchestration de l'écran Analytics et de sa période. */
export function useAnalyticsViewModel() {
  const [period, setPeriod] = useState<Period>("trente_days");
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const query = useQuery({
    queryKey: [...ANALYTICS_KEY, "detail", period],
    queryFn: () => analyticsService.load(period),
  });

  const createFollowUp = useMutation({
    mutationFn: (input: NewFollowUp) => followUpService.create(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ANALYTICS_KEY });
      await queryClient.invalidateQueries({ queryKey: FOLLOW_UPS_KEY });
      notify({ tone: "success", title: "Relance programmée" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Relance impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });
  const exportCsv = useMutation({
    mutationFn: () => analyticsService.exportCsv(period),
    onSuccess: (exported) => {
      if (exported) notify({ tone: "success", title: "Analyses exportées" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Export impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  const changePeriod = useCallback((value: Period) => setPeriod(value), []);

  return {
    period,
    data: query.data,
    isLoading: query.isPending,
    isSaving: createFollowUp.isPending,
    isExporting: exportCsv.isPending,
    error: query.error,
    changePeriod,
    recharger: () => void query.refetch(),
    createFollowUp: createFollowUp.mutateAsync,
    exportCsv: exportCsv.mutateAsync,
  };
}
