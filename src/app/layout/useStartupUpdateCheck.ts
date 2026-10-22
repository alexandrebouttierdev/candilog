import { useEffect, useRef, useState } from "react";
// Module feuille et non baril de feature : la coque charge ce hook à chaque démarrage, et un
// baril tirerait tout ce qu'il réexporte dans le chunk d'entrée (voir `useNavCounts`).
import { settingsService } from "@/features/settings/services/settingsService";
import type { UpdateInfo } from "@/shared/types/generated/settings";

/**
 * Recherche une nouvelle version au démarrage (`docs/RELEASES.md`).
 *
 * Chaque lancement interroge GitHub. Une nouvelle version est présentée par le dialogue
 * de la coque ; les erreurs réseau restent silencieuses au démarrage.
 */
export function useStartupUpdateCheck(): {
  update: UpdateInfo | null;
  dismiss: () => void;
} {
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  // Un effet monté deux fois (mode strict) ne doit pas poser deux fois la même question.
  const demande = useRef(false);

  useEffect(() => {
    if (demande.current) return;
    demande.current = true;
    void settingsService
      .checkUpdateOnStartup()
      .then((available) => setUpdate(available))
      .catch(() => {
        // La revue dans un navigateur n'a pas de runtime Tauri, et un démarrage n'est pas le
        // moment d'annoncer une panne que l'utilisateur n'a pas provoquée.
      });
  }, []);

  return { update, dismiss: () => setUpdate(null) };
}
