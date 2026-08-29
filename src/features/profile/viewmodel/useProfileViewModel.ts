import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { profileService } from "../services/profileService";
import type { ImportProfileRequest, Profile, ProfilePayload } from "../services/profileService";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const PROFILE_KEY = ["profil"] as const;

/** Chargement et remplacement atomique du profil complet. */
export function useProfileViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const query = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const save = useMutation({
    mutationFn: (profile: Profile) => profileService.save(profile),
    onSuccess: (payload: ProfilePayload) => {
      queryClient.setQueryData(PROFILE_KEY, payload);
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
  const applyImport = useMutation({
    mutationFn: (request: ImportProfileRequest) => profileService.applyImport(request),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: PROFILE_KEY });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Import impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  return {
    data: query.data,
    error: query.error,
    isLoading: query.isPending,
    isSaving: save.isPending || applyImport.isPending,
    recharger: () => void query.refetch(),
    save: save.mutateAsync,
    applyImport: applyImport.mutateAsync,
  };
}
