import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useShortcut, isTypingTarget } from "@/shared/hooks/useShortcut";
import { hasOpenSurface } from "@/shared/hooks/useDismissable";
import { useUiStore } from "@/shared/lib/ui-store";
import { DESTINATIONS } from "@/app/router/routes";

/** Délai accordé entre `G` et la lettre de destination. */
const SEQUENCE_MS = 1200;

/**
 * Raccourcis de la coque (`INTERACTIONS.md` §6) : `⌘K` palette, `⌘,` Réglages — actifs
 * partout, y compris dans un champ — et `G` puis `A/C/R/D/P` pour changer de destination,
 * muets pendant une saisie ou quand une surface est ouverte.
 */
export function useShellShortcuts() {
  const navigate = useNavigate();
  const palette = useUiStore((state) => state.palette);
  const setPalette = useUiStore((state) => state.setPalette);
  const openSettings = useUiStore((state) => state.openSettings);

  useShortcut("mod+k", () => setPalette(!palette), { global: true });
  useShortcut("mod+,", () => openSettings(), { global: true });

  useEffect(() => {
    let armedUntil = 0;
    const listener = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      if (isTypingTarget(event.target) || hasOpenSurface()) return;
      const key = event.key.toLowerCase();
      if (Date.now() < armedUntil) {
        armedUntil = 0;
        const destination = DESTINATIONS.find((item) => item.goKey === key);
        if (destination) {
          event.preventDefault();
          void navigate(destination.path);
        }
        return;
      }
      if (key === "g" && !event.shiftKey) armedUntil = Date.now() + SEQUENCE_MS;
    };
    document.addEventListener("keydown", listener);
    return () => document.removeEventListener("keydown", listener);
  }, [navigate]);
}
