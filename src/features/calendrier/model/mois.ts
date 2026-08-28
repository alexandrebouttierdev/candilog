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
function iso(date: Date): string {
  const deuxChiffres = (valeur: number) => String(valeur).padStart(2, "0");
  return `${date.getFullYear()}-${deuxChiffres(date.getMonth() + 1)}-${deuxChiffres(date.getDate())}`;
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
  const isoAujourdhui = iso(aujourdhui);

  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(debut.getFullYear(), debut.getMonth(), debut.getDate() + index);
    const cle = iso(date);
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
