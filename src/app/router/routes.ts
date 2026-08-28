/**
 * Carte des écrans de Candilog.
 *
 * Reprend à l'identique la navigation de l'application Iced (`src/navigation/mod.rs`) et
 * des maquettes SPECDESIGN : sept sections dans le rail, chacune ouvrant un écran par
 * défaut et exposant ses onglets contextuels.
 */

export type SectionKey =
  | "accueil"
  | "suivi"
  | "relations"
  | "documents"
  | "analyses"
  | "profil"
  | "reglages";

export interface RouteDef {
  /** Path React Router. */
  readonly path: string;
  /** Libellé de l'onglet contextuel. */
  readonly label: string;
  /** Name d'icône Material Symbols, tel que retenu par les maquettes. */
  readonly icon: string;
}

export interface SectionDef {
  readonly key: SectionKey;
  /** Libellé court affiché sous la tuile du rail. */
  readonly short_label: string;
  /** Libellé complet, donné en infobulle. */
  readonly long_label: string;
  readonly icon: string;
  /** Onglets contextuels ; le premier est l'écran par défaut de la section. */
  readonly routes: readonly RouteDef[];
}

export const Sections: readonly SectionDef[] = [
  {
    key: "accueil",
    short_label: "Accueil",
    long_label: "Tableau de bord",
    icon: "space_dashboard",
    routes: [{ path: "/", label: "Vue d'ensemble", icon: "donut_small" }],
  },
  {
    key: "suivi",
    short_label: "Suivi",
    long_label: "Candidatures et calendrier",
    icon: "work",
    routes: [
      { path: "/tracking/applications", label: "Candidatures", icon: "work" },
      { path: "/tracking/calendar", label: "Calendrier", icon: "calendar_month" },
    ],
  },
  {
    key: "relations",
    short_label: "Relations",
    long_label: "Entreprises et réseau",
    icon: "hub",
    routes: [
      { path: "/relations/companies", label: "Entreprises", icon: "apartment" },
      { path: "/relations/network", label: "Réseau", icon: "hub" },
    ],
  },
  {
    key: "documents",
    short_label: "Documents",
    long_label: "CV et lettres de motivation",
    icon: "description",
    routes: [
      { path: "/documents/cv", label: "Mes CV", icon: "description" },
      { path: "/documents/generate-resume", label: "Générer un CV", icon: "auto_awesome" },
      { path: "/documents/cover-letters", label: "Mes lettres", icon: "mail" },
      { path: "/documents/write-cover-letter", label: "Lettre de motivation", icon: "edit_note" },
      { path: "/documents/analyze", label: "Analyser", icon: "query_stats" },
    ],
  },
  {
    key: "analyses",
    short_label: "Analyses",
    long_label: "Statistiques",
    icon: "monitoring",
    routes: [{ path: "/analytics", label: "Statistiques", icon: "monitoring" }],
  },
  {
    key: "profil",
    short_label: "Profil",
    long_label: "Profil professionnel",
    icon: "account_circle",
    routes: [{ path: "/profile", label: "Profil", icon: "account_circle" }],
  },
  {
    key: "reglages",
    short_label: "Réglages",
    long_label: "Intelligence artificielle et maintenance",
    icon: "tune",
    routes: [
      { path: "/settings/ai", label: "Intelligence artificielle", icon: "smart_toy" },
      { path: "/settings/backups", label: "Sauvegardes", icon: "save" },
      { path: "/settings/updates", label: "Mises à jour", icon: "system_update" },
      { path: "/settings/about", label: "À propos", icon: "info" },
    ],
  },
] as const;

/** Section à laquelle appartient un chemin, pour l'état sélectionné du rail. */
export function sectionForPath(pathname: string): SectionDef {
  const match = Sections.find((section) =>
    section.routes.some((route) =>
      route.path === "/" ? pathname === "/" : pathname.startsWith(route.path),
    ),
  );
  return match ?? Sections[0]!;
}
