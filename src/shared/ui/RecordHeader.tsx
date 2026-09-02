import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import type { IconName } from "./icon-names";

/**
 * Bandeau d'identité d'une fiche : pastille, nom, badge, actions, puis statistiques.
 *
 * Géométrie des maquettes Relations : bande en surface à filet bas, padding 20 px / 26 px,
 * pastille de 50 px, nom 19 px/650, actions de 31 px, et une rangée de statistiques séparée
 * par un filet haut (28 px de gouttière, 18 px / 16 px de padding).
 *
 * Partagé par les deux écrans Relations : la fiche entreprise et la fiche contact ne
 * diffèrent que par la forme de la pastille — ronde pour une personne, arrondie pour une
 * organisation — et par le contenu des statistiques.
 */
export function RecordHeader({
  initials,
  round = false,
  title,
  badge,
  subtitle,
  actions,
  stats,
}: {
  initials: string;
  round?: boolean;
  title: string;
  badge?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  /** Rangée de chiffres clés ; omise si la fiche n'en porte pas. */
  stats?: ReactNode;
}) {
  return (
    <header className="flex-none border-b border-line bg-surface px-[26px] pt-5">
      <div className="flex flex-wrap items-start gap-3.5">
        <span
          aria-hidden="true"
          className={cn(
            "flex size-[50px] flex-none items-center justify-center bg-accent-tint text-[16px] font-strong text-accent",
            round ? "rounded-full" : "rounded-card",
          )}
        >
          {initials}
        </span>
        <div className="min-w-[260px] flex-1">
          <div className="flex flex-wrap items-center gap-[9px]">
            <h2 className="text-heading text-ink">{title}</h2>
            {badge}
          </div>
          {subtitle ? <p className="mt-1 text-body text-ink-faint">{subtitle}</p> : null}
        </div>
        {actions ? <div className="flex flex-wrap gap-[7px]">{actions}</div> : null}
      </div>

      {stats ? (
        <div className="mt-4 flex flex-wrap gap-7 border-t border-line pt-[18px] pb-4">{stats}</div>
      ) : (
        <div className="h-5" />
      )}
    </header>
  );
}

/** Chiffre clé du bandeau : icône teintée, libellé, valeur 19 px tabulaire. */
export function RecordStat({
  icon,
  iconClassName,
  label,
  children,
}: {
  icon: IconName;
  iconClassName?: string;
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="min-w-0">
      <div className="mb-[5px] flex items-center gap-1.5">
        <Icon name={icon} size={15} className={cn("flex-none", iconClassName ?? "text-ink-faint")} />
        <span className="text-label font-medium text-ink-muted">{label}</span>
      </div>
      <p className="tabular text-heading text-ink">{children}</p>
    </div>
  );
}

/**
 * Action secondaire d'une fiche : bouton de 31 px, plus compact que ceux de l'en-tête.
 *
 * Les maquettes en alignent jusqu'à trois sous le nom ; au gabarit courant de 33 px, la
 * rangée déborderait sur le bandeau de statistiques.
 */
export function RecordAction({
  icon,
  children,
  onClick,
  disabled,
}: {
  icon: IconName;
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "inline-flex h-[31px] items-center gap-1.5 rounded-button border border-line bg-surface px-3",
        "text-note font-medium text-ink-muted transition-colors duration-150 hover:bg-neutral-tint",
        "disabled:pointer-events-none disabled:text-ink-faint",
      )}
    >
      <Icon name={icon} size={15} />
      {children}
    </button>
  );
}
