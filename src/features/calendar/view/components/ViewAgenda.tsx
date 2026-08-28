import type { GridDay } from "../../model/month";
import { DAYS } from "../../model/month";
import type { CalendarEvent } from "../../model/event";
import { Button, EmptyState, Icon } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

const PASTILLE: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * View semaine : une rangée de sept jours, mêmes pastilles que la grille mensuelle.
 */
export function ViewWeek({
  days,
  parDay,
  selection,
  onDayClick,
  onEventClick,
}: {
  days: readonly GridDay[];
  parDay: Map<string, CalendarEvent[]>;
  selection: string;
  onDayClick: (iso: string) => void;
  onEventClick: (event: CalendarEvent) => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
      <div className="grid flex-none grid-cols-7 border-b border-line bg-surface-alt">
        {DAYS.map((day) => (
          <div key={day} className="px-2 py-2 text-center text-eyebrow uppercase text-ink-faint">
            {day}
          </div>
        ))}
      </div>
      <div className="grid min-h-0 flex-1 grid-cols-7">
        {days.map((day) => {
          const events = parDay.get(day.iso) ?? [];
          return (
            <div
              key={day.iso}
              className={cn(
                "flex min-h-0 flex-col gap-1 overflow-hidden border-r border-line p-2 last:border-r-0",
                day.iso === selection ? "bg-accent-tint/40" : day.in_month ? "bg-surface" : "bg-surface-alt",
              )}
            >
              <button
                type="button"
                onClick={() => onDayClick(day.iso)}
                aria-label={`Ajouter au ${day.number}`}
                aria-pressed={day.iso === selection}
                className={cn(
                  "tabular flex size-6 flex-none items-center justify-center rounded-pill text-meta",
                  day.today
                    ? "bg-accent font-medium text-white"
                    : day.in_month
                      ? "text-ink-muted hover:bg-neutral-tint hover:text-ink"
                      : "text-ink-faint hover:bg-neutral-tint",
                )}
              >
                {day.number}
              </button>
              <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto">
                {events.map((event) => (
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
                    {event.time ? <span className="tabular flex-none">{event.time}</span> : null}
                    <span className="truncate">{event.label}</span>
                  </button>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

/**
 * View jour : liste des événements de la journée sélectionnée.
 */
export function ViewDay({
  events,
  onDayClick,
  onEventClick,
  day,
}: {
  events: readonly CalendarEvent[];
  day: string;
  onDayClick: (iso: string) => void;
  onEventClick: (event: CalendarEvent) => void;
}) {
  if (events.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center overflow-hidden rounded-card border border-line bg-surface">
        <EmptyState
          icon="event_available"
          title="Rien de prévu"
          description="Ajoutez un entretien ou une relance pour cette journée."
          action={<Button icon="add" onClick={() => onDayClick(day)}>Ajouter un entretien</Button>}
        />
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
      <ul className="min-h-0 flex-1 overflow-y-auto p-3">
        {events.map((event) => (
          <li key={event.id} className="mb-1.5">
            <button
              type="button"
              onClick={() => onEventClick(event)}
              className={cn(
                "flex w-full items-center gap-3 rounded-[10px] px-3 py-2.5 text-left",
                "transition-colors duration-150 hover:brightness-95",
                PASTILLE[event.tone],
              )}
            >
              <Icon name={event.icon} size={18} className="flex-none" />
              <span className="min-w-0 flex-1">
                <span className="block truncate text-body font-medium">{event.label}</span>
                {event.detail ? (
                  <span className="mt-0.5 block truncate text-meta opacity-80">{event.detail}</span>
                ) : null}
              </span>
              {event.time ? <span className="tabular flex-none text-meta">{event.time}</span> : null}
            </button>
          </li>
        ))}
      </ul>
      <button
        type="button"
        onClick={() => onDayClick(day)}
        className="flex h-11 flex-none items-center justify-center gap-1.5 border-t border-line text-body font-medium text-accent hover:bg-accent-tint"
      >
        <Icon name="add" size={16} />
        Ajouter un entretien
      </button>
    </div>
  );
}
