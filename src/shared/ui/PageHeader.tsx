import type { ReactNode } from "react";
import { Icon } from "./Icon";

/**
 * En-tête d'écran : icône, titre et sous-titre en ligne, actions à droite.
 *
 * Reprend le chrome des maquettes SPECDESIGN (titre 20 px, filet vertical, sous-titre
 * tertiaire). Le guide n'admet qu'une seule action `primary` par écran.
 */
export function PageHeader({
  icon,
  title,
  subtitle,
  badge,
  secondary,
  primary,
}: {
  icon: string;
  title: string;
  subtitle?: string | undefined;
  badge?: ReactNode;
  secondary?: ReactNode;
  primary?: ReactNode;
}) {
  return (
    <header className="flex flex-none items-center justify-between gap-5 border-b border-line bg-surface px-7 py-[17px]">
      <div className="flex min-w-0 items-center gap-3.5">
        <span className="flex size-[34px] flex-none items-center justify-center rounded-[10px] bg-accent-tint text-accent">
          <Icon name={icon} size={19} />
        </span>
        <h1 className="truncate text-title tracking-tight">{title}</h1>
        {subtitle ? (
          <>
            <span aria-hidden className="h-[18px] w-px flex-none bg-line" />
            <p className="truncate text-body text-ink-faint">{subtitle}</p>
          </>
        ) : null}
      </div>
      <div className="flex flex-none items-center gap-2.5">
        {badge}
        {secondary}
        {primary}
      </div>
    </header>
  );
}
