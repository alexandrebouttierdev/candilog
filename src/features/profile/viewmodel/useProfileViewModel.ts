import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { profileService } from "../services/profileService";
import type { ImportProfileRequest, Profile, ProfilePayload } from "@/shared/types/generated/profile";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const PROFILE_KEY = ["profil"] as const;

/**
 * Marque d'une requête calculée **depuis** le profil (le CV de base composé en Rust, par
 * exemple) : toute écriture du profil la périme.
 *
 * Une marque plutôt que la clé de la requête : le profil n'a pas à connaître les features
 * qui le lisent — importer la clé du CV de base refermait un cycle `documents ↔ profile`.
 * C'est la feature consommatrice qui pose `meta: PROFILE_DERIVED_META` sur sa requête.
 */
export const PROFILE_DERIVED_META = { derivedFromProfile: true } as const;

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

  /**
   * Les requêtes calculées depuis le profil (`PROFILE_DERIVED_META`) — le CV de base,
   * composé en Rust et mis en cache — sont périmées par toute écriture du profil. Sans
   * cela, revenir sur l'écran dans les trente secondes (`staleTime`) y affichait l'ancienne
   * feuille sous la mention « composé depuis votre profil », avec une colonne de gauche —
   * elle, rafraîchie — qui la contredisait.
   */
  const perimerDerives = () =>
    queryClient.invalidateQueries({ predicate: (query) => query.meta?.["derivedFromProfile"] === true });

  const save = useMutation({
    mutationFn: (profile: Profile) => profileService.save(profile),
    onSuccess: async (payload: ProfilePayload) => {
      queryClient.setQueryData(PROFILE_KEY, payload);
      await perimerDerives();
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
      await perimerDerives();
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
    // Chemin partagé par la photo et la réinitialisation : la seconde vide le profil, donc
    // la composition en cache.
    await perimerDerives();
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
    reload: () => void query.refetch(),
    save: save.mutateAsync,
    applyImport: applyImport.mutateAsync,
    setPhoto: setPhoto.mutateAsync,
    removePhoto: removePhoto.mutateAsync,
    reset: reset.mutateAsync,
  };
}
