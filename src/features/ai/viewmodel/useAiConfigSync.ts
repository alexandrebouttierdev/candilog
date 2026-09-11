import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { SETTINGS_KEY, settingsService } from "@/features/settings";
import { useAiRequiredStore } from "./ai-required-store";

/**
 * Maintient le garde-fou IA aligné sur les réglages chargés.
 *
 * La requête vit ici et non dans le composant : une vue n'appelle pas un service
 * directement (`docs/CODE_RULES.md` §4). Elle partage la clé de cache des réglages, donc
 * toute mutation les invalidant rafraîchit l'état synchrone que lit `useAiOperation`.
 */
export function useAiConfigSync() {
  const setLlm = useAiRequiredStore((state) => state.setLlm);
  const query = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: settingsService.load,
  });

  useEffect(() => {
    setLlm(query.data?.llm ?? null);
  }, [query.data?.llm, setLlm]);
}
