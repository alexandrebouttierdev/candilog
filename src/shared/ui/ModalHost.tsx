import type { ReactNode } from "react";
import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icon";
import { Button, IconButton } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";

/**
 * Modale du guide : en-tête, corps défilant, pied fixe.
 *
 * Géométrie des maquettes : rayon 14 px, ombre de niveau 3, en-tête de 18 px / 22 px avec
 * pastille d'icône de 34 px et titre 16 px/650, pied en `surface-alt` de 14 px / 22 px.
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
  footer_note,
  footerIcon = "info",
  footerTone = "neutral",
  submitLabel = "Enregistrer",
  submitIcon = "check",
  submitDisabled = false,
  busy = false,
  cancelLabel = "Annuler",
  flush = false,
  onClose,
  onSubmit,
  width = "620px",
  children,
}: {
  open: boolean;
  icon: string;
  title: string;
  subtitle?: string | undefined;
  footer_note?: string | undefined;
  footerIcon?: string;
  footerTone?: "neutral" | "danger";
  submitLabel?: string;
  submitIcon?: string;
  submitDisabled?: boolean;
  busy?: boolean;
  cancelLabel?: string;
  /** Corps sans gouttière, pour un split liste / détail. */
  flush?: boolean;
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
    const firstChamp = body.current?.querySelector<HTMLElement>(
      "input, select, textarea, button, [tabindex]:not([tabindex='-1'])",
    );
    (firstChamp ?? panel.current)?.focus();
  }, [open]);

  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-scrim/70 p-[34px] backdrop-blur-[2px]">
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        style={{ width, maxWidth: "100%" }}
        className={`flex max-h-full flex-col overflow-hidden rounded-[14px] border border-line bg-surface shadow-e3${flush ? " h-[min(720px,100%)]" : ""}`}
      >
        <header className="flex flex-none items-start gap-[13px] border-b border-line px-[22px] py-[18px]">
          <span className="flex size-[34px] flex-none items-center justify-center rounded-tile bg-accent-tint text-accent">
            <Icon name={icon} size={19} />
          </span>
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-[16px] leading-tight font-strong tracking-[-0.015em] text-ink">
              {title}
            </h2>
            {subtitle ? (
              <p className="mt-[3px] truncate text-note text-ink-faint">{subtitle}</p>
            ) : null}
          </div>
          <IconButton icon="close" label="Fermer" onClick={onClose} />
        </header>

        <div
          ref={body}
          className={
            flush
              ? "flex min-h-0 flex-1 flex-col overflow-hidden"
              : "min-h-0 flex-1 overflow-y-auto px-[22px] pt-1.5 pb-[18px]"
          }
        >
          {children}
        </div>

        <footer className="flex flex-none flex-wrap items-center gap-3 border-t border-line bg-surface-alt px-[22px] py-3.5">
          {footer_note ? (
            <p
              className={`flex min-w-[180px] flex-1 items-center gap-1.5 text-label ${
                footerTone === "danger" ? "text-danger" : "text-ink-faint"
              }`}
            >
              <Icon name={footerIcon} size={15} className="flex-none" />
              {footer_note}
            </p>
          ) : (
            <div className="flex-1" />
          )}
          <Button variant="secondary" size="dialog" onClick={onClose}>
            {cancelLabel}
          </Button>
          {onSubmit ? (
            <Button
              variant="primary"
              size="dialog"
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

/**
 * Title de section d'un formulaire : icône, libellé 12,5 px/600, filet occupant le reste.
 */
export function ModalSection({
  icon,
  title,
  children,
}: {
  icon: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="pt-[18px]">
      <div className="mb-3 flex items-center gap-2">
        <Icon name={icon} size={16} className="flex-none text-ink-faint" />
        <span className="text-body font-semibold text-ink">{title}</span>
        <span aria-hidden className="h-px flex-1 bg-line" />
      </div>
      {children}
    </section>
  );
}
