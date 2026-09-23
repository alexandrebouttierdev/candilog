import { useEffect } from "react";
import { createPortal } from "react-dom";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { StatusGlyph } from "./StatusGlyph";
import type { GlyphTone } from "./StatusGlyph";

/** Durée d'affichage d'un toast : 2,6 s (`COMPONENTS.md` §12 du design). */
const DUREE_MS = 2600;

const GLYPHES: Record<ToastMessage["tone"], GlyphTone> = {
  success: "g",
  error: "c",
  info: "n",
};

/**
 * Notification brève, centrée à 20 px au-dessus de la barre d'état.
 *
 * Toujours factuelle et au passé (« CAN-142 supprimée »). Un seul toast à la fois : le
 * store remplace le précédent. Aucun bouton : une action annulable propose « annuler » à
 * l'endroit où elle a eu lieu, et une erreur qui exige une décision passe par un dialogue.
 */
export function Toaster() {
  const toasts = useUiStore((state) => state.toasts);

  return createPortal(
    <div
      aria-live="polite"
      className="pointer-events-none fixed inset-x-0 bottom-[54px] z-[80] flex justify-center"
    >
      {toasts.map((toast) => (
        <Toast key={toast.id} toast={toast} />
      ))}
    </div>,
    document.body,
  );
}

function Toast({ toast }: { toast: ToastMessage }) {
  const dismiss = useUiStore((state) => state.dismissToast);

  useEffect(() => {
    const timer = setTimeout(() => dismiss(toast.id), DUREE_MS);
    return () => clearTimeout(timer);
  }, [toast.id, dismiss]);

  return (
    <div
      role="status"
      className="pointer-events-auto flex max-w-[560px] animate-pop items-center gap-2 rounded-r9 border border-bd-menu bg-menu px-3.5 py-2 text-ui text-tx-2 shadow-pop"
    >
      <StatusGlyph tone={GLYPHES[toast.tone]} small />
      <p className="min-w-0 truncate">
        <span className="text-tx">{toast.title}</span>
        {toast.detail ? <span className="text-tx-4"> · {toast.detail}</span> : null}
      </p>
    </div>
  );
}
