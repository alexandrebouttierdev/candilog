import type { ReactNode } from "react";
import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icon";
import { Button } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";

/**
 * Modale du guide : en-tête, corps défilant, pied fixe.
 *
 * Rendue en superposition dans le document plutôt que dans une fenêtre native : le guide
 * demande de conserver l'arrière-plan atténué et le focus dans la page. La structure en
 * trois bandes (`grid-rows-[auto_1fr_auto]`) garantit que le pied et son action primaire
 * restent visibles quelle que soit la longueur du formulaire — c'est le défaut que la
 * refonte corrige.
 */
export function ModalHost({
  open,
  icon,
  title,
  subtitle,
  footerNote,
  submitLabel = "Enregistrer",
  submitIcon = "check",
  submitDisabled = false,
  busy = false,
  onClose,
  onSubmit,
  width = "620px",
  children,
}: {
  open: boolean;
  icon: string;
  title: string;
  subtitle?: string | undefined;
  footerNote?: string | undefined;
  submitLabel?: string;
  submitIcon?: string;
  submitDisabled?: boolean;
  busy?: boolean;
  onClose: () => void;
  onSubmit?: () => void;
  width?: string;
  children: ReactNode;
}) {
  const panel = useRef<HTMLDivElement>(null);
  const body = useRef<HTMLDivElement>(null);

  useDismissable({ open, onDismiss: onClose, ...(onSubmit ? { onSubmit } : {}) });

  // Le focus doit entrer dans la modale à l'ouverture, sinon la tabulation continue de
  // parcourir l'arrière-plan atténué, invisible mais toujours atteignable au clavier.
  //
  // La recherche est bornée au corps et non au panneau entier : `querySelector` renvoie le
  // premier élément dans l'ordre du document, ce qui serait le bouton de fermeture de
  // l'en-tête — l'utilisateur devrait alors tabuler jusqu'au premier champ à chaque
  // ouverture. À défaut de champ (fiche en lecture seule), le panneau lui-même reçoit le
  // focus, ce qui fait annoncer le dialogue par les lecteurs d'écran.
  useEffect(() => {
    if (!open) return;
    const premierChamp = body.current?.querySelector<HTMLElement>(
      "input, select, textarea, button, [tabindex]:not([tabindex='-1'])",
    );
    (premierChamp ?? panel.current)?.focus();
  }, [open]);

  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-6">
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        style={{ width, maxWidth: "100%" }}
        className="grid max-h-full grid-rows-[auto_1fr_auto] overflow-hidden rounded-card border border-line bg-surface shadow-e3"
      >
        <header className="flex items-center gap-3 border-b border-line px-5 py-4">
          <span className="flex size-9 flex-none items-center justify-center rounded-card bg-accent-tint text-accent">
            <Icon name={icon} size={19} />
          </span>
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-section text-ink">{title}</h2>
            {subtitle ? <p className="truncate text-meta text-ink-muted">{subtitle}</p> : null}
          </div>
          <button
            type="button"
            aria-label="Fermer"
            onClick={onClose}
            className="flex size-8 items-center justify-center rounded-button text-ink-faint transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
          >
            <Icon name="close" size={18} />
          </button>
        </header>

        <div ref={body} className="min-h-0 overflow-y-auto px-5 py-4">
          {children}
        </div>

        <footer className="flex items-center gap-3 border-t border-line bg-surface-alt px-5 py-3">
          {footerNote ? (
            <p className="min-w-0 flex-1 truncate text-meta text-ink-faint">{footerNote}</p>
          ) : (
            <div className="flex-1" />
          )}
          <Button variant="ghost" onClick={onClose}>
            Annuler
          </Button>
          {onSubmit ? (
            <Button
              variant="primary"
              icon={busy ? "progress_activity" : submitIcon}
              disabled={submitDisabled || busy}
              onClick={onSubmit}
            >
              {submitLabel}
            </Button>
          ) : null}
        </footer>
      </div>
    </div>,
    document.body,
  );
}
