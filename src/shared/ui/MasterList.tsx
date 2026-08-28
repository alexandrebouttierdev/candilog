import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";
import { controlClasses } from "./FormField";

/**
 * Liste maître d'un écran maître-détail : barre de recherche, éléments, pied paginé.
 *
 * Générique parce que Relations l'utilise deux fois — entreprises et réseau — et que
 * Candidatures la réutilisera. Ce qu'elle ne fait pas : filtrer. La recherche est remontée
 * à l'appelant, qui la transmet au backend.
 */
export function MasterList({
  search,
  searchPlaceholder,
  onSearchChange,
  toolbar,
  children,
  footer,
}: {
  search: string;
  searchPlaceholder: string;
  onSearchChange: (valeur: string) => void;
  /** Filtres additionnels, placés sous la recherche. */
  toolbar?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
}) {
  return (
    <div className="flex w-[340px] flex-none flex-col border-r border-line bg-surface">
      <div className="flex flex-none flex-col gap-2 border-b border-line p-3">
        <div className="relative">
          <Icon
            name="search"
            size={16}
            className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-faint"
          />
          <input
            type="search"
            value={search}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder={searchPlaceholder}
            aria-label={searchPlaceholder}
            className={controlClasses(false, "pl-8")}
          />
        </div>
        {toolbar}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>

      {footer}
    </div>
  );
}

/**
 * Élément de liste maître : initiales, titre, sous-titre, méta.
 *
 * `button` et non `div` cliquable : la sélection au clavier et la restitution par les
 * lecteurs d'écran en dépendent, et le guide impose une cible d'au moins 44 px.
 */
export function MasterListItem({
  initials,
  title,
  subtitle,
  meta,
  selected,
  onSelect,
}: {
  initials: string;
  title: string;
  subtitle?: string | undefined;
  meta?: ReactNode;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      aria-current={selected ? "true" : undefined}
      className={cn(
        "flex min-h-row w-full items-center gap-2.5 border-b border-line px-3 py-2 text-left",
        "transition-colors duration-150",
        selected ? "bg-accent-tint" : "hover:bg-neutral-tint",
      )}
    >
      <span
        aria-hidden="true"
        className={cn(
          "flex size-8 flex-none items-center justify-center rounded-pill text-meta font-medium",
          selected ? "bg-accent text-white" : "bg-neutral-tint text-ink-muted",
        )}
      >
        {initials}
      </span>
      <span className="min-w-0 flex-1">
        <span
          className={cn(
            "block truncate text-body",
            selected ? "font-medium text-accent" : "text-ink",
          )}
        >
          {title}
        </span>
        {subtitle ? (
          <span className="block truncate text-meta text-ink-muted">{subtitle}</span>
        ) : null}
      </span>
      {meta}
    </button>
  );
}

/** Initiales d'un nom, pour la pastille des listes maîtresses. */
export function initiales(...parties: Array<string | null | undefined>): string {
  const lettres = parties
    .map((partie) => partie?.trim()?.[0])
    .filter((lettre): lettre is string => Boolean(lettre));
  return (lettres.length > 0 ? lettres.join("") : "?").slice(0, 2).toUpperCase();
}
