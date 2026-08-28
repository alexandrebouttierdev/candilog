import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AiProgress } from "../model/types";

/** Abonnement événementiel Tauri, nettoyé à chaque changement d'opération. */
export function useAiProgress(generation_id: string | null) {
  const [state, setState] = useState<{ id: string; progress: AiProgress } | null>(null);
  useEffect(() => {
    if (!generation_id) return;
    let dispose: (() => void) | undefined;
    void listen<AiProgress>("ia-progression", (event) => {
      if (event.payload.generation_id === generation_id) setState({ id: generation_id, progress: event.payload });
    }).then((unlisten) => { dispose = unlisten; }).catch(() => { /* navigateur de revue sans runtime Tauri */ });
    return () => dispose?.();
  }, [generation_id]);
  return state?.id === generation_id ? state.progress : null;
}
