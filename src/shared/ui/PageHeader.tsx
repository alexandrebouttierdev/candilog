import type { ReactNode } from "react";
import { Icon } from "./Icon";

/**
 * En-tête d'écran : icône, titre et sous-titre en ligne, actions à droite.
 *
 * Géométrie des maquettes SPECDESIGN : bande en surface de 17 px de padding vertical et
 * 28 px horizontal, pastille d'icône de 34 px, titre 20 px/650, filet vertical de 18 px
 * puis sous-titre tertiaire. Le guide n'admet qu'une seule action `primary` par écran.
 */
export function PageHeader({
  icon,
  title,
  subtitle,
  badge,
  toolbar,
  secondary,
  primary,
}: {
  icon: string;
  title: string;
  subtitle?: string | undefined;
  badge?: ReactNode;
  /** Bascule de vue ou sélecteur de période, posé avant les boutons. */
  toolbar?: ReactNode;
  secondary?: ReactNode;
  primary?: ReactNode;
}) {
  return (
    <header className="flex flex-none items-center justify-between gap-5 border-b border-line bg-surface px-7 py-[17px]">
      <div className="flex min-w-0 items-center gap-[13px]">
        <span className="flex size-[34px] flex-none items-center justify-center rounded-tile bg-accent-tint text-accent">
          <Icon name={icon} size={19} />
        </span>
        <h1 className="truncate text-title">{title}</h1>
        {subtitle ? (
          <>
            <span aria-hidden className="h-[18px] w-px flex-none bg-line" />
            <p className="truncate text-body text-ink-faint">{subtitle}</p>
          </>
        ) : null}
      </div>
      <div className="flex flex-none items-center gap-[9px]">
        {badge}
        {toolbar}
        {secondary}
        {primary}
      </div>
    </header>
  );
}

/**
 * Bascule segmentée de l'en-tête (Kanban / List, Month / Week / Day, 30 j / 90 j).
 *
 * Piste en teinte neutre de 3 px de padding, pastille active en surface : les maquettes
 * l'emploient à l'identique dans le Tracking, le Calendar et les Analytics.
 */
export function SegmentedControl<TValue extends string>({
  value,
  options,
  onChange,
  label,
  dense = false,
}: {
  value: TValue;
  options: readonly { readonly value: TValue; readonly label: string; readonly icon?: string }[];
  onChange: (value: TValue) => void;
  label: string;
  /** Variante sans icône des cartes d'analyse : 11,5 px et padding réduit. */
  dense?: boolean;
}) {
  return (
    <div
      role="group"
      aria-label={label}
      className="flex flex-none items-center gap-[3px] rounded-button bg-neutral-tint p-[3px]"
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
              "inline-flex items-center gap-1.5 rounded-pill font-medium transition-colors duration-150",
              dense ? "px-2.5 py-1 text-label" : "px-[11px] py-[5px] text-note",
              actif ? "bg-surface text-ink shadow-e1" : "text-ink-muted hover:text-ink",
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
