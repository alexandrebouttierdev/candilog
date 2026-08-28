import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ProgressionIa } from "../model/types";

/** Abonnement événementiel Tauri, nettoyé à chaque changement d'opération. */
export function useIaProgress(generationId: string | null) {
  const [state, setState] = useState<{ id: string; progress: ProgressionIa } | null>(null);
  useEffect(() => {
    if (!generationId) return;
    let dispose: (() => void) | undefined;
    void listen<ProgressionIa>("ia-progression", (event) => {
      if (event.payload.generationId === generationId) setState({ id: generationId, progress: event.payload });
    }).then((unlisten) => { dispose = unlisten; }).catch(() => { /* navigateur de revue sans runtime Tauri */ });
    return () => dispose?.();
  }, [generationId]);
  return state?.id === generationId ? state.progress : null;
}
