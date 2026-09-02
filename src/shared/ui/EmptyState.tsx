import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import type { IconName } from "./icon-names";

/**
 * État vide d'un écran ou d'une carte.
 *
 * Reprend la carte « Vide » des maquettes : encadré pointillé de 1,5 px, pastille d'icône
 * de 36 px, titre 13 px/600 puis explication 11,5 px et action. `bordered` peut être
 * désactivé lorsque l'état vide occupe déjà une carte à filet plein.
 *
 * Le design system demande un état vide **par carte** plutôt qu'un écran vide global :
 * ce composant est donc dimensionné pour être inséré dans un conteneur, pas pour occuper
 * la fenêtre.
 */
export function EmptyState({
  icon = "inbox",
  title,
  description,
  action,
  bordered = false,
  className,
}: {
  icon?: IconName;
  title: string;
  description?: string | undefined;
  action?: ReactNode;
  bordered?: boolean;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "px-[18px] py-6 text-center",
        bordered && "rounded-[11px] border-[1.5px] border-dashed border-line",
        className,
      )}
    >
      <span className="mb-[11px] inline-flex size-9 items-center justify-center rounded-tile bg-neutral-tint text-ink-faint">
        <Icon name={icon} size={20} />
      </span>
      <p className="mb-[5px] text-item font-semibold text-ink">{title}</p>
      {description ? (
        <p className="mx-auto max-w-sm text-label leading-normal text-ink-faint">{description}</p>
      ) : null}
      {action ? <div className="mt-[13px] flex justify-center">{action}</div> : null}
    </div>
  );
}
