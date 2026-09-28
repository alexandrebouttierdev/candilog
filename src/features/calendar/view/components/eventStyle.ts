import type { GlyphTone } from "@/shared/ui";
import type { CalendarEvent } from "../../model/event";

/**
 * Habillage d'un événement (`screens/04-calendar.png`) : un entretien est vert ; une relance
 * à venir reste neutre avec son glyphe ambre, en retard elle passe en rouge ; une relance
 * faite s'atténue. Table statique : Tailwind n'émet que les classes écrites en toutes lettres.
 */
export function eventStyle(event: CalendarEvent, today: string): { pill: string; glyph: GlyphTone } {
  if (event.kind === "interview") return { pill: "bg-tint-g-bg text-tint-g-tx", glyph: "g" };
  if (event.done) return { pill: "bg-group text-tx-5 line-through", glyph: "n" };
  if (event.day < today) return { pill: "bg-tint-c-bg text-tint-c-tx", glyph: "c" };
  return { pill: "bg-chip text-tx-2", glyph: "a" };
}

/** Texte d'une pastille : l'entreprise d'abord, l'intitulé à défaut. */
export function eventText(event: CalendarEvent): string {
  return event.detail ?? event.label;
}
