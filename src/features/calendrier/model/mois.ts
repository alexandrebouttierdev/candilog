/**
 * Grille d'un mois : six semaines de sept jours, comme dans les maquettes.
 *
 * Toujours 42 cellules, quel que soit le mois : une grille à hauteur variable ferait sauter
 * la mise en page d'un mois à l'autre, ce que le guide interdit.
 */

/** Une case de la grille. */
export interface JourGrille {
  /** Date `AAAA-MM-JJ`, clé de regroupement des événements. */
  readonly iso: string;
  /** Numéro du jour dans son mois. */
  readonly numero: number;
  /** La case appartient-elle au mois affiché ? Les autres sont estompées. */
  readonly dansLeMois: boolean;
  /** La case est-elle aujourd'hui ? */
  readonly aujourdhui: boolean;
}

/** Noms des jours, semaine commençant le lundi comme dans les maquettes. */
export const JOURS = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"] as const;

const MOIS = [
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
export function libelleMois(annee: number, mois: number): string {
  return `${MOIS[mois]} ${annee}`;
}

/** Date `AAAA-MM-JJ` d'un objet `Date`, en heure locale. */
export function isoLocal(date: Date): string {
  const deuxChiffres = (valeur: number) => String(valeur).padStart(2, "0");
  return `${date.getFullYear()}-${deuxChiffres(date.getMonth() + 1)}-${deuxChiffres(date.getDate())}`;
}

/** Reconstruit un `Date` local depuis une clé `AAAA-MM-JJ` (sans passer par UTC). */
export function dateDepuisIso(cle: string): Date {
  const [annee, mois, jour] = cle.split("-").map(Number);
  return new Date(annee ?? 0, (mois ?? 1) - 1, jour ?? 1);
}

/** Décale une clé ISO d'un nombre de jours, en heure locale. */
export function decalerJours(cle: string, pas: number): string {
  const date = dateDepuisIso(cle);
  date.setDate(date.getDate() + pas);
  return isoLocal(date);
}

/**
 * Les sept jours de la semaine (lundi → dimanche) qui contient `cle`.
 *
 * `dansLeMois` se lit par rapport au mois de `cle`, pour estomper le débordement
 * comme sur la grille mensuelle.
 */
export function joursDeLaSemaine(cle: string, aujourdhui = new Date()): JourGrille[] {
  const ancre = dateDepuisIso(cle);
  const decalage = (ancre.getDay() + 6) % 7;
  const debut = new Date(ancre.getFullYear(), ancre.getMonth(), ancre.getDate() - decalage);
  const isoAujourdhui = isoLocal(aujourdhui);

  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(debut.getFullYear(), debut.getMonth(), debut.getDate() + index);
    const jour = isoLocal(date);
    return {
      iso: jour,
      numero: date.getDate(),
      dansLeMois: date.getMonth() === ancre.getMonth(),
      aujourdhui: jour === isoAujourdhui,
    };
  });
}

/** Libellé « lundi 24 août 2026 » d'un jour. */
export function libelleJour(cle: string): string {
  return new Intl.DateTimeFormat("fr-FR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  }).format(dateDepuisIso(cle));
}

/** Libellé « 24 – 30 août 2026 » de la semaine contenant `cle`. */
export function libelleSemaine(cle: string): string {
  const jours = joursDeLaSemaine(cle);
  const debut = dateDepuisIso(jours[0]!.iso);
  const fin = dateDepuisIso(jours[6]!.iso);
  const jourMois = new Intl.DateTimeFormat("fr-FR", { day: "numeric", month: "short" });
  return `${jourMois.format(debut)} – ${jourMois.format(fin)} ${fin.getFullYear()}`;
}

/**
 * Construit les 42 cases du mois, débordant sur les mois voisins.
 *
 * `mois` est l'index JavaScript, de 0 à 11.
 */
export function grilleDuMois(annee: number, mois: number, aujourdhui = new Date()): JourGrille[] {
  const premier = new Date(annee, mois, 1);
  // `getDay` place dimanche à 0 ; la semaine des maquettes commence le lundi.
  const decalage = (premier.getDay() + 6) % 7;
  const debut = new Date(annee, mois, 1 - decalage);
  const isoAujourdhui = isoLocal(aujourdhui);

  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(debut.getFullYear(), debut.getMonth(), debut.getDate() + index);
    const cle = isoLocal(date);
    return {
      iso: cle,
      numero: date.getDate(),
      dansLeMois: date.getMonth() === mois,
      aujourdhui: cle === isoAujourdhui,
    };
  });
}

/** Bornes `AAAA-MM-JJ` de la grille, pour interroger le backend une seule fois. */
export function bornesDeLaGrille(annee: number, mois: number): { from: string; to: string } {
  const cases = grilleDuMois(annee, mois);
  return { from: cases[0]!.iso, to: cases[41]!.iso };
}

/** Mois précédent ou suivant, en gérant le passage d'année. */
export function decalerMois(
  annee: number,
  mois: number,
  pas: number,
): { annee: number; mois: number } {
  const date = new Date(annee, mois + pas, 1);
  return { annee: date.getFullYear(), mois: date.getMonth() };
}
