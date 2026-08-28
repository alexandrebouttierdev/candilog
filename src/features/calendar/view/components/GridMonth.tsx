import type { GridDay } from "../../model/month";
import { DAYS } from "../../model/month";
import type { CalendarEvent } from "../../model/event";
import { Icon } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/** Count d'événements affichés par case avant le repli « +N ». */
const MAX_PAR_CELL = 3;

/**
 * Classes des pastilles d'événement, par tonalité.
 *
 * Table statique et non interpolation : Tailwind n'émet que les classes qu'il trouve
 * littéralement dans les sources.
 */
const PASTILLE: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Grid mensuelle : six semaines de sept jours.
 *
 * Le nombre de cases est fixe (42) : une grille à hauteur variable ferait sauter la mise en
 * page d'un mois à l'autre. Les jours hors du mois affiché sont estompés, sans être masqués
 * — ils portent de vrais événements.
 */
export function GridMonth({
  cells,
  parDay,
  onDayClick,
  onEventClick,
}: {
  cells: readonly GridDay[];
  parDay: Map<string, CalendarEvent[]>;
  onDayClick: (iso: string) => void;
  onEventClick: (event: CalendarEvent) => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
      <div className="grid flex-none grid-cols-7 border-b border-line bg-surface-alt">
        {DAYS.map((day) => (
          <div key={day} className="px-2 py-2 text-eyebrow uppercase text-ink-faint">
            {day}
          </div>
        ))}
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6">
        {cells.map((day) => {
          const events = parDay.get(day.iso) ?? [];
          const visibles = events.slice(0, MAX_PAR_CELL);
          const surplus = events.length - visibles.length;

          return (
            <div
              key={day.iso}
              className={cn(
                "flex min-h-0 flex-col gap-1 overflow-hidden border-r border-b border-line p-1.5",
                "last-in-row:border-r-0",
                day.in_month ? "bg-surface" : "bg-surface-alt",
              )}
            >
              <button
                type="button"
                onClick={() => onDayClick(day.iso)}
                aria-label={`Ajouter au ${day.number}`}
                className={cn(
                  "tabular flex size-6 flex-none items-center justify-center rounded-pill text-meta",
                  "transition-colors duration-150",
                  day.today
                    ? "bg-accent font-medium text-white"
                    : day.in_month
                      ? "text-ink-muted hover:bg-neutral-tint hover:text-ink"
                      : "text-ink-faint hover:bg-neutral-tint",
                )}
              >
                {day.number}
              </button>

              <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-hidden">
                {visibles.map((event) => (
                  <button
                    key={event.id}
                    type="button"
                    onClick={() => onEventClick(event)}
                    title={`${event.label}${event.detail ? ` — ${event.detail}` : ""}`}
                    className={cn(
                      "flex w-full items-center gap-1 rounded-pill px-1.5 py-0.5 text-left text-meta",
                      "transition-[filter] duration-150 hover:brightness-95",
                      PASTILLE[event.tone],
                    )}
                  >
                    <Icon name={event.icon} size={11} className="flex-none" />
                    {event.time ? (
                      <span className="tabular flex-none">{event.time}</span>
                    ) : null}
                    <span className="truncate">{event.label}</span>
                  </button>
                ))}

                {surplus > 0 ? (
                  <span className="px-1.5 text-meta text-ink-faint">+{surplus}</span>
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
