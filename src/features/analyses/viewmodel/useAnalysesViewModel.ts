import { useCallback, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { analysesService } from "../services/analyses.service";
import type { NouvelleRelance } from "@/features/relances/services/relance.service";
import { relanceService } from "@/features/relances/services/relance.service";
import { RELANCES_KEY } from "@/features/calendrier/viewmodel/useCalendrierViewModel";
import type { Periode } from "@/shared/types/generated/analyses";
import { ANALYSES_KEY } from "./useTableauDeBordViewModel";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

/** Orchestration de l'écran Analyses et de sa période. */
export function useAnalysesViewModel() {
  const [periode, setPeriode] = useState<Periode>("trenteJours");
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const requete = useQuery({
    queryKey: [...ANALYSES_KEY, "detail", periode],
    queryFn: () => analysesService.charger(periode),
  });

  const creerRelance = useMutation({
    mutationFn: (input: NouvelleRelance) => relanceService.creer(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ANALYSES_KEY });
      await queryClient.invalidateQueries({ queryKey: RELANCES_KEY });
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

  const changerPeriode = useCallback((valeur: Periode) => setPeriode(valeur), []);

  return {
    periode,
    data: requete.data,
    isLoading: requete.isPending,
    isSaving: creerRelance.isPending,
    error: requete.error,
    changerPeriode,
    recharger: () => void requete.refetch(),
    creerRelance: creerRelance.mutateAsync,
  };
}
