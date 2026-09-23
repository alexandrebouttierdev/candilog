/**
 * Chemins des destinations de Candilog v2.
 *
 * Centralisés ici, et non dans `app/router`, parce que les features ouvrent elles aussi des
 * écrans (le tableau de bord ouvre une candidature, la bibliothèque ouvre le générateur) et
 * qu'une feature ne dépend pas de la coque. Les anciens chemins restent redirigés par le
 * routeur le temps que plus rien ne les produise.
 */
export const PATHS = {
  today: "/",
  applications: "/applications",
  applicationsKanban: "/applications/kanban",
  calendar: "/applications/calendar",
  analytics: "/applications/analytics",
  companies: "/relations/companies",
  contacts: "/relations/contacts",
  documents: "/documents",
  letters: "/documents/letters",
  generateResume: "/documents/generate-resume",
  writeLetter: "/documents/write-cover-letter",
  analyzeResume: "/documents/analyze",
  ai: "/ai",
  profile: "/profile",
} as const;

/** Écran Candidatures sur une fiche (`id`) ou sur le formulaire de création (`new`). */
export function applicationsPath(options: { id?: string; create?: boolean } = {}): string {
  const params = new URLSearchParams();
  if (options.id) params.set("id", options.id);
  if (options.create) params.set("new", "1");
  const query = params.toString();
  return query ? `${PATHS.applications}?${query}` : PATHS.applications;
}
