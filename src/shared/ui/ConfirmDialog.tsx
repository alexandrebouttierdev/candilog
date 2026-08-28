import { createPortal } from "react-dom";
import { Icon } from "./Icon";
import { Button } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";

/**
 * Confirmation d'une action destructive.
 *
 * Reprend la maquette « Confirmation de suppression » : panneau de 440 px et rayon 14 px,
 * pastille d'alerte de 38 px, énoncé 12,5 px, encadré de réassurance sur fond page, pied
 * en `surface-alt`.
 *
 * Le guide impose que toute destruction soit rouge, isolée et confirmée. L'énoncé doit
 * nommer ce qui disparaît **et ce qui survit** : la maquette précise que l'entreprise et le
 * contact associés sont conservés, sans quoi l'utilisateur renonce par prudence.
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-scrim/70 p-[34px] backdrop-blur-[2px]">
      <div
        role="alertdialog"
        aria-modal="true"
        aria-label={title}
        className="w-[440px] max-w-full overflow-hidden rounded-[14px] border border-line bg-surface shadow-e3"
      >
        <div className="px-[22px] pt-[22px] pb-[18px]">
          <span className="mb-3.5 flex size-[38px] items-center justify-center rounded-[11px] bg-danger-tint text-danger">
            <Icon name="warning" size={21} />
          </span>
          <h2 className="mb-2 text-[16.5px] font-strong tracking-[-0.015em] text-ink">{title}</h2>
          <p className="mb-3.5 text-body leading-[1.6] text-pretty text-ink-muted">{description}</p>
          {note ? (
            <p className="flex items-center gap-2.5 rounded-tile border border-line bg-page px-[13px] py-[11px] text-label leading-normal text-ink-muted">
              <Icon name="info" size={17} className="flex-none text-ink-faint" />
              {note}
            </p>
          ) : null}
        </div>

        <footer className="flex items-center gap-2.5 border-t border-line bg-surface-alt px-[22px] py-3.5">
          <p className="flex-1 text-label text-ink-faint">Échap pour annuler</p>
          <Button variant="secondary" size="dialog" onClick={onCancel}>
            Annuler
          </Button>
          <Button
            variant="primary"
            size="dialog"
            icon={busy ? "progress_activity" : "delete"}
            disabled={busy}
            onClick={onConfirm}
            className="bg-danger shadow-none hover:brightness-110"
          >
            {confirmLabel}
          </Button>
        </footer>
      </div>
    </div>,
    document.body,
  );
}
