import { useEffect } from "react";
import type { RefObject } from "react";

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"]), [contenteditable="true"]';

/**
 * Piège le focus dans une surface modale et le rend à l'élément déclencheur à la fermeture
 * (`reference_design/INTERACTIONS.md` §7) : tant qu'un dialogue ou un formulaire est
 * ouvert, `Tab` ne doit pas parcourir l'arrière-plan atténué, invisible mais atteignable.
 */
export function useFocusTrap(container: RefObject<HTMLElement | null>, active: boolean) {
  useEffect(() => {
    if (!active) return;
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;

    const handler = (event: KeyboardEvent) => {
      if (event.key !== "Tab" || !container.current) return;
      const focusables = [...container.current.querySelectorAll<HTMLElement>(FOCUSABLE)];
      if (focusables.length === 0) {
        event.preventDefault();
        return;
      }
      const first = focusables[0]!;
      const last = focusables[focusables.length - 1]!;
      const inside = container.current.contains(document.activeElement);
      if (event.shiftKey && (document.activeElement === first || !inside)) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && (document.activeElement === last || !inside)) {
        event.preventDefault();
        first.focus();
      }
    };

    document.addEventListener("keydown", handler);
    return () => {
      document.removeEventListener("keydown", handler);
      // Le déclencheur peut avoir disparu entre-temps (ligne supprimée) : ne rien forcer.
      if (trigger?.isConnected) trigger.focus();
    };
  }, [active, container]);
}
