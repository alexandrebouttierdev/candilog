import type { ReactNode } from "react";
import { Icon } from "./Icon";

/**
 * État vide d'un écran ou d'une carte.
 *
 * Le guide SPECDESIGN demande un état vide **par carte** plutôt qu'un écran vide global :
 * ce composant est donc dimensionné pour être inséré dans un conteneur, pas pour occuper
 * la fenêtre.
 */
export function EmptyState({
  icon = "inbox",
  title,
  description,
  action,
}: {
  icon?: string;
  title: string;
  description?: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 px-6 py-10 text-center">
      <span className="flex size-11 items-center justify-center rounded-card bg-neutral-tint text-ink-faint">
        <Icon name={icon} size={22} />
      </span>
      <p className="text-section text-ink">{title}</p>
      {description ? (
        <p className="max-w-sm text-meta text-ink-muted">{description}</p>
      ) : null}
      {action ? <div className="mt-2">{action}</div> : null}
    </div>
  );
}
