import { useRef, useState } from "react";
import { useElapsedClock } from "@/shared/hooks/useElapsedClock";

/**
 * Chronomètre d'un traitement IA : temps écoulé pendant, durée totale après.
 *
 * Aucun pourcentage n'est affiché parce qu'aucun n'est connu : la durée d'une génération
 * dépend du fournisseur et du modèle, et un chiffre inventé se bloque toujours au même
 * palier. Le temps écoulé, lui, est vrai.
 *
 * L'instant de départ est aussi tenu dans une `ref` : `stop()` est appelé depuis la même
 * fermeture asynchrone que `start()`, où l'état du rendu précédent est encore visible.
 */
export function useAiTimer(running: boolean) {
  const startedAtRef = useRef<number | null>(null);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [durationMs, setDurationMs] = useState<number | null>(null);
  const elapsedMs = useElapsedClock(running, startedAt);

  return {
    elapsedMs,
    durationMs,
    /** Démarre le décompte et efface la durée du traitement précédent. */
    start: () => {
      const now = Date.now();
      startedAtRef.current = now;
      setStartedAt(now);
      setDurationMs(null);
    },
    /** Fige la durée totale et la retourne ; sans départ connu, rien n'est affiché. */
    stop: (): number | null => {
      const debut = startedAtRef.current;
      const duree = debut === null ? null : Date.now() - debut;
      setDurationMs(duree);
      return duree;
    },
  };
}
