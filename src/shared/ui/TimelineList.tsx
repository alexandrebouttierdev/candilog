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
 * Les maquettes ne tracent pas de filet vertical : chaque entrée est une pastille de 8 px
 * cerclée d'un halo de 3 px dans la teinte de sa tonalité, suivie du titre et de la date.
 */
export function TimelineList({ entries }: { entries: readonly TimelineEntry[] }) {
  const dot: Record<Tone, string> = {
    neutral: "bg-ink-faint shadow-[0_0_0_3px_transparent]",
    accent: "bg-accent shadow-[0_0_0_3px_var(--color-accent-tint)]",
    success: "bg-success shadow-[0_0_0_3px_var(--color-success-tint)]",
    warning: "bg-warning shadow-[0_0_0_3px_var(--color-warning-tint)]",
    danger: "bg-danger shadow-[0_0_0_3px_var(--color-danger-tint)]",
  };

  return (
    <ol className="flex flex-col">
      {entries.map((entry) => (
        <li key={entry.id} className="flex items-start gap-[11px] pb-[13px] last:pb-0">
          <span
            aria-hidden="true"
            className={cn("mt-[5px] size-2 flex-none rounded-full", dot[entry.tone ?? "neutral"])}
          />
          <div className="min-w-0 flex-1">
            <p className="truncate text-body font-mid text-ink">{entry.title}</p>
            {entry.detail ? (
              <p className="mt-0.5 truncate text-label text-ink-faint">{entry.detail}</p>
            ) : null}
          </div>
          <time className="tabular flex-none text-label text-ink-faint">{entry.date}</time>
        </li>
      ))}
    </ol>
  );
}
