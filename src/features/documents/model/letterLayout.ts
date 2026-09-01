/**
 * Composition d'une lettre : ce que le PDF imprime autour du corps rédigé.
 *
 * Ces règles sont le jumeau de `features/documents/application/cover_letter_document.rs` et
 * de `infrastructure/pdf/cover_letter_pdf.rs`. Elles vivent ici pour que l'aperçu montre
 * **exactement** la page exportée : un aperçu qui compose autrement est un aperçu qui ment.
 */

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
];

/** Nom affiché en tête et en signature ; « Candilog » à défaut, comme à l'export. */
export function letterSignature(first_name: string, name: string): string {
  const complet = `${first_name} ${name}`.trim();
  return complet === "" ? "Candilog" : complet;
}

/** Ligne de lieu et date : « Rennes, le 31 août 2026 », ou « Le 31 août 2026 » sans ville. */
export function letterDateLine(city: string | null, today: Date = new Date()): string {
  const date = `${today.getDate()} ${MOIS[today.getMonth()] ?? ""} ${today.getFullYear()}`;
  const ville = city?.trim() ?? "";
  return ville === "" ? `Le ${date}` : `${ville}, le ${date}`;
}

/** Intitulé de candidature affiché sur la feuille ; absent si le poste n'est pas renseigné. */
export function letterHeadline(job_title: string | null): string | null {
  const poste = job_title?.trim() ?? "";
  return poste === "" ? null : `Candidature au poste de ${poste}`;
}
