/**
 * Carte des destinations de Candilog v2 (`reference_design/INTERACTIONS.md` §1-2).
 *
 * Six destinations dans la barre de navigation. Les Réglages ne sont pas une destination
 * mais une surcouche (`⌘,`) ; les générateurs, l'analyse et l'import sont des actions.
 * Chaque destination peut exposer des onglets de vue, rendus dans la barre de titre.
 */

import type { LineIconName } from "@/shared/ui";
import { PATHS } from "@/shared/lib/paths";

export type DestinationKey = "today" | "applications" | "relations" | "documents" | "ai" | "profile";

export interface ViewTab {
  readonly path: string;
  readonly label: string;
}

export interface Destination {
  readonly key: DestinationKey;
  readonly label: string;
  readonly icon: LineIconName;
  /** Chemin ouvert par la navigation (vue par défaut). */
  readonly path: string;
  /** Lettre du raccourci `G` puis lettre, quand la destination en a un. */
  readonly goKey?: string;
  /** Préfixes de chemin appartenant à la destination. */
  readonly prefixes: readonly string[];
  /** Onglets de vue de la barre de titre ; le premier est la vue par défaut. */
  readonly tabs?: readonly ViewTab[];
}

export const DESTINATIONS: readonly Destination[] = [
  { key: "today", label: "Aujourd'hui", icon: "today", path: PATHS.today, goKey: "a", prefixes: [] },
  {
    key: "applications",
    label: "Candidatures",
    icon: "applications",
    path: PATHS.applications,
    goKey: "c",
    prefixes: ["/applications"],
    tabs: [
      { path: PATHS.applications, label: "Liste" },
      { path: PATHS.applicationsKanban, label: "Kanban" },
      { path: PATHS.calendar, label: "Calendrier" },
      { path: PATHS.analytics, label: "Analyse" },
    ],
  },
  {
    key: "relations",
    label: "Relations",
    icon: "relations",
    path: PATHS.companies,
    goKey: "r",
    // La bascule Entreprises / Contacts vit dans la barre d'outils de l'écran
    // (`screens/05-companies.png`), pas dans la barre de titre.
    prefixes: ["/relations"],
  },
  {
    key: "documents",
    label: "Documents",
    icon: "documents",
    path: PATHS.documents,
    goKey: "d",
    prefixes: ["/documents"],
    // Les onglets Tous / CV / Lettres / Analyses vivent dans la barre d'outils de l'écran.
  },
  { key: "ai", label: "Intelligence artificielle", icon: "ai", path: PATHS.ai, prefixes: ["/ai"] },
  { key: "profile", label: "Profil", icon: "profile", path: PATHS.profile, goKey: "p", prefixes: ["/profile"] },
] as const;

/** Destination d'un chemin, pour l'état actif de la navigation ; Aujourd'hui par défaut. */
export function destinationForPath(pathname: string): Destination {
  return (
    DESTINATIONS.find((destination) =>
      destination.prefixes.some((prefix) => pathname === prefix || pathname.startsWith(`${prefix}/`)),
    ) ?? DESTINATIONS[0]!
  );
}

/** Onglet de vue actif : correspondance exacte, sinon le premier onglet. */
export function activeTab(destination: Destination, pathname: string): ViewTab | undefined {
  return destination.tabs?.find((tab) => tab.path === pathname) ?? destination.tabs?.[0];
}

/**
 * Anciens chemins (v1) → nouveaux. Conservés pour les liens encore émis par un écran non
 * migré ; les Réglages v1 ouvrent la surcouche sur leur section.
 */
export const LEGACY_REDIRECTS: Readonly<Record<string, string>> = {
  "/tracking/applications": PATHS.applications,
  "/tracking/calendar": PATHS.calendar,
  "/analytics": PATHS.analytics,
  "/relations/network": PATHS.contacts,
  "/documents/cv": PATHS.documents,
  "/documents/cover-letters": PATHS.letters,
  "/settings/ai": PATHS.ai,
};
