import type { ReactNode } from "react";
import { Icon } from "./Icon";

/**
 * En-tête d'écran : icône, titre, sous-titre, actions.
 *
 * Le guide SPECDESIGN impose une seule action primaire par écran ; `primary` est donc un
 * emplacement unique et non une liste, pour que la règle tienne par construction.
 */
export function PageHeader({
  icon,
  title,
  subtitle,
  secondary,
  primary,
}: {
  icon: string;
  title: string;
  subtitle?: string | undefined;
  secondary?: ReactNode;
  primary?: ReactNode;
}) {
  return (
    <header className="flex flex-none items-center gap-3 border-b border-line bg-surface-alt px-7 py-4">
      <span className="flex size-9 flex-none items-center justify-center rounded-card bg-accent-tint text-accent">
        <Icon name={icon} size={20} />
      </span>
      <div className="min-w-0 flex-1">
        <h1 className="truncate text-title">{title}</h1>
        {subtitle ? <p className="truncate text-meta text-ink-muted">{subtitle}</p> : null}
      </div>
      {secondary}
      {primary}
    </header>
  );
}
