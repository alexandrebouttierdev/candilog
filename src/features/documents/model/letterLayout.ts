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
  return poste === "" ? null : `Candidature au poste ${elider("de", poste)}`;
}

/**
 * Colle une préposition à un mot en respectant l'élision : « de Astek » → « d'Astek ».
 *
 * Jumeau de `core::utils::text::elider`. Sans elle, la feuille annonçait « Candidature au
 * poste de Administrateur », faute que personne ne commettrait dans une vraie candidature.
 * Le `h` est traité comme muet : distinguer le `h` aspiré demanderait un lexique.
 */
export function elider(preposition: string, suivant: string): string {
  const valeur = suivant.trim();
  const premiere = valeur.normalize("NFD").replace(/[\u0300-\u036f]/g, "")[0]?.toLowerCase();
  if (premiere !== undefined && "aeiouyh".includes(premiere)) {
    return `${preposition.replace(/[ea]$/, "")}\u2019${valeur}`;
  }
  return `${preposition} ${valeur}`;
}

/**
 * Retire l'amorce de l'intitulé saisi sur la feuille, élidée ou non.
 *
 * La zone éditable rend `letterHeadline` : ce qui en ressort porte l'amorce, et l'enregistrer
 * telle quelle ferait « Candidature au poste de Candidature au poste d\u2019Administrateur ».
 */
export function letterJobTitleFromHeadline(value: string): string {
  const amorce = /^Candidature au poste (?:de |d[\u2019'])/;
  return value.replace(amorce, "");
}
