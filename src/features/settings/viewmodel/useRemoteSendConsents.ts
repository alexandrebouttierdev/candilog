import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { OLLAMA_PROVIDER, PROVIDERS } from "../model/providers";
import { settingsService } from "../services/settingsService";
import { SETTINGS_KEY } from "./useSettingsViewModel";

/**
 * Services distants dispensés de la confirmation de premier envoi (D4), et leur remise à
 * zéro depuis Réglages → Intelligence artificielle.
 */
export function useRemoteSendConsents() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const settings = useQuery({ queryKey: SETTINGS_KEY, queryFn: settingsService.load });
  const consents = settings.data?.remote_send_consents ?? [];
  const reset = useMutation({
    mutationFn: async () => {
      if (!settings.data) return;
      await settingsService.save({ ...settings.data, remote_send_consents: [] });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      notify({ tone: "success", title: "Confirmations rétablies" });
    },
    onError: (error: unknown) => {
      notify({
        tone: "error",
        title: "Remise à zéro impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    },
  });

  return {
    /** Libellés des services dispensés, dans l'ordre enregistré. */
    labels: consents.map((id) => [...PROVIDERS, OLLAMA_PROVIDER].find((provider) => provider.id === id)?.label ?? id),
    isResetting: reset.isPending,
    reset: () => reset.mutate(),
  };
}
