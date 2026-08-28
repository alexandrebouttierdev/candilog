import { useEffect } from "react";
import { createPortal } from "react-dom";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { Icon } from "./Icon";

const TONES = {
  success: { icon: "check_circle", className: "text-success" },
  error: { icon: "error", className: "text-danger" },
  info: { icon: "info", className: "text-accent" },
} as const;

/** Durée d'affichage d'un toast, fixée à 4 s par le guide. */
const DUREE_MS = 4000;

/**
 * File de notifications discrètes, en bas à droite.
 *
 * Jamais bloquantes : une erreur qui exige une décision passe par `ConfirmDialog`, une
 * erreur de chargement par `ErrorBanner`. Le toast ne sert qu'à confirmer ce qui vient
 * d'aboutir ou d'échouer sans conséquence sur la suite.
 */
export function Toaster() {
  const toasts = useUiStore((state) => state.toasts);

  return createPortal(
    <div
      aria-live="polite"
      className="pointer-events-none fixed right-5 bottom-5 z-60 flex flex-col gap-2"
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
  const tone = TONES[toast.tone];

  useEffect(() => {
    const timer = setTimeout(() => dismiss(toast.id), DUREE_MS);
    return () => clearTimeout(timer);
  }, [toast.id, dismiss]);

  return (
    <div
      role="status"
      className="pointer-events-auto flex w-[320px] items-start gap-2.5 rounded-card border border-line bg-surface px-3.5 py-3 shadow-e2"
    >
      <Icon name={tone.icon} size={17} className={`mt-px flex-none ${tone.className}`} />
      <div className="min-w-0 flex-1">
        <p className="truncate text-body font-medium text-ink">{toast.title}</p>
        {toast.detail ? (
          <p className="truncate text-meta text-ink-muted">{toast.detail}</p>
        ) : null}
      </div>
      <button
        type="button"
        aria-label="Fermer la notification"
        onClick={() => dismiss(toast.id)}
        className="flex size-6 flex-none items-center justify-center rounded-button text-ink-faint transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
      >
        <Icon name="close" size={15} />
      </button>
    </div>
  );
}
