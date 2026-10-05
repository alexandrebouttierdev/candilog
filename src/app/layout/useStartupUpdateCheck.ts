import { useEffect, useRef } from "react";
import { useUiStore } from "@/shared/lib/ui-store";
// Module feuille et non baril de feature : la coque charge ce hook à chaque démarrage, et un
// baril tirerait tout ce qu'il réexporte dans le chunk d'entrée (voir `useNavCounts`).
import { settingsService } from "@/features/settings/services/settingsService";

/**
 * Annonce une nouvelle version au démarrage (`docs/RELEASES.md`).
 *
 * Toute la politique est côté Rust : une interrogation de l'API GitHub par jour au plus, et
 * silence quand le réseau n'a pas répondu. Ici il ne reste qu'à dire ce qui a été trouvé.
 *
 * Le toast n'a pas de bouton — le design l'interdit (`docs/DESIGN.md` §7) — il nomme donc sa
 * destination. Il ne remplace pas l'écran Mises à jour, qui reste le seul endroit où l'on
 * vérifie, télécharge et installe.
 */
export function useStartupUpdateCheck(): void {
  const notify = useUiStore((state) => state.notify);
  // Un effet monté deux fois (mode strict) ne doit pas poser deux fois la même question.
  const demande = useRef(false);

  useEffect(() => {
    if (demande.current) return;
    demande.current = true;
    void settingsService
      .checkUpdateIfDue()
      .then((update) => {
        if (!update) return;
        notify({
          tone: "info",
          title: `Candilog ${update.version} est disponible`,
          detail: "Réglages › Mises à jour",
        });
      })
      .catch(() => {
        // La revue dans un navigateur n'a pas de runtime Tauri, et un démarrage n'est pas le
        // moment d'annoncer une panne que l'utilisateur n'a pas provoquée.
      });
  }, [notify]);
}
