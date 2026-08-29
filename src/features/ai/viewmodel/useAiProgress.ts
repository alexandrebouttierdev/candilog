import { useEffect, useState } from "react";
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

/** Annule la génération en cours si l'écran est quitté pendant l'appel IA. */
export function useCancelAiOnUnmount(generation_id: string | null) {
  useEffect(() => {
    return () => {
      if (generation_id) void aiService.cancel(generation_id);
    };
  }, [generation_id]);
}
