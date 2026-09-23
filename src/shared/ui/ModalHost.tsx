import type { ReactNode } from "react";
import { createContext, useContext, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { Button, IconButton } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { useFocusTrap } from "@/shared/hooks/useFocusTrap";
import type { IconName } from "./icon-names";

/**
 * Profondeur d'imbrication des modales.
 *
 * Une modale ouverte depuis le corps d'une autre — créer une entreprise sans quitter le
 * formulaire de candidature — est un descendant React de la première, portail compris. Le
 * contexte la superpose donc de façon explicite, sans dépendre de l'ordre d'insertion des
 * portails dans le document.
 */
const ProfondeurModale = createContext(0);

/**
 * Formulaire modal (`COMPONENTS.md` §11 du design).
 *
 * Surface `modal-bg`, rayon 11, ancrée à 58 px du haut, hauteur du contenu bornée à la
 * fenêtre − 78 px, corps défilant. En-tête : titre serif 17 px et sous-titre qui **énonce
 * le contrat** (« Le minimum suffit : intitulé et entreprise »). Pied : contrat clavier en
 * mono à gauche, `Annuler` et le bouton principal à droite. La structure en trois bandes
 * garde le pied et son action visibles quelle que soit la longueur du formulaire.
 *
 * Le focus entre dans le formulaire (premier champ), y reste piégé, et revient au
 * déclencheur à la fermeture. `icon` et `submitIcon` restent acceptés pour les écrans pas
 * encore migrés ; le design n'en affiche pas.
 */
export function ModalHost({
  open,
  title,
  subtitle,
  footer_note,
  footerTone = "neutral",
  submitLabel = "Enregistrer",
  submitDisabled = false,
  submitShortcut,
  busy = false,
  cancelLabel = "Annuler",
  flush = false,
  onClose,
  onSubmit,
  width = "600px",
  children,
}: {
  open: boolean;
  icon?: IconName;
  title: string;
  subtitle?: string | undefined;
  /** Contrat clavier ou précision, en mono à gauche du pied (`⏎ créer · ⇧⏎ …`). */
  footer_note?: string | undefined;
  footerIcon?: IconName;
  footerTone?: "neutral" | "danger";
  submitLabel?: string;
  submitIcon?: IconName;
  submitDisabled?: boolean;
  /** Raccourci imprimé dans le bouton principal. */
  submitShortcut?: string;
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
  const profondeur = useContext(ProfondeurModale);

  // Pendant un traitement long (ex. benchmark IA), Échap ne doit pas fermer la modale :
  // cela annulait la génération en cours et donnait l'impression que le test « ne marche pas ».
  useDismissable({
    open,
    onDismiss: onClose,
    dismissDisabled: busy,
    ...(onSubmit ? { onSubmit } : {}),
  });
  useFocusTrap(panel, open);

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
    <div
      style={{ zIndex: 60 + profondeur * 4 }}
      className="fixed inset-0 flex items-start justify-center bg-scrim px-8 pt-[58px] pb-5"
    >
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        tabIndex={-1}
        style={{ width, maxWidth: "100%" }}
        className={`flex max-h-[calc(100vh-78px)] animate-pop flex-col overflow-hidden rounded-r11 border border-bd-menu bg-modal shadow-pop${flush ? " h-[min(720px,calc(100vh-78px))]" : ""}`}
      >
        <header className="flex flex-none items-start gap-3 border-b border-bd-soft px-[17px] pt-[15px] pb-[13px]">
          <div className="min-w-0 flex-1">
            <h2 className="serif-title truncate text-form text-tx">{title}</h2>
            {subtitle ? <p className="mt-1 text-small text-tx-5">{subtitle}</p> : null}
          </div>
          <IconButton
            icon="close"
            label="Fermer"
            onClick={onClose}
            disabled={busy}
            size={13}
            className="size-[22px]"
          />
        </header>

        <div
          ref={body}
          className={
            flush
              ? "flex min-h-0 flex-1 flex-col overflow-hidden"
              : "min-h-0 flex-1 overflow-y-auto px-[17px] py-3.5"
          }
        >
          <ProfondeurModale.Provider value={profondeur + 1}>{children}</ProfondeurModale.Provider>
        </div>

        <footer className="flex flex-none items-center gap-2 border-t border-bd-soft px-[17px] pt-[13px] pb-4">
          <p
            className={`min-w-0 flex-1 truncate font-mono text-mono-sm ${
              footerTone === "danger" ? "text-st-c" : "text-tx-6"
            }`}
          >
            {footer_note}
          </p>
          <Button variant="secondary" onClick={onClose} disabled={busy}>
            {cancelLabel}
          </Button>
          {onSubmit ? (
            <Button
              variant="primary"
              disabled={submitDisabled || busy}
              onClick={onSubmit}
              {...(submitShortcut ? { shortcut: submitShortcut } : {})}
            >
              {busy ? "Enregistrement…" : submitLabel}
            </Button>
          ) : null}
        </footer>
      </div>
    </div>,
    document.body,
  );
}

/**
 * Section d'un formulaire : en-tête capitalisé et filet occupant le reste. `icon` est
 * conservé pour les écrans pas encore migrés.
 */
export function ModalSection({
  icon,
  title,
  children,
}: {
  icon: IconName;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="pt-4" data-icon={icon}>
      <div className="mb-2.5 flex items-center gap-2">
        <span className="caps">{title}</span>
        <span aria-hidden className="h-px flex-1 bg-bd-soft" />
      </div>
      {children}
    </section>
  );
}
