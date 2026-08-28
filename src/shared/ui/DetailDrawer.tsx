import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { useDismissable } from "@/shared/hooks/useDismissable";

/**
 * Panneau latéral de fiche.
 *
 * Contrairement à `ModalHost`, le panneau n'atténue pas l'arrière-plan : la liste reste
 * lisible et cliquable à côté de la fiche, ce qui est tout l'intérêt d'un panneau plutôt
 * que d'une modale pour parcourir des éléments les uns après les autres.
 */
export function DetailDrawer({
  open,
  title,
  subtitle,
  actions,
  onClose,
  children,
}: {
  open: boolean;
  title: string;
  subtitle?: string | undefined;
  /** Actions contextuelles du bandeau d'identité. */
  actions?: ReactNode;
  onClose: () => void;
  children: ReactNode;
}) {
  useDismissable({ open, onDismiss: onClose });

  if (!open) return null;

  return (
    <aside
      aria-label={title}
      className="flex w-[380px] flex-none flex-col border-l border-line bg-surface"
    >
      <header className="flex flex-none items-start gap-3 border-b border-line px-4 py-3.5">
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-section text-ink">{title}</h2>
          {subtitle ? <p className="truncate text-meta text-ink-muted">{subtitle}</p> : null}
        </div>
        <button
          type="button"
          aria-label="Fermer le panneau"
          onClick={onClose}
          className="flex size-8 flex-none items-center justify-center rounded-button text-ink-faint transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
        >
          <Icon name="close" size={18} />
        </button>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-4">{children}</div>

      {actions ? (
        <footer className="flex flex-none items-center gap-2 border-t border-line bg-surface-alt px-4 py-3">
          {actions}
        </footer>
      ) : null}
    </aside>
  );
}
