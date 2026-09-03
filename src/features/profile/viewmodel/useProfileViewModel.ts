import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { profileService } from "../services/profileService";
import type { ImportProfileRequest, Profile, ProfilePayload } from "../services/profileService";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const PROFILE_KEY = ["profil"] as const;

/** Clé de la photo, distincte du profil : elle transporte une image, pas des champs. */
export const PROFILE_PHOTO_KEY = [...PROFILE_KEY, "photo"] as const;

/**
 * Photo du profil, en `data:` URL.
 *
 * Requête séparée du profil : l'image pèse plusieurs dizaines de kilo-octets et n'intéresse
 * que les écrans qui l'affichent. Le reste de l'application charge le profil sans elle.
 */
export function useProfilePhoto() {
  return useQuery({ queryKey: PROFILE_PHOTO_KEY, queryFn: profileService.photo });
}

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

  /** Toute écriture de la photo rafraîchit le profil et l'image elle-même. */
  const appliquerPhoto = async (payload: ProfilePayload | null) => {
    if (payload === null) return;
    queryClient.setQueryData(PROFILE_KEY, payload);
    await queryClient.invalidateQueries({ queryKey: PROFILE_PHOTO_KEY });
  };

  const setPhoto = useMutation({
    mutationFn: () => profileService.setPhoto(),
    onSuccess: async (payload) => {
      await appliquerPhoto(payload);
      if (payload !== null) notify({ tone: "success", title: "Photo enregistrée" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Photo non enregistrée",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  const removePhoto = useMutation({
    mutationFn: () => profileService.removePhoto(),
    onSuccess: async (payload) => {
      await appliquerPhoto(payload);
      notify({ tone: "success", title: "Photo supprimée" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Suppression impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  const reset = useMutation({
    mutationFn: () => profileService.reset(),
    onSuccess: async (payload) => {
      await appliquerPhoto(payload);
      notify({
        tone: "success",
        title: "Profil réinitialisé",
        detail: "Vos candidatures et vos autres données sont intactes.",
      });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Réinitialisation impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  return {
    data: query.data,
    error: query.error,
    isLoading: query.isPending,
    isSaving: save.isPending || applyImport.isPending,
    isPhotoBusy: setPhoto.isPending || removePhoto.isPending,
    isResetting: reset.isPending,
    recharger: () => void query.refetch(),
    save: save.mutateAsync,
    applyImport: applyImport.mutateAsync,
    setPhoto: setPhoto.mutateAsync,
    removePhoto: removePhoto.mutateAsync,
    reset: reset.mutateAsync,
  };
}
