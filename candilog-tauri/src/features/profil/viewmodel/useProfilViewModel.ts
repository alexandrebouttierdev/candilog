import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { profilService } from "../services/profil.service";
import type { Profil, ProfilCharge } from "../services/profil.service";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const PROFIL_KEY = ["profil"] as const;

/** Chargement et remplacement atomique du profil complet. */
export function useProfilViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const requete = useQuery({ queryKey: PROFIL_KEY, queryFn: profilService.charger });
  const enregistrer = useMutation({
    mutationFn: (profil: Profil) => profilService.enregistrer(profil),
    onSuccess: (charge: ProfilCharge) => {
      queryClient.setQueryData(PROFIL_KEY, charge);
      notify({ tone: "success", title: "Profil enregistré" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Enregistrement impossible",
        detail: error instanceof AppError ? error.message : undefined,
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
