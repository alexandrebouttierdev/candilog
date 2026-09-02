import { Children, useEffect, useRef, useState, type ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import { useDismissable } from "@/shared/hooks/useDismissable";

/**
 * Toolbar de tableau : recherche, filtres, chips, actions (50 px).
 */
export function FilterBar({ children, actions }: { children: ReactNode; actions?: ReactNode }) {
  return (
    <div className="flex min-h-[50px] flex-none flex-wrap items-center gap-2 border-b border-line-soft px-3 py-2.5">
      {children}
      <span className="flex-1" />
      {actions ? <div className="flex flex-none flex-wrap items-center gap-2">{actions}</div> : null}
    </div>
  );
}

/** Bouton « Filtres » de toolbar, 30 px, pastille de count si des critères sont actifs. */
export function FilterTrigger({
  count,
  pressed,
  onClick,
}: {
  count: number;
  pressed: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-haspopup="dialog"
      aria-expanded={pressed}
      aria-label={count > 0 ? `Filtres, ${count} actifs` : "Filtres"}
      onClick={onClick}
      className={cn(
        "inline-flex h-control items-center gap-1.5 rounded-button border px-[11px] text-item font-semibold",
        "transition-colors duration-hover",
        pressed || count > 0
          ? "border-accent-border bg-accent-tint text-accent-text-soft"
          : "border-control-strong bg-fill text-ink hover:bg-fill-hover",
      )}
    >
      <Icon name="filter_list" size={16} />
      Filtres
      {count > 0 ? (
        <span className="tabular text-label">{count}</span>
      ) : (
        <Icon name="expand_more" size={15} className="text-ink-faint" />
      )}
    </button>
  );
}

/** Popover accroché sous le déclencheur, compact ou élargi selon la densité du contenu. */
export function FilterMenu({
  count,
  children,
}: {
  count: number;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);

  useDismissable({ open, onDismiss: () => setOpen(false) });

  useEffect(() => {
    if (!open) return;
    const onPointer = (event: MouseEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onPointer);
    return () => document.removeEventListener("mousedown", onPointer);
  }, [open]);

  return (
    <div ref={root} className="relative">
      <FilterTrigger count={count} pressed={open} onClick={() => setOpen((value) => !value)} />
      {open ? (
        <div
          role="dialog"
          aria-label="Filtres"
          className="glass-popover absolute top-[calc(100%+6px)] left-0 z-40 max-h-[min(70vh,640px)] w-max min-w-[230px] max-w-[calc(100vw-2rem)] overflow-y-auto overscroll-contain rounded-overlay border border-overlay p-3 shadow-overlay sm:max-w-[640px]"
        >
          {children}
        </div>
      ) : null}
    </div>
  );
}

const FILTER_GROUP_OPTION_LIMIT = 6;

export function FilterGroup({ label, children }: { label: string; children: ReactNode }) {
  const [expanded, setExpanded] = useState(false);
  const options = Children.toArray(children);
  const hasMore = options.length > FILTER_GROUP_OPTION_LIMIT;
  const visibleOptions = expanded ? options : options.slice(0, FILTER_GROUP_OPTION_LIMIT);

  return (
    <div className="mb-3 last:mb-0">
      <p className="mb-1.5 text-eyebrow uppercase text-ink-label">{label}</p>
      <div className="flex flex-wrap gap-1.5">{visibleOptions}</div>
      {hasMore ? (
        <button
          type="button"
          aria-expanded={expanded}
          aria-label={`${expanded ? "Voir moins" : "Voir plus"} pour ${label}`}
          onClick={() => setExpanded((value) => !value)}
          className="mt-1.5 inline-flex h-6 items-center text-label font-semibold text-accent-text-soft hover:text-accent"
        >
          {expanded ? "Voir moins" : "Voir plus"}
        </button>
      ) : null}
    </div>
  );
}

/** Option 25 px du popover : inactive neutre, active teinte accent. */
export function FilterOption({
  label,
  selected,
  onSelect,
}: {
  label: string;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onSelect}
      className={cn(
        "inline-flex h-[25px] items-center whitespace-nowrap rounded-chip border px-[9px] text-label font-medium",
        "transition-colors duration-hover",
        selected
          ? "border-accent-border bg-accent-tint-12 text-accent-text-soft"
          : "border-control bg-fill text-ink-muted hover:bg-fill-hover",
      )}
    >
      {label}
    </button>
  );
}

/** Chip actif « Champ · Valeur », retrait 20 × 20. */
export function ActiveFilterChip({
  field,
  value,
  onRemove,
}: {
  field: string;
  value: string;
  onRemove: () => void;
}) {
  return (
    <span className="inline-flex h-chip items-center gap-1.5 rounded-chip border border-accent-border bg-accent-tint py-0 pr-1 pl-[9px] text-label font-medium text-accent-text-soft">
      <span className="max-w-[180px] truncate">
        {field} · {value}
      </span>
      <button
        type="button"
        aria-label={`Retirer le filtre ${field} ${value}`}
        onClick={onRemove}
        className="flex size-5 flex-none items-center justify-center rounded-[4px] text-accent-text-soft hover:bg-accent-tint-12"
      >
        <Icon name="close" size={14} />
      </button>
    </span>
  );
}

export function ClearFiltersButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="text-label font-semibold text-ink-muted hover:text-ink"
    >
      Tout effacer
    </button>
  );
}
