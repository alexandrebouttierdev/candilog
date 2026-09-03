/**
 * Libellés de date des écrans Analyses et Aujourd'hui.
 *
 * Distincts de `@/shared/lib/dates`, qui sert la saisie et la lecture d'une date isolée :
 * ici les formats sont ceux d'un axe de graphique ou d'une ligne de liste, où la place
 * disponible commande la longueur.
 */
export function formatDate(value: string, format: "court" | "long" | "numeric"): string {
  const date = new Date(`${value.slice(0, 10)}T12:00:00`);
  if (Number.isNaN(date.getTime())) return value.slice(0, 10);
  if (format === "numeric") {
    return new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "2-digit" }).format(date);
  }
  return new Intl.DateTimeFormat("fr-FR", {
    day: format === "long" ? "2-digit" : "numeric",
    month: "short",
  }).format(date);
}
