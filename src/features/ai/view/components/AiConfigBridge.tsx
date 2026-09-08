import { useAiConfigSync } from "../../viewmodel/useAiConfigSync";

/**
 * Monte la synchronisation du garde-fou IA, une fois, dans la coque.
 *
 * Composant sans rendu : il n'existe que pour rattacher `useAiConfigSync` au cycle de vie
 * de l'application.
 */
export function AiConfigBridge() {
  useAiConfigSync();
  return null;
}
