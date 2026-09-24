import type { GridDay } from "../../model/month";
import { DAYS, isoLocal } from "../../model/month";
import type { CalendarEvent } from "../../model/event";
import { StatusGlyph } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { eventStyle, eventText } from "./eventStyle";

/** Nombre d'événements affichés par case avant le repli « +N ». */
const MAX_PAR_CELL = 2;

/**
 * Grille mensuelle (`screens/04-calendar.png`) : six semaines de sept jours, filets fins,
 * aujourd'hui sur fond de sélection. Le nombre de cases est fixe (42) : une grille à hauteur
 * variable ferait sauter la mise en page d'un mois à l'autre. Les jours hors du mois sont
 * estompés, sans être masqués — ils portent de vrais événements.
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

      <div className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6">
        {cells.map((day) => {
          const events = parDay.get(day.iso) ?? [];
          const visibles = events.slice(0, MAX_PAR_CELL);
          const surplus = events.length - visibles.length;
          const past = day.iso < today;

          return (
            <div
              key={day.iso}
              className={cn(
                "flex min-h-0 flex-col gap-1 overflow-hidden border-r border-b border-bd-soft p-1.5 [&:nth-child(7n)]:border-r-0",
                day.today ? "bg-sel" : day.in_month ? "bg-panel" : "bg-group",
              )}
            >
              <button
                type="button"
                onClick={() => onDayClick(day.iso)}
                aria-label={`Ajouter au ${day.number}`}
                className={cn(
                  "flex h-[18px] min-w-[18px] flex-none items-center justify-center self-start rounded-r5 px-1 font-mono text-caps",
                  day.today
                    ? "bg-ac font-semibold text-white"
                    : !day.in_month
                      ? "text-tx-7 hover:bg-hover"
                      : past
                        ? "text-tx-6 hover:bg-hover"
                        : "text-tx-3 hover:bg-hover",
                )}
              >
                {day.number}
              </button>

              <div className="flex min-h-0 flex-1 flex-col gap-[3px] overflow-hidden">
                {visibles.map((event) => {
                  const style = eventStyle(event, today);
                  return (
                    <button
                      key={event.id}
                      type="button"
                      onClick={() => onEventClick(event)}
                      title={`${event.kind === "interview" ? "Entretien" : "Relance"} · ${event.label}${event.detail ? ` — ${event.detail}` : ""}`}
                      className={cn(
                        "flex h-5 w-full flex-none items-center gap-1.5 rounded-r5 px-1.5 text-left text-tiny hover:brightness-95",
                        style.pill,
                      )}
                    >
                      <StatusGlyph tone={style.glyph} small />
                      <span className="min-w-0 flex-1 truncate">{eventText(event)}</span>
                      {event.time ? <span className="flex-none font-mono text-[10px] opacity-80">{event.time}</span> : null}
                    </button>
                  );
                })}

                {surplus > 0 ? <span className="px-1.5 font-mono text-[10px] text-tx-5">+{surplus}</span> : null}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
