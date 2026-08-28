import { createPortal } from "react-dom";
import { Icon } from "./Icon";
import { Button } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";

/**
 * Confirmation d'une action destructive.
 *
 * Le guide impose que toute destruction soit rouge, isolée et confirmée. L'énoncé doit
 * nommer ce qui disparaît **et ce qui survit** : la maquette de suppression de candidature
 * précise que l'entreprise et le contact associés sont conservés, sans quoi l'utilisateur
 * renonce par prudence.
 */
export function ConfirmDialog({
  open,
  title,
  description,
  note,
  confirmLabel = "Supprimer",
  busy = false,
  onCancel,
  onConfirm,
}: {
  open: boolean;
  title: string;
  description: string;
  /** Ce que l'action ne détruit pas, ou toute précision rassurante. */
  note?: string | undefined;
  confirmLabel?: string;
  busy?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  useDismissable({ open, onDismiss: onCancel });

  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-6">
      <div
        role="alertdialog"
        aria-modal="true"
        aria-label={title}
        className="w-[440px] max-w-full overflow-hidden rounded-card border border-line bg-surface shadow-e3"
      >
        <div className="flex items-start gap-3 px-5 pt-5">
          <span className="flex size-9 flex-none items-center justify-center rounded-card bg-danger-tint text-danger">
            <Icon name="warning" size={19} />
          </span>
          <div className="min-w-0">
            <h2 className="text-section text-ink">{title}</h2>
            <p className="mt-1 text-body text-ink-muted">{description}</p>
          </div>
        </div>

        {note ? (
          <p className="mx-5 mt-4 flex items-start gap-1.5 rounded-field bg-neutral-tint px-3 py-2 text-meta text-ink-muted">
            <Icon name="info" size={14} className="mt-px flex-none" />
            {note}
          </p>
        ) : null}

        <footer className="mt-5 flex items-center gap-3 border-t border-line bg-surface-alt px-5 py-3">
          <p className="flex-1 text-meta text-ink-faint">Échap pour annuler</p>
          <Button variant="ghost" onClick={onCancel}>
            Annuler
          </Button>
          <Button
            variant="danger"
            icon={busy ? "progress_activity" : "delete"}
            disabled={busy}
            onClick={onConfirm}
          >
            {confirmLabel}
          </Button>
        </footer>
      </div>
    </div>,
    document.body,
  );
}
