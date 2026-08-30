import { useCallback, useEffect, useRef, type PointerEvent as ReactPointerEvent } from "react";

/**
 * Possède le cycle de vie complet d'un glisser-déposer au pointeur.
 *
 * Les écouteurs globaux sont retirés à la fin normale, sur annulation, quand la fenêtre
 * perd le focus et au démontage du composant. Un nouveau drag nettoie aussi le précédent.
 */
export function usePointerDrag(
  onMove: (event: PointerEvent) => void,
  onEnd?: () => void,
) {
  const onMoveRef = useRef(onMove);
  const onEndRef = useRef(onEnd);
  const cleanupRef = useRef<(notify: boolean) => void>(() => undefined);
  useEffect(() => {
    onMoveRef.current = onMove;
    onEndRef.current = onEnd;
  }, [onMove, onEnd]);

  useEffect(() => () => cleanupRef.current(false), []);

  return useCallback((event: ReactPointerEvent) => {
    event.preventDefault();
    cleanupRef.current(false);

    const handleMove = (moveEvent: Event) => onMoveRef.current(moveEvent as PointerEvent);
    let active = true;
    const cleanup = (notify: boolean) => {
      if (!active) return;
      active = false;
      document.removeEventListener("pointermove", handleMove);
      document.removeEventListener("pointerup", handleEnd);
      document.removeEventListener("pointercancel", handleEnd);
      window.removeEventListener("blur", handleEnd);
      cleanupRef.current = () => undefined;
      if (notify) onEndRef.current?.();
    };
    const handleEnd = () => cleanup(true);

    cleanupRef.current = cleanup;
    document.addEventListener("pointermove", handleMove);
    document.addEventListener("pointerup", handleEnd, { once: true });
    document.addEventListener("pointercancel", handleEnd, { once: true });
    window.addEventListener("blur", handleEnd, { once: true });
  }, []);
}
