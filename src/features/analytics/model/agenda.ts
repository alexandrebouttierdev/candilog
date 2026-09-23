import type { AgendaItem } from "@/shared/types/generated/analytics";

export type Horizon = "late" | "today" | "week";

/** Date locale `AAAA-MM-JJ` : « aujourd'hui » est celui de l'utilisateur, pas l'UTC. */
export function localIso(date = new Date()): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** Nombre de jours entre deux dates `AAAA-MM-JJ` (b − a). */
export function daysBetween(a: string, b: string): number {
  const start = new Date(`${a}T00:00:00`);
  const end = new Date(`${b}T00:00:00`);
  return Math.round((end.getTime() - start.getTime()) / 86_400_000);
}

/**
 * Horizon d'une échéance (`screens/01-today-light.png`) : une relance datée d'avant
 * aujourd'hui est **en retard** ; un entretien passé n'arrive jamais ici (le backend ne
 * renvoie que ceux à venir).
 */
export function horizonOf(item: AgendaItem, today = localIso()): Horizon {
  const day = item.date.slice(0, 10);
  if (day < today) return "late";
  if (day === today) return "today";
  return "week";
}

/** Échéances réparties en horizons, dans l'ordre chronologique reçu. */
export function splitAgenda(items: readonly AgendaItem[], today = localIso()) {
  const groups: Record<Horizon, AgendaItem[]> = { late: [], today: [], week: [] };
  for (const item of items) groups[horizonOf(item, today)].push(item);
  return groups;
}

const JOURS = ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."];
const JOURS_LONGS = ["Dimanche", "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi"];
const MOIS = [
  "janvier", "février", "mars", "avril", "mai", "juin",
  "juillet", "août", "septembre", "octobre", "novembre", "décembre",
];

/** « Vendredi 12 septembre ». */
export function longDay(date = new Date()): string {
  return `${JOURS_LONGS[date.getDay()]} ${date.getDate()} ${MOIS[date.getMonth()]}`;
}

/** « 17 septembre » pour une date `AAAA-MM-JJ`. */
export function dayMonth(iso: string): string {
  const date = new Date(`${iso.slice(0, 10)}T00:00:00`);
  return `${date.getDate()} ${MOIS[date.getMonth()]}`;
}

/** « mar. 17 » : jour court de la semaine qui vient. */
export function shortWeekday(iso: string): string {
  const date = new Date(`${iso.slice(0, 10)}T00:00:00`);
  return `${JOURS[date.getDay()]} ${date.getDate()}`;
}
