import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/**
 * List maître d'un écran maître-détail : en-tête compté, éléments, pied paginé.
 *
 * Géométrie des maquettes Relations : colonne de 37 % de la largeur sur fond surface,
 * en-tête à filet de 13 px / 18 px portant le titre et le compte, éléments dans une
 * gouttière de 9 px, pied de pagination en `surface-alt`.
 *
 * Générique parce que Relations l'utilise deux fois — entreprises et réseau. Ce qu'elle ne
 * fait pas : filtrer. La recherche est remontée à l'appelant, qui la transmet au backend.
 */
export function MasterList({
  title,
  count,
  toolbar,
  children,
  footer,
}: {
  title: string;
  /** Résumé affiché à droite du titre : « 7 entreprises · 10 candidatures ». */
  count?: ReactNode;
  /** Filters additionnels, placés sous l'en-tête. */
  toolbar?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
}) {
  return (
    <div className="flex w-[37%] min-w-[300px] flex-none flex-col border-r border-line bg-surface">
      <div className="flex flex-none items-center justify-between gap-3 border-b border-line px-[18px] py-[13px]">
        <span className="truncate text-body font-semibold text-ink">{title}</span>
        {count ? <span className="flex-none text-label text-ink-faint">{count}</span> : null}
      </div>

      {toolbar ? (
        <div className="flex flex-none items-center gap-2 border-b border-line px-[9px] py-[9px]">
          {toolbar}
        </div>
      ) : null}

      <div className="min-h-0 flex-1 overflow-y-auto p-[9px]">{children}</div>

      {footer}
    </div>
  );
}

/**
 * Élément de liste maître : pastille d'initiales, titre, sous-titre, méta.
 *
 * `button` et non `div` cliquable : la sélection au clavier et la restitution par les
 * lecteurs d'écran en dépendent, et le guide impose une cible d'au moins 44 px.
 */
export function MasterListItem({
  initials,
  round = false,
  title,
  subtitle,
  meta,
  selected,
  onSelect,
}: {
  initials: string;
  /** Pastille ronde pour les personnes, carrée pour les organisations. */
  round?: boolean;
  title: string;
  subtitle?: ReactNode;
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
        "mb-1 flex w-full items-center gap-[11px] rounded-tile border px-3 py-[11px] text-left",
        "transition-colors duration-150",
        selected
          ? "border-accent-border bg-accent-tint"
          : "border-transparent hover:bg-neutral-tint",
      )}
    >
      <span
        aria-hidden="true"
        className={cn(
          "flex size-8 flex-none items-center justify-center text-label font-strong",
          round ? "rounded-full" : "rounded-field",
          selected ? "bg-accent text-white" : "bg-neutral-tint text-ink-muted",
        )}
      >
        {initials}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-item font-semibold text-ink">{title}</span>
        {subtitle ? (
          <span className="mt-px block truncate text-label text-ink-faint">{subtitle}</span>
        ) : null}
      </span>
      {meta}
    </button>
  );
}

/**
 * Étiquette de droite d'un élément de liste maître : puce de 11 px à icône, teintée
 * selon le sens porté (rôle d'un contact, type d'une entreprise).
 */
export function MasterListTag({
  icon,
  tone = "neutral",
  children,
}: {
  icon: string;
  tone?: "neutral" | "accent" | "success";
  children: ReactNode;
}) {
  const tones = {
    neutral: "bg-neutral-tint text-ink-muted",
    accent: "bg-accent-tint text-accent",
    success: "bg-success-tint text-success",
  } as const;

  return (
    <span
      className={cn(
        "inline-flex flex-none items-center gap-1 rounded-chip px-[7px] py-[3px] text-meta font-mid",
        tones[tone],
      )}
    >
      <Icon name={icon} size={13} />
      {children}
    </span>
  );
}

/** Initials d'un nom, pour la pastille des listes maîtresses. */
export function initials(...parts: Array<string | null | undefined>): string {
  const cover_letters = parts
    .map((part) => part?.trim()?.[0])
    .filter((cover_letter): cover_letter is string => Boolean(cover_letter));
  return (cover_letters.length > 0 ? cover_letters.join("") : "?").slice(0, 2).toUpperCase();
}

/** Initials d'une raison sociale : deux premières initiales de mots (« Nova Digital » → ND). */
export function wordInitials(name: string | null | undefined): string {
  return initials(...(name ?? "").split(/\s+/).filter(Boolean).slice(0, 2));
}
