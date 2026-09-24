import type { GridDay } from "../../model/month";
import { DAYS, isoLocal } from "../../model/month";
import type { CalendarEvent } from "../../model/event";
import { Button, EmptyState, StatusGlyph } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { eventStyle, eventText } from "./eventStyle";

/**
 * Vue semaine : une rangée de sept jours, mêmes pastilles que la grille mensuelle, sans
 * limite de nombre — une semaine a la hauteur de tout montrer.
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
  const today = isoLocal(new Date());
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="grid flex-none grid-cols-7 border-b border-bd-soft">
        {DAYS.map((day) => (
          <div key={day} className="px-2 py-1.5 text-caps tracking-[.04em] text-tx-6 uppercase">
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
                "flex min-h-0 flex-col gap-1 overflow-hidden border-r border-bd-soft p-2 last:border-r-0",
                day.today ? "bg-sel" : day.iso === selection ? "bg-hover" : "bg-panel",
              )}
            >
              <button
                type="button"
                onClick={() => onDayClick(day.iso)}
                aria-label={`Ajouter au ${day.number}`}
                aria-pressed={day.iso === selection}
                className={cn(
                  "flex h-[18px] min-w-[18px] flex-none items-center justify-center self-start rounded-r5 px-1 font-mono text-caps",
                  day.today ? "bg-ac font-semibold text-white" : "text-tx-3 hover:bg-hover",
                )}
              >
                {day.number}
              </button>
              <div className="flex min-h-0 flex-1 flex-col gap-[3px] overflow-y-auto">
                {events.map((event) => {
                  const style = eventStyle(event, today);
                  return (
                    <button
                      key={event.id}
                      type="button"
                      onClick={() => onEventClick(event)}
                      title={`${event.label}${event.detail ? ` — ${event.detail}` : ""}`}
                      className={cn(
                        "flex w-full flex-none flex-col items-start gap-0.5 rounded-r5 px-1.5 py-1 text-left text-tiny hover:brightness-95",
                        style.pill,
                      )}
                    >
                      <span className="flex w-full items-center gap-1.5">
                        <StatusGlyph tone={style.glyph} small />
                        <span className="min-w-0 flex-1 truncate">{eventText(event)}</span>
                        {event.time ? <span className="flex-none font-mono text-[10px] opacity-80">{event.time}</span> : null}
                      </span>
                      {event.detail ? <span className="w-full truncate pl-[17px] opacity-75">{event.label}</span> : null}
                    </button>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

/** Vue jour : liste des événements de la journée sélectionnée. */
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
  const today = isoLocal(new Date());
  if (events.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 items-start justify-center overflow-hidden pt-[min(12vh,90px)]">
        <EmptyState
          icon="event_available"
          title="Rien de prévu"
          description="Ajoutez un entretien ou une relance pour cette journée."
          action={
            <Button size="empty" onClick={() => onDayClick(day)}>
              Ajouter un entretien
            </Button>
          }
        />
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ul className="min-h-0 flex-1 overflow-y-auto px-3.5 py-3">
        {events.map((event) => {
          const style = eventStyle(event, today);
          return (
            <li key={event.id} className="mb-1.5">
              <button
                type="button"
                onClick={() => onEventClick(event)}
                className={cn("flex w-full items-center gap-3 rounded-r8 px-3 py-2.5 text-left hover:brightness-95", style.pill)}
              >
                <StatusGlyph tone={style.glyph} />
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-row font-medium">
                    {event.kind === "interview" ? "Entretien" : "Relance"} — {event.label}
                  </span>
                  {event.detail ? <span className="mt-0.5 block truncate text-sub opacity-80">{event.detail}</span> : null}
                </span>
                {event.time ? <span className="flex-none font-mono text-caps">{event.time}</span> : null}
              </button>
            </li>
          );
        })}
      </ul>
      <button
        type="button"
        onClick={() => onDayClick(day)}
        className="flex h-10 flex-none items-center justify-center gap-1.5 border-t border-bd-soft text-small font-medium text-ac-tx hover:bg-hover"
      >
        + Ajouter un entretien
      </button>
    </div>
  );
}
