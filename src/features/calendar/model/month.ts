/**
 * Grid d'un mois : six semaines de sept jours, comme dans les maquettes.
 *
 * Toujours 42 cellules, quel que soit le mois : une grille à hauteur variable ferait sauter
 * la mise en page d'un mois à l'autre, ce que le guide interdit.
 */

/** Une case de la grille. */
export interface GridDay {
  /** Date `AAAA-MM-JJ`, clé de regroupement des événements. */
  readonly iso: string;
  /** Numéro du jour dans son mois. */
  readonly number: number;
  /** La case appartient-elle au mois affiché ? Les autres sont estompées. */
  readonly in_month: boolean;
  /** La case est-elle aujourd'hui ? */
  readonly today: boolean;
}

/** Names des jours, semaine commençant le lundi comme dans les maquettes. */
export const DAYS = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"] as const;

const MONTHS = [
  "janvier",
  "février",
  "mars",
  "avril",
  "mai",
  "juin",
  "juillet",
  "août",
  "septembre",
  "octobre",
  "novembre",
  "décembre",
] as const;

/** Libellé « août 2026 » d'un mois. */
export function monthLabel(year: number, month: number): string {
  return `${MONTHS[month]} ${year}`;
}

/** Date `AAAA-MM-JJ` d'un objet `Date`, en heure locale. */
export function isoLocal(date: Date): string {
  const deuxChiffres = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${deuxChiffres(date.getMonth() + 1)}-${deuxChiffres(date.getDate())}`;
}

/** Reconstruit un `Date` local depuis une clé `AAAA-MM-JJ` (sans passer par UTC). */
export function dateFromIso(cle: string): Date {
  const [year, month, day] = cle.split("-").map(Number);
  return new Date(year ?? 0, (month ?? 1) - 1, day ?? 1);
}

/** Décale une clé ISO d'un nombre de jours, en heure locale. */
export function decalerDays(cle: string, pas: number): string {
  const date = dateFromIso(cle);
  date.setDate(date.getDate() + pas);
  return isoLocal(date);
}

/**
 * Les sept jours de la semaine (lundi → dimanche) qui contient `cle`.
 *
 * `dansLeMois` se lit par rapport au mois de `cle`, pour estomper le débordement
 * comme sur la grille mensuelle.
 */
export function daysDeLaWeek(cle: string, today = new Date()): GridDay[] {
  const ancre = dateFromIso(cle);
  const decalage = (ancre.getDay() + 6) % 7;
  const start = new Date(ancre.getFullYear(), ancre.getMonth(), ancre.getDate() - decalage);
  const isoToday = isoLocal(today);

  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(start.getFullYear(), start.getMonth(), start.getDate() + index);
    const day = isoLocal(date);
    return {
      iso: day,
      number: date.getDate(),
      in_month: date.getMonth() === ancre.getMonth(),
      today: day === isoToday,
    };
  });
}

/** Libellé « lundi 24 août 2026 » d'un jour. */
export function labelDay(cle: string): string {
  return new Intl.DateTimeFormat("fr-FR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  }).format(dateFromIso(cle));
}

/** Libellé « 24 – 30 août 2026 » de la semaine contenant `cle`. */
export function labelWeek(cle: string): string {
  const days = daysDeLaWeek(cle);
  const start = dateFromIso(days[0]!.iso);
  const end = dateFromIso(days[6]!.iso);
  const day_month = new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "short" });
  return `${day_month.format(start)} – ${day_month.format(end)} ${end.getFullYear()}`;
}

/**
 * Construit les 42 cases du mois, débordant sur les mois voisins.
 *
 * `mois` est l'index JavaScript, de 0 à 11.
 */
export function gridDuMonth(year: number, month: number, today = new Date()): GridDay[] {
  const first = new Date(year, month, 1);
  // `getDay` place dimanche à 0 ; la semaine des maquettes commence le lundi.
  const decalage = (first.getDay() + 6) % 7;
  const start = new Date(year, month, 1 - decalage);
  const isoToday = isoLocal(today);

  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(start.getFullYear(), start.getMonth(), start.getDate() + index);
    const cle = isoLocal(date);
    return {
      iso: cle,
      number: date.getDate(),
      in_month: date.getMonth() === month,
      today: cle === isoToday,
    };
  });
}

/** Bounds `AAAA-MM-JJ` de la grille, pour interroger le backend une seule fois. */
export function gridBounds(year: number, month: number): { from: string; to: string } {
  const cells = gridDuMonth(year, month);
  return { from: cells[0]!.iso, to: cells[41]!.iso };
}

/** Month précédent ou suivant, en gérant le passage d'année. */
export function decalerMonth(
  year: number,
  month: number,
  pas: number,
): { year: number; month: number } {
  const date = new Date(year, month + pas, 1);
  return { year: date.getFullYear(), month: date.getMonth() };
}
