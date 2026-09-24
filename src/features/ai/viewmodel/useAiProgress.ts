import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { AiProgress } from "../model/types";

/**
 * Progression de la génération `generation_id`.
 *
 * L'écoute commence au montage, pas au lancement : le backend annonce sa première étape
 * dès l'appel, souvent avant que l'écran connaisse l'identifiant et bien avant qu'un
 * abonnement asynchrone ouvert à ce moment-là soit actif. Le dernier événement reçu est
 * gardé avec son identifiant ; seul celui de l'opération courante est rendu.
 */
export function useAiProgress(generation_id: string | null) {
  const [latest, setLatest] = useState<AiProgress | null>(null);
  useEffect(() => {
    let cancelled = false;
    let dispose: (() => void) | undefined;
    void listen<AiProgress>("ia-progression", (event) => setLatest(event.payload))
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
  }, []);
  return generation_id !== null && latest?.generation_id === generation_id ? latest : null;
}
