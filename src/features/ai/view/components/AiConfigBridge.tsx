import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { settingsService } from "@/features/settings/services/settingsService";
import { SETTINGS_KEY } from "@/features/settings/viewmodel/useSettingsViewModel";
import { useAiRequiredStore } from "../../viewmodel/ai-required-store";

/**
 * Maintient le garde-fou IA aligné sur les réglages chargés.
 *
 * Monté une fois dans la coque : toute mutation des réglages invalide `SETTINGS_KEY` et
 * rafraîchit automatiquement l'état synchrone lu par `useAiOperation`.
 */
export function AiConfigBridge() {
  const setLlm = useAiRequiredStore((state) => state.setLlm);
  const query = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: settingsService.load,
  });

  useEffect(() => {
    setLlm(query.data?.llm ?? null);
  }, [query.data?.llm, setLlm]);

  return null;
}
