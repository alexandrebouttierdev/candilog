import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { settingsService } from "../services/settingsService";
import type { LlmForm, Settings, ThemePref } from "@/shared/types/generated/settings";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

export const SETTINGS_KEY = ["parametres"] as const;
export const A_ABOUT_KEY = ["parametres", "a-propos"] as const;

function message(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

async function loadSettingsAndApplyTheme(
  setTheme: (theme: ThemePref) => void,
): Promise<Settings> {
  const settings = await settingsService.load();
  setTheme(settings.theme);
  applyTheme(settings.theme);
  return settings;
}

/**
 * Applique le thème persisté au démarrage, une fois le QueryClient disponible.
 * À monter dans AppProviders (pas dans App hors provider).
 */
export function useBootstrapTheme() {
  const setTheme = useUiStore((state) => state.setTheme);
  useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: () => loadSettingsAndApplyTheme(setTheme),
  });
}

/**
 * Préférence de thème pour la coque (rail) : mise à jour immédiate + persistance silencieuse.
 */
export function useThemePreference() {
  const queryClient = useQueryClient();
  const setTheme = useUiStore((state) => state.setTheme);
  const theme = useUiStore((state) => state.theme);

  const saveTheme = async (next: ThemePref) => {
    setTheme(next);
    applyTheme(next);
    try {
      const current =
        queryClient.getQueryData<Settings>(SETTINGS_KEY) ?? (await settingsService.load());
      const saved = await settingsService.save({ ...current, theme: next }, null);
      queryClient.setQueryData(SETTINGS_KEY, saved);
    } catch {
      /* Revue navigateur sans backend : le thème reste en session. */
    }
  };

  return { theme, saveTheme };
}

/** Chargement et enregistrement des réglages, thème compris. */
export function useSettingsViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const setTheme = useUiStore((state) => state.setTheme);
  const query = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: () => loadSettingsAndApplyTheme(setTheme),
  });
  const save = useMutation({
    mutationFn: ({ settings, apiKey }: { settings: Settings; apiKey: string | null }) =>
      settingsService.save(settings, apiKey),
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
  const clearApiKey = useMutation({
    mutationFn: settingsService.clearApiKey,
    onSuccess: () => {
      queryClient.setQueryData<Settings>(SETTINGS_KEY, (current) => {
        if (!current) return current;
        const id =
          typeof current.llm.provider === "string"
            ? current.llm.provider
            : "custom";
        const preset = current.llm_presets[id];
        return {
          ...current,
          llm: { ...current.llm, api_key_configured: false },
          llm_presets: preset
            ? {
                ...current.llm_presets,
                [id]: { ...preset, api_key_configured: false },
              }
            : current.llm_presets,
        };
      });
      notify({ tone: "success", title: "Clé API supprimée" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Suppression impossible",
        detail: message(error),
      });
    },
  });

  return {
    data: query.data,
    error: query.error,
    isLoading: query.isPending,
    isSaving: save.isPending,
    isClearingApiKey: clearApiKey.isPending,
    reload: () => void query.refetch(),
    save: (settings: Settings, apiKey: string | null) =>
      save.mutateAsync({ settings, apiKey }),
    clearApiKey: clearApiKey.mutateAsync,
    testConnection: (llm: LlmForm, apiKey: string | null) =>
      settingsService.testConnection(llm, apiKey),
    listModels: (llm: LlmForm, apiKey: string | null) =>
      settingsService.listModels(llm, apiKey),
  };
}
