/**
 * Conversions de date entre la saisie utilisateur, la base et l'affichage.
 *
 * Centralisées ici parce que trois features les partagent — candidatures, entretiens,
 * relances — et que chaque duplication serait une occasion de diverger sur un format que la
 * base compare comme une chaîne.
 */

/** Format de date saisi et affiché, conformément aux maquettes. */
export const FORMAT_DATE = "JJ-MM-AAAA";

/** Convertit une date `JJ-MM-AAAA` en `AAAA-MM-JJ`, ou `null` si elle n'existe pas. */
export function versDateIso(saisie: string): string | null {
  const correspondance = /^(\d{2})-(\d{2})-(\d{4})$/.exec(saisie.trim());
  if (!correspondance) return null;
  const [, jour, mois, annee] = correspondance;
  const iso = `${annee}-${mois}-${jour}`;
  // `Date` accepte le 31 février en le décalant au 3 mars : comparer la valeur relue est le
  // seul moyen de refuser une date qui n'existe pas.
  const date = new Date(`${iso}T00:00:00Z`);
  return Number.isNaN(date.getTime()) || date.toISOString().slice(0, 10) !== iso ? null : iso;
}

/** Convertit une date `AAAA-MM-JJ` en `JJ-MM-AAAA` pour l'affichage. */
export function versDateAffichee(iso: string): string {
  const correspondance = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
  if (!correspondance) return iso;
  const [, annee, mois, jour] = correspondance;
  return `${jour}-${mois}-${annee}`;
}

/**
 * Date d'affichage des maquettes : « 02 août », « 02 août 2026 » avec l'année.
 *
 * Distincte de `versDateAffichee`, qui reste le format **de saisie** `JJ-MM-AAAA` des
 * champs de formulaire : les maquettes n'écrivent jamais une date en chiffres ailleurs
 * que dans un champ.
 */
export function versDateLongue(iso: string, avecAnnee = false): string {
  const jour = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
  if (!jour) return iso;
  const date = new Date(`${jour[1]}-${jour[2]}-${jour[3]}T00:00:00`);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat("fr-FR", {
    day: "2-digit",
    month: "long",
    ...(avecAnnee ? { year: "numeric" } : {}),
  }).format(date);
}

/** Valide une heure `HH:MM` sur 24 heures. */
export function heureValide(saisie: string): boolean {
  return /^([01]\d|2[0-3]):[0-5]\d$/.test(saisie.trim());
}

/**
 * Compose une date et une heure locales en horodatage `RFC 3339`.
 *
 * L'entretien est stocké avec son décalage horaire : sans lui, un entretien saisi à 14 h
 * s'afficherait à 12 h ou 16 h selon le fuseau où la base est relue.
 */
export function versHorodatage(dateSaisie: string, heureSaisie: string): string | null {
  const iso = versDateIso(dateSaisie);
  if (iso === null || !heureValide(heureSaisie)) return null;

  const [heures, minutes] = heureSaisie.trim().split(":").map(Number);
  const locale = new Date(
    Number(iso.slice(0, 4)),
    Number(iso.slice(5, 7)) - 1,
    Number(iso.slice(8, 10)),
    heures,
    minutes,
  );

  // `getTimezoneOffset` renvoie des minutes **à retrancher** de l'heure locale pour obtenir
  // UTC : son signe est donc l'inverse de celui du décalage écrit dans l'horodatage.
  const decalage = -locale.getTimezoneOffset();
  const signe = decalage >= 0 ? "+" : "-";
  const absolu = Math.abs(decalage);
  const deuxChiffres = (valeur: number) => String(valeur).padStart(2, "0");

  return (
    `${iso}T${deuxChiffres(heures!)}:${deuxChiffres(minutes!)}:00` +
    `${signe}${deuxChiffres(Math.floor(absolu / 60))}:${deuxChiffres(absolu % 60)}`
  );
}

/** Extrait la date `JJ-MM-AAAA` d'un horodatage. */
export function dateDepuisHorodatage(horodatage: string): string {
  return versDateAffichee(horodatage.slice(0, 10));
}

/** Extrait l'heure `HH:MM` d'un horodatage. */
export function heureDepuisHorodatage(horodatage: string): string {
  return horodatage.slice(11, 16);
}

/** Jour `AAAA-MM-JJ` d'un horodatage ou d'une date, pour regrouper par journée. */
export function jourDe(valeur: string): string {
  return valeur.slice(0, 10);
}

/**
 * Jours écoulés depuis une date `AAAA-MM-JJ`, jamais négatif.
 *
 * Les maquettes affichent l'ancienneté d'une candidature plutôt que sa date d'envoi sur les
 * cartes du Kanban : « 12 j » se lit d'un coup d'œil là où une date demande un calcul.
 */
export function joursDepuis(iso: string): number {
  const date = new Date(`${iso.slice(0, 10)}T00:00:00`);
  if (Number.isNaN(date.getTime())) return 0;
  const aujourdhui = new Date();
  aujourdhui.setHours(0, 0, 0, 0);
  return Math.max(0, Math.round((aujourdhui.getTime() - date.getTime()) / 86_400_000));
}
