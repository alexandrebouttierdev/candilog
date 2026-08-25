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
  /** Chemin React Router. */
  readonly path: string;
  /** Libellé de l'onglet contextuel. */
  readonly label: string;
  /** Nom d'icône Material Symbols, tel que retenu par les maquettes. */
  readonly icon: string;
}

export interface SectionDef {
  readonly key: SectionKey;
  /** Libellé court affiché sous la tuile du rail. */
  readonly shortLabel: string;
  /** Libellé complet, donné en infobulle. */
  readonly longLabel: string;
  readonly icon: string;
  /** Onglets contextuels ; le premier est l'écran par défaut de la section. */
  readonly routes: readonly RouteDef[];
}

export const SECTIONS: readonly SectionDef[] = [
  {
    key: "accueil",
    shortLabel: "Accueil",
    longLabel: "Tableau de bord",
    icon: "space_dashboard",
    routes: [{ path: "/", label: "Vue d'ensemble", icon: "donut_small" }],
  },
  {
    key: "suivi",
    shortLabel: "Suivi",
    longLabel: "Candidatures et calendrier",
    icon: "work",
    routes: [
      { path: "/suivi/candidatures", label: "Candidatures", icon: "work" },
      { path: "/suivi/calendrier", label: "Calendrier", icon: "calendar_month" },
    ],
  },
  {
    key: "relations",
    shortLabel: "Relations",
    longLabel: "Entreprises et réseau",
    icon: "hub",
    routes: [
      { path: "/relations/entreprises", label: "Entreprises", icon: "apartment" },
      { path: "/relations/reseau", label: "Réseau", icon: "hub" },
    ],
  },
  {
    key: "documents",
    shortLabel: "Documents",
    longLabel: "CV et lettres de motivation",
    icon: "description",
    routes: [
      { path: "/documents/cv", label: "Mes CV", icon: "description" },
      { path: "/documents/generer-cv", label: "Générer un CV", icon: "auto_awesome" },
      { path: "/documents/lettres", label: "Mes lettres", icon: "mail" },
      { path: "/documents/rediger-lettre", label: "Lettre de motivation", icon: "edit_note" },
      { path: "/documents/analyser", label: "Analyser", icon: "query_stats" },
    ],
  },
  {
    key: "analyses",
    shortLabel: "Analyses",
    longLabel: "Statistiques",
    icon: "monitoring",
    routes: [{ path: "/analyses", label: "Statistiques", icon: "monitoring" }],
  },
  {
    key: "profil",
    shortLabel: "Profil",
    longLabel: "Profil professionnel",
    icon: "account_circle",
    routes: [{ path: "/profil", label: "Profil", icon: "account_circle" }],
  },
  {
    key: "reglages",
    shortLabel: "Réglages",
    longLabel: "Intelligence artificielle et maintenance",
    icon: "tune",
    routes: [
      { path: "/reglages/ia", label: "Intelligence artificielle", icon: "smart_toy" },
      { path: "/reglages/sauvegardes", label: "Sauvegardes", icon: "save" },
      { path: "/reglages/mises-a-jour", label: "Mises à jour", icon: "system_update" },
      { path: "/reglages/a-propos", label: "À propos", icon: "info" },
    ],
  },
] as const;

/** Section à laquelle appartient un chemin, pour l'état sélectionné du rail. */
export function sectionForPath(pathname: string): SectionDef {
  const match = SECTIONS.find((section) =>
    section.routes.some((route) =>
      route.path === "/" ? pathname === "/" : pathname.startsWith(route.path),
    ),
  );
  return match ?? SECTIONS[0]!;
}
