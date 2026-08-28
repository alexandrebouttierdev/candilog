import { cn } from "@/shared/lib/cn";
import type { Tone } from "./StatusPill";

export interface TimelineEntry {
  readonly id: string;
  readonly title: string;
  readonly detail?: string;
  /** Date déjà formatée pour l'affichage (« 25 août »). */
  readonly date: string;
  readonly tone?: Tone;
}

/**
 * Chronologie d'activité d'une fiche.
 *
 * Le filet vertical est tracé par une bordure sur le conteneur de la pastille plutôt que par
 * un élément dédié : il s'arrête ainsi naturellement à la dernière entrée, sans dépasser
 * sous elle.
 */
export function TimelineList({ entries }: { entries: readonly TimelineEntry[] }) {
  const dot: Record<Tone, string> = {
    neutral: "bg-neutral-tint text-ink-faint",
    accent: "bg-accent-tint text-accent",
    success: "bg-success-tint text-success",
    warning: "bg-warning-tint text-warning",
    danger: "bg-danger-tint text-danger",
  };

  return (
    <ol className="flex flex-col">
      {entries.map((entry, index) => (
        <li key={entry.id} className="flex gap-3">
          <div
            className={cn(
              "flex flex-col items-center",
              index < entries.length - 1 && "after:mt-1 after:w-px after:flex-1 after:bg-line",
            )}
          >
            <span
              className={cn("mt-1 size-2.5 rounded-full", dot[entry.tone ?? "neutral"])}
              aria-hidden="true"
            />
          </div>
          <div className="min-w-0 flex-1 pb-4">
            <div className="flex items-baseline gap-2">
              <p className="min-w-0 flex-1 truncate text-body text-ink">{entry.title}</p>
              <time className="tabular flex-none text-meta text-ink-faint">{entry.date}</time>
            </div>
            {entry.detail ? (
              <p className="truncate text-meta text-ink-muted">{entry.detail}</p>
            ) : null}
          </div>
        </li>
      ))}
    </ol>
  );
}
