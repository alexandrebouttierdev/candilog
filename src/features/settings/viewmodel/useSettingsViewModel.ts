import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { settingsService } from "../services/settingsService";
import type { Settings } from "../services/settingsService";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const SETTINGS_KEY = ["parametres"] as const;
export const A_ABOUT_KEY = ["parametres", "a-propos"] as const;

function message(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Chargement et enregistrement des réglages, thème compris. */
export function useSettingsViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const setTheme = useUiStore((state) => state.setTheme);
  const query = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: async () => {
      const settings = await settingsService.load();
      setTheme(settings.theme);
      applyTheme(settings.theme);
      return settings;
    },
  });
  const save = useMutation({
    mutationFn: (settings: Settings) => settingsService.save(settings),
    onSuccess: (backups) => {
      queryClient.setQueryData(SETTINGS_KEY, backups);
      setTheme(backups.theme);
      applyTheme(backups.theme);
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
    data: query.data,
    error: query.error,
    isLoading: query.isPending,
    isSaving: save.isPending,
    recharger: () => void query.refetch(),
    save: save.mutateAsync,
  };
}
