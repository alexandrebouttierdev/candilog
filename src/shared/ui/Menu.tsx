import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { createPortal } from "react-dom";
import { cn } from "@/shared/lib/cn";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { Kbd } from "./Kbd";

/** Entrée activable d'un menu. */
export interface MenuItem {
  readonly kind: "item";
  readonly id: string;
  readonly label: string;
  /** Raccourci imprimé à droite (notation neutre : `s`, `mod+d`, `enter`). */
  readonly shortcut?: string;
  /** Métadonnée mono à droite (décompte, taille, latence). */
  readonly meta?: string;
  /** Glyphe ou marque à gauche (glyphe de statut, puce de localité). */
  readonly leading?: ReactNode;
  /** Élément courant : coche `ac-tx` à droite. */
  readonly checked?: boolean;
  readonly disabled?: boolean;
  /** Motif affiché sous un élément désactivé (« contexte trop court »). */
  readonly reason?: string;
  readonly tone?: "danger";
  /** Garde le menu ouvert après l'activation (navigation vers un sous-niveau). */
  readonly keepOpen?: boolean;
  readonly onSelect: () => void;
}

export type MenuEntry =
  | MenuItem
  | { readonly kind: "separator"; readonly id: string }
  | { readonly kind: "section"; readonly id: string; readonly label: string };

/** Ancre : un point (clic droit) ou le rectangle d'un déclencheur. */
export type MenuAnchor = { readonly x: number; readonly y: number } | DOMRect;

const MARGE = 8;

/**
 * Menu flottant (`COMPONENTS.md` §3.2 du design) : surface `menu-bg`, rayon 9, ombre
 * unique, lignes de 28 px, en-têtes capitalisés, raccourcis en mono à droite.
 *
 * Clavier : `↑ ↓` parcourent les entrées actives, `⏎` active, `Échap` ferme (via la pile
 * de `useDismissable` : seule la surface la plus haute réagit). Il s'ouvre vers le haut
 * quand l'espace sous l'ancre ne suffit pas, et reste dans la fenêtre.
 */
