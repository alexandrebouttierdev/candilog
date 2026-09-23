import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import type { IconName } from "./icon-names";

/**
 * État vide (`COMPONENTS.md` §15 du design).
 *
 * Il **explique la cause et propose la sortie** : jamais seulement « aucune donnée ».
 * Cercle de 38 px en pointillés (plein et vert quand le vide est une bonne nouvelle :
 * « Rien à faire aujourd'hui »), titre serif 19 px, corps 13 px, puis les actions.
 * `compact` donne la variante en bandeau des sections de profil vides.
 */
export function EmptyState({
  icon = "inbox",
  title,
  description,
  action,
  good = false,
  compact = false,
  bordered = false,
  className,
}: {
  icon?: IconName;
  title: string;
  description?: string | undefined;
  action?: ReactNode;
  /** Le vide est une bonne nouvelle : cercle plein `st-g`. */
  good?: boolean;
  compact?: boolean;
  bordered?: boolean;
  className?: string;
}) {
  if (compact) {
    return (
      <div
        role="status"
        className={cn("flex items-center gap-3.5 rounded-r8 bg-app px-[18px] py-[22px]", className)}
      >
        <Circle icon={icon} good={good} size="size-8" />
        <div className="min-w-0 flex-1">
          <p className="text-row font-medium text-tx">{title}</p>
          {description ? <p className="mt-0.5 text-small text-tx-4">{description}</p> : null}
        </div>
        {action}
      </div>
    );
  }

  return (
    <div
      role="status"
      className={cn(
        "flex flex-col items-center gap-[3px] px-10 py-6 text-center",
        bordered && "rounded-r9 border border-dashed border-bd-menu",
        className,
      )}
    >
      <Circle icon={icon} good={good} size="size-[38px]" />
      <p className="serif-title mt-2.5 text-empty text-tx">{title}</p>
      {description ? (
        <p className="max-w-[400px] text-row leading-[1.55] text-pretty text-tx-4">{description}</p>
      ) : null}
      {action ? <div className="mt-3.5 flex items-center justify-center gap-2">{action}</div> : null}
    </div>
  );
}

function Circle({ icon, good, size }: { icon: IconName; good: boolean; size: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        "inline-flex flex-none items-center justify-center rounded-full",
        size,
        good ? "bg-st-g text-white" : "border-[1.6px] border-dashed border-tx-6 text-tx-5",
      )}
    >
      <Icon name={icon} size={15} />
    </span>
  );
}
