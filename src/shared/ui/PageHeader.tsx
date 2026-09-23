import type { ReactNode } from "react";
import { Icon } from "./Icon";
import type { IconName } from "./icon-names";

/** Toolbar d'écran : titre 13,5 px, actions compactes, pas de pastille SaaS. */
export function PageHeader({
  icon,
  title,
  subtitle,
  badge,
  toolbar,
  secondary,
  primary,
}: {
  icon: IconName;
  title: string;
  subtitle?: string | undefined;
  badge?: ReactNode;
  toolbar?: ReactNode;
  secondary?: ReactNode;
  primary?: ReactNode;
}) {
  return (
    <header className="flex h-topbar flex-none items-center justify-between gap-3 border-b border-line-soft px-4">
      <div className="flex min-w-0 items-center gap-2">
        <Icon name={icon} size={16} className="flex-none text-ink-disabled" />
        <h1 className="truncate text-section text-ink">{title}</h1>
        {subtitle ? (
          <>
            <span aria-hidden className="h-3.5 w-px flex-none bg-line" />
            <p className="truncate text-note text-ink-faint">{subtitle}</p>
          </>
        ) : null}
      </div>
      <div className="flex flex-none items-center gap-1.5">
        {badge}
        {toolbar}
        {secondary}
        {primary}
      </div>
    </header>
  );
}

/**
 * Groupe segmenté (`COMPONENTS.md` §3.1 du design) : piste `bg-chip`, segment actif
 * `bg-panel` en graisse 500. Remplace les listes déroulantes pour 2 à 4 options courtes
 * (thème, densité, ton, longueur, contrat…).
 */
export function SegmentedControl<TValue extends string>({
  value,
  options,
  onChange,
  label,
  dense = false,
}: {
  value: TValue;
  options: readonly { readonly value: TValue; readonly label: string; readonly icon?: IconName }[];
  onChange: (value: TValue) => void;
  label: string;
  dense?: boolean;
}) {
  return (
    <div
      role="group"
      aria-label={label}
      className="flex flex-none items-center gap-px rounded-r7 bg-chip p-0.5"
    >
      {options.map((option) => {
        const actif = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            aria-pressed={actif}
            onClick={() => onChange(option.value)}
            className={[
              "inline-flex h-[22px] items-center gap-1.5 rounded-r6 transition-color",
              dense ? "px-2 text-small" : "px-2.5 text-ui",
              actif ? "bg-panel font-medium text-tx" : "text-tx-4 hover:text-tx-2",
            ].join(" ")}
          >
            {option.icon ? <Icon name={option.icon} size={15} /> : null}
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