export function Menu({
  open,
  anchor,
  entries,
  onClose,
  label,
  width = 236,
  header,
  onBack,
}: {
  open: boolean;
  anchor: MenuAnchor | null;
  entries: readonly MenuEntry[];
  onClose: () => void;
  /** Nom accessible du menu. */
  label: string;
  width?: number;
  /** Contenu fixe au-dessus des entrées (retour de sous-niveau, titre). */
  header?: ReactNode;
  /** Retour au niveau précédent : `←` ou `⌫`, hors saisie dans un champ de l'en-tête. */
  onBack?: () => void;
}) {
  const surface = useRef<HTMLDivElement>(null);
  const actives = entries.filter((entry): entry is MenuItem => entry.kind === "item" && !entry.disabled);
  const signature = entries.map((entry) => entry.id).join("|");
  // L'élément actif est rattaché à la liste qui l'a vu naître : quand les entrées changent
  // (sous-niveau du menu de filtre), l'activation repart de la première sans effet.
  const [active, setActive] = useState<{ signature: string; id: string } | null>(null);
  const activeId = active?.signature === signature ? active.id : null;
  const setActiveId = (id: string) => setActive({ signature, id });
  const [position, setPosition] = useState<{ left: number; top: number } | null>(null);

  const courant = actives.find((item) => item.id === activeId) ?? actives[0] ?? null;

  const activer = (item: MenuItem) => {
    item.onSelect();
    if (!item.keepOpen) onClose();
  };

  useDismissable({
    open,
    onDismiss: onClose,
    ...(courant ? { onEnter: () => activer(courant) } : {}),
  });

  useLayoutEffect(() => {
    if (!open || !anchor || !surface.current) return;
    const hauteur = surface.current.offsetHeight;
    const rect = "width" in anchor ? anchor : null;
    const x = rect ? rect.left : anchor.x;
    const basAncre = rect ? rect.bottom + 4 : anchor.y;
    const hautAncre = rect ? rect.top - 4 : anchor.y;
    const top =
      basAncre + hauteur + MARGE > window.innerHeight
        ? Math.max(MARGE, hautAncre - hauteur)
        : basAncre;
    const left = Math.min(Math.max(MARGE, x), window.innerWidth - width - MARGE);
    setPosition({ left, top });
  }, [open, anchor, width, signature]);

  useEffect(() => {
    if (!open) return;
    surface.current?.focus();
    const dehors = (event: PointerEvent) => {
      if (surface.current && !surface.current.contains(event.target as Node)) onClose();
    };
    document.addEventListener("pointerdown", dehors, true);
    return () => document.removeEventListener("pointerdown", dehors, true);
  }, [open, onClose]);

  if (!open || !anchor) return null;

  const deplacer = (delta: number) => {
    if (actives.length === 0) return;
    const index = courant ? actives.indexOf(courant) : -1;
    const suivant = actives[(index + delta + actives.length) % actives.length]!;
    setActiveId(suivant.id);
  };

  return createPortal(
    <div
      ref={surface}
      role="menu"
      aria-label={label}
      tabIndex={-1}
      aria-activedescendant={courant ? `menu-${courant.id}` : undefined}
      onKeyDown={(event) => {
        if (event.key === "ArrowDown") {
          event.preventDefault();
          deplacer(1);
        } else if (event.key === "ArrowUp") {
          event.preventDefault();
          deplacer(-1);
        } else if (
          onBack &&
          (event.key === "ArrowLeft" || event.key === "Backspace") &&
          !(event.target instanceof HTMLInputElement)
        ) {
          event.preventDefault();
          onBack();
        }
      }}
      style={{
        width,
        left: position?.left ?? -9999,
        top: position?.top ?? -9999,
      }}
      className="fixed z-[75] max-h-[calc(100vh-16px)] animate-pop-menu overflow-y-auto rounded-r9 border border-bd-menu bg-menu p-[5px] shadow-pop outline-none"
    >
      {header}
      {entries.map((entry) => {
        if (entry.kind === "separator") {
          return <div key={entry.id} role="separator" className="mx-1 my-1 h-px bg-bd-soft" />;
        }
        if (entry.kind === "section") {
          return (
            <div key={entry.id} role="presentation" className="caps px-[9px] pt-1.5 pb-1">
              {entry.label}
            </div>
          );
        }
        const actif = courant?.id === entry.id;
        return (
          <div
            key={entry.id}
            id={`menu-${entry.id}`}
            role={entry.checked === undefined ? "menuitem" : "menuitemradio"}
            aria-disabled={entry.disabled || undefined}
            {...(entry.checked === undefined ? {} : { "aria-checked": entry.checked })}
            onPointerMove={() => !entry.disabled && setActiveId(entry.id)}
            onClick={() => !entry.disabled && activer(entry)}
            className={cn(
              "flex min-h-menu-row cursor-default items-center gap-2 rounded-r6 px-[9px] text-ui",
              entry.disabled ? "text-tx-6 opacity-55" : entry.tone === "danger" ? "text-st-c" : "text-tx-2",
              actif && "bg-elev",
            )}
          >
            {entry.leading}
            <span className="min-w-0 flex-1 py-1">
              <span className="block truncate">{entry.label}</span>
              {entry.disabled && entry.reason ? (
                <span className="block text-tiny text-tx-6">{entry.reason}</span>
              ) : null}
            </span>
            {entry.meta ? <span className="font-mono text-mono-sm text-tx-6">{entry.meta}</span> : null}
            {entry.checked ? (
              <span aria-hidden className="text-small text-ac-tx">
                ✓
              </span>
            ) : null}
            {entry.shortcut ? <Kbd shortcut={entry.shortcut} tone="ghost" decorative /> : null}
          </div>
        );
      })}
    </div>,
    document.body,
  );
}
