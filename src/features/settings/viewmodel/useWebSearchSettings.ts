import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { Settings } from "@/shared/types/generated/settings";
import { searxngUrlSchema } from "../model/web-search.schema";
import { settingsService } from "../services/settingsService";
import { SETTINGS_KEY } from "./useSettingsViewModel";

/**
 * Instance SearXNG préférée pour la recherche Web de « Compléter avec l'IA »
 * (Réglages → Intelligence artificielle). Vide : instances publiques de l'annuaire.
 */
export function useWebSearchSettings() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const settings = useQuery({ queryKey: SETTINGS_KEY, queryFn: settingsService.load });
  const save = useMutation({
    mutationFn: async (url: string | null) => {
      if (!settings.data) return;
      const next: Settings = { ...settings.data };
      if (url === null) delete next.searxng_url;
      else next.searxng_url = url;
      await settingsService.save(next);
    },
    onSuccess: (_, url) => {
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      notify({
        tone: "success",
        title: url === null ? "Instances publiques utilisées" : "Instance SearXNG enregistrée",
      });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Instance non enregistrée",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  return {
    current: settings.data?.searxng_url ?? "",
    ready: settings.data !== undefined,
    isSaving: save.isPending,
    /** Valide puis enregistre ; renvoie le message d'erreur à afficher, ou `null`. */
    submit: (value: string): string | null => {
      const parsed = searxngUrlSchema.safeParse(value);
      if (!parsed.success) return parsed.error.issues[0]?.message ?? "Adresse invalide";
      save.mutate(parsed.data);
      return null;
    },
  };
}
