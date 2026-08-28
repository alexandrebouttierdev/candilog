import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { parametresService } from "../services/parametres.service";
import type { Parametres } from "../services/parametres.service";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const PARAMETRES_KEY = ["parametres"] as const;
export const A_PROPOS_KEY = ["parametres", "a-propos"] as const;

function message(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Chargement et enregistrement des réglages, thème compris. */
export function useParametresViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const setTheme = useUiStore((state) => state.setTheme);
  const requete = useQuery({
    queryKey: PARAMETRES_KEY,
    queryFn: async () => {
      const parametres = await parametresService.charger();
      setTheme(parametres.theme);
      applyTheme(parametres.theme);
      return parametres;
    },
  });
  const enregistrer = useMutation({
    mutationFn: (parametres: Parametres) => parametresService.enregistrer(parametres),
    onSuccess: (sauvegardes) => {
      queryClient.setQueryData(PARAMETRES_KEY, sauvegardes);
      setTheme(sauvegardes.theme);
      applyTheme(sauvegardes.theme);
      notify({ tone: "success", title: "Réglages enregistrés" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Enregistrement impossible",
        detail: message(error),
      });
    },
  });

  return {
    data: requete.data,
    error: requete.error,
    isLoading: requete.isPending,
    isSaving: enregistrer.isPending,
    recharger: () => void requete.refetch(),
    enregistrer: enregistrer.mutateAsync,
  };
}
