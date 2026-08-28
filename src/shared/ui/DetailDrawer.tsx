import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { IconButton } from "./Button";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { cn } from "@/shared/lib/cn";

/**
 * Panneau latéral de fiche.
 *
 * Géométrie des maquettes : 460 px, en-tête de 18 px / 20 px avec pastille d'initiales de
 * 36 px et titre 16 px/650, corps à 16 px / 20 px, pied en `surface-alt`.
 *
 * Contrairement à `ModalHost`, le panneau n'atténue pas l'arrière-plan : la liste reste
 * lisible et cliquable à côté de la fiche, ce qui est tout l'intérêt d'un panneau plutôt
 * que d'une modale pour parcourir des éléments les uns après les autres.
 */
export function DetailDrawer({
  open,
  initials,
  icon,
  title,
  subtitle,
  actions,
  onClose,
  children,
}: {
  open: boolean;
  /** Pastille d'initiales de l'en-tête ; à défaut, `icon` est utilisée. */
  initials?: string;
  icon?: string;
  title: string;
  subtitle?: string | undefined;
  /** Actions contextuelles du pied. */
  actions?: ReactNode;
  onClose: () => void;
  children: ReactNode;
}) {
  useDismissable({ open, onDismiss: onClose });

  if (!open) return null;

  return (
    <aside
      aria-label={title}
      className="flex w-[460px] flex-none flex-col border-l border-line bg-surface"
    >
      <header className="flex flex-none items-start gap-[13px] border-b border-line px-5 py-[18px]">
        <span className="flex size-9 flex-none items-center justify-center rounded-tile bg-accent-tint text-note font-strong text-accent">
          {initials ?? <Icon name={icon ?? "description"} size={19} />}
        </span>
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-[16px] leading-tight font-strong tracking-[-0.015em] text-ink">
            {title}
          </h2>
          {subtitle ? (
            <p className="mt-[3px] truncate text-note text-ink-faint">{subtitle}</p>
          ) : null}
        </div>
        <IconButton icon="close" label="Fermer le panneau" onClick={onClose} />
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">{children}</div>

      {actions ? (
        <footer className="flex flex-none items-center gap-2.5 border-t border-line bg-surface-alt px-5 py-[13px]">
          {actions}
        </footer>
      ) : null}
    </aside>
  );
}

/** Groupe de la fiche : titre à filet, puis rangées libellé / valeur. */
export function DrawerSection({
  icon,
  title,
  className,
  children,
}: {
  icon: string;
  title: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <section className={cn("mb-[18px]", className)}>
      <div className="mb-2.5 flex items-center gap-2">
        <Icon name={icon} size={16} className="flex-none text-ink-faint" />
        <span className="text-note font-semibold tracking-[0.02em] text-ink">{title}</span>
        <span aria-hidden className="h-px flex-1 bg-line" />
      </div>
      {children}
    </section>
  );
}

/** Rangée libellé / valeur d'une fiche : filet bas, valeur alignée à droite. */
export function DrawerRow({
  label,
  children,
  tone,
}: {
  label: string;
  children: ReactNode;
  tone?: "accent" | "muted" | undefined;
}) {
  return (
    <div className="flex items-center justify-between gap-3.5 border-b border-line py-2">
      <span className="flex-none text-note text-ink-faint">{label}</span>
      <span
        className={cn(
          "min-w-0 flex-1 truncate text-right text-body font-medium",
          tone === "accent" ? "text-accent" : tone === "muted" ? "text-ink-faint" : "text-ink",
        )}
      >
        {children}
      </span>
    </div>
  );
}
