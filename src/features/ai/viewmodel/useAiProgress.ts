import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { aiService } from "../services/aiService";
import type { AiProgress } from "../model/types";

/** Abonnement événementiel Tauri, nettoyé à chaque changement d'opération. */
export function useAiProgress(generation_id: string | null) {
  const [state, setState] = useState<{ id: string; progress: AiProgress } | null>(null);
  useEffect(() => {
    if (!generation_id) return;
    let cancelled = false;
    let dispose: (() => void) | undefined;
    void listen<AiProgress>("ia-progression", (event) => {
      if (event.payload.generation_id === generation_id) setState({ id: generation_id, progress: event.payload });
    })
      .then((unlisten) => {
        if (cancelled) {
          unlisten();
          return;
        }
        dispose = unlisten;
      })
      .catch(() => {
        /* navigateur de revue sans runtime Tauri */
      });
    return () => {
      cancelled = true;
      dispose?.();
    };
  }, [generation_id]);
  return state?.id === generation_id ? state.progress : null;
}

/** Annule la génération en cours uniquement si l'écran est quitté pendant l'appel.
 *
 * Le cleanup d'un effet dépendant de `generation_id` s'exécutait aussi quand
 * l'opération **réussissait** (`setOperation(null)`) et envoyait un `ai_cancel` parasite.
 */
export function useCancelAiOnUnmount(generation_id: string | null) {
  const generationIdRef = useRef(generation_id);
  useEffect(() => {
    generationIdRef.current = generation_id;
  }, [generation_id]);
  useEffect(() => {
    return () => {
      const id = generationIdRef.current;
      if (id) void aiService.cancel(id);
    };
  }, []);
}
