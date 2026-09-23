import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { Button } from "./Button";
import { StatusGlyph } from "./StatusGlyph";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { useFocusTrap } from "@/shared/hooks/useFocusTrap";
import { cn } from "@/shared/lib/cn";
import type { IconName } from "./icon-names";

/**
 * Registre d'un dialogue (`COMPONENTS.md` §10 du design) : la couleur du glyphe est la
 * seule différence structurelle.
 * - `information` : tuile `bg-chip`, glyphe neutre, bouton primaire ;
 * - `confirmation` : tuile `tint-ac-bg`, glyphe ambre, bouton primaire ;
 * - `destruction` : tuile `tint-c-bg`, disque plein rouge, bouton en contour rouge.
 */
export type DialogRegister = "information" | "confirmation" | "destruction";

const TUILE: Record<DialogRegister, string> = {
  information: "bg-chip",
  confirmation: "bg-tint-ac-bg",
  destruction: "bg-tint-c-bg",
};

const GLYPHE = { information: "n", confirmation: "a", destruction: "c" } as const;

/** Une conséquence énumérée : ce qui disparaît (ou change), avec sa valeur à droite. */
export interface DialogConsequence {
  readonly label: string;
  readonly value?: string;
}

/**
 * Dialogue modal : destruction, confirmation ou information.
 *
 * Il remplace « êtes-vous sûr ? » par l'énumération de ce qui disparaît **et** de ce qui
 * survit. Largeur 420 px, rayon 11, animation `pop`, voile `scrim` au-dessus de toutes les
 * surcouches (z 70). `⏎` confirme, `Échap` annule ; le focus est piégé dans le dialogue et
 * rendu à l'élément déclencheur à la fermeture.
 *
 * `confirmIcon` est conservé pour les écrans pas encore migrés ; le design n'en affiche pas.
 */
export function ConfirmDialog({
  open,
  title,
  description,
  note,
  consequences,
  footnote,
  register = "destruction",
  confirmLabel = "Supprimer",
  cancelLabel = "Annuler",
  busy = false,
  cancelDisabled = false,
  dismissDisabled = false,
  confirmDisabled = false,
  hideCancel = false,
  initialFocus,
  onCancel,
  onConfirm,
  children,
}: {
  open: boolean;
  title: string;
  description: string;
  /** Ce que l'action ne détruit pas, ou toute précision rassurante. */
  note?: string | undefined;
  consequences?: readonly DialogConsequence[] | undefined;
  /** Mention mono à gauche du pied : « action définitive », « aucun retour possible ». */
  footnote?: string | undefined;
  register?: DialogRegister;
  confirmLabel?: string;
  cancelLabel?: string;
  confirmIcon?: IconName;
  busy?: boolean;
  cancelDisabled?: boolean;
  dismissDisabled?: boolean;
  /** Bouton principal inactif (ex. saisie de confirmation incomplète). */
  confirmDisabled?: boolean;
  /** Dialogue d'information sans alternative : un seul bouton. */
  hideCancel?: boolean;
  /**
   * Bouton qui reçoit le focus à l'ouverture. Par défaut, le moins destructeur pour une
   * destruction, le principal sinon ; `cancel` pour un dialogue dont l'action principale
   * fait perdre quelque chose (« Fermer sans enregistrer ? »).
   */
  initialFocus?: "confirm" | "cancel";
  onCancel: () => void;
  onConfirm: () => void;
  /** Contenu additionnel entre les conséquences et le pied (champ de confirmation…). */
  children?: React.ReactNode;
}) {
  const panel = useRef<HTMLDivElement>(null);
  const confirmer = confirmDisabled || busy ? undefined : onConfirm;
  useDismissable({
    open,
    onDismiss: onCancel,
    dismissDisabled,
    ...(confirmer ? { onEnter: confirmer } : {}),
  });
  useFocusTrap(panel, open);

  // Focus initial sur le bouton le moins destructeur d'un dialogue de destruction : un
  // `⏎` réflexe ne doit pas supprimer. Sur les autres registres, sur le bouton principal.
  const principal = useRef<HTMLButtonElement>(null);
  const secondaire = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) return;
    const annuler = initialFocus ? initialFocus === "cancel" : register === "destruction";
    const cible = annuler && !hideCancel ? secondaire : principal;
    if (!panel.current?.contains(document.activeElement)) cible.current?.focus();
  }, [open, register, hideCancel, initialFocus]);

  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-scrim p-8">
      <div
        ref={panel}
        role={register === "information" ? "dialog" : "alertdialog"}
        aria-modal="true"
        aria-label={title}
        className={cn(
          "max-w-full animate-pop rounded-r11 border border-bd-menu bg-modal px-[18px] pt-[17px] pb-[15px] shadow-pop",
          children ? "w-[440px]" : "w-[420px]",
        )}
      >
        <div className="flex items-start gap-3">
          <span
            className={cn("flex size-7 flex-none items-center justify-center rounded-r8", TUILE[register])}
          >
            <StatusGlyph tone={GLYPHE[register]} />
          </span>
          <div className="min-w-0 flex-1">
            <h2 className="serif-title text-dialog text-tx">{title}</h2>
            <p className="mt-1.5 text-ui leading-[1.5] text-pretty text-tx-4">{description}</p>
          </div>
        </div>

        {consequences && consequences.length > 0 ? (
          <ul className="mt-3.5 flex flex-col gap-1.5 rounded-r8 bg-group px-3 py-2.5">
            {consequences.map((item) => (
              <li key={item.label} className="flex items-center gap-2 text-small text-tx-3">
                <span
                  aria-hidden
                  className={cn(
                    "size-[7px] flex-none rounded-full",
                    register === "destruction" ? "bg-st-c" : "bg-tx-6",
                  )}
                />
                <span className="min-w-0 flex-1">{item.label}</span>
                {item.value ? (
                  <span className="font-mono text-mono-sm text-tx-5">{item.value}</span>
                ) : null}
              </li>
            ))}
          </ul>
        ) : null}

        {note ? <p className="mt-3 text-sub leading-[1.5] text-tx-5">{note}</p> : null}
        {children ? <div className="mt-3.5">{children}</div> : null}

        <footer className="mt-4 flex items-center gap-2">
          <p className="min-w-0 flex-1 truncate font-mono text-mono-sm text-tx-6">
            {dismissDisabled ? "arrêt en cours…" : footnote}
          </p>
          {hideCancel ? null : (
            <Button ref={secondaire} variant="secondary" disabled={cancelDisabled} onClick={onCancel}>
              {cancelLabel}
            </Button>
          )}
          <Button
            ref={principal}
            variant={register === "destruction" ? "danger" : "primary"}
            disabled={busy || confirmDisabled}
            aria-busy={busy || undefined}
            onClick={onConfirm}
            {...(busy ? {} : { shortcut: "enter" })}
          >
            {busy ? (
              <span
                aria-hidden
                className="size-3 animate-spin-ring rounded-full border-[1.5px] border-current border-t-transparent"
              />
            ) : null}
            {confirmLabel}
          </Button>
        </footer>
      </div>
    </div>,
    document.body,
  );
}
