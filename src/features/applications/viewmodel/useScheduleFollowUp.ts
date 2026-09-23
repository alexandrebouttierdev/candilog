import { useMutation, useQueryClient } from "@tanstack/react-query";
import { FOLLOW_UPS_KEY, followUpService } from "@/features/followups";
import type { NewFollowUp } from "@/features/followups";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { APPLICATIONS_KEY } from "./useApplicationsViewModel";

/** Racine des analyses (Aujourd'hui) : même valeur que `ANALYTICS_KEY`, sans dépendre de la feature. */
const ANALYSES = ["analyses"] as const;

/**
 * Programme une relance depuis l'écran Candidatures (`R`).
 *
 * Invalide aussi les candidatures : la pastille « ↻ 17-09 » de la ligne et le champ
 * Relance de l'inspecteur viennent de la prochaine relance calculée par SQLite.
 */
export function useScheduleFollowUp() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  return useMutation({
    mutationFn: (input: NewFollowUp) => followUpService.create(input),
    onSuccess: async (followUp) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: FOLLOW_UPS_KEY }),
        queryClient.invalidateQueries({ queryKey: APPLICATIONS_KEY }),
        queryClient.invalidateQueries({ queryKey: ANALYSES }),
      ]);
      const date = followUp.follow_up_date;
      notify({ tone: "success", title: `Relance programmée pour le ${date.slice(8, 10)}-${date.slice(5, 7)}` });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Relance impossible à programmer",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });
}
