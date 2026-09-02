/** Statuts de candidature — miroir de `src/features/applications/model/statuses.ts`.
 *
 *  L'application n'en connaît que quatre, dans cet ordre, et c'est celui des colonnes du
 *  Kanban. Il n'existe pas de statut « offre reçue » : une réponse positive se lit dans
 *  l'entretien et les notes, pas dans une cinquième colonne.
 *
 *  Les tonalités viennent de l'application : neutre pour l'attente, ambre pour ce qui est à
 *  traiter, vert pour l'avancement, rouge pour l'échec. La couleur ne porte jamais
 *  l'information seule — chaque pastille affiche son libellé et son icône. */
export const STATUT = {
  attente: "bg-page text-ink-muted border-line",
  relancee: "bg-warning-tint text-warning-text border-warning-border",
  entretien: "bg-success-tint text-success-text border-success-border",
  refus: "bg-danger-tint text-danger-text border-danger-border",
} as const;

export type CleStatut = keyof typeof STATUT;

export const LIBELLE_STATUT: Record<CleStatut, string> = {
  attente: "En attente",
  relancee: "Relancée",
  entretien: "Entretien",
  refus: "Refusée",
};

/** Icône de la pastille, identique à celle de `StatusPill` dans l'application. */
export const ICONE_STATUT: Record<CleStatut, string> = {
  attente: "hourglass_top",
  relancee: "send",
  entretien: "event_available",
  refus: "do_not_disturb_on",
};

/** Pastille de couleur de l'en-tête de colonne du Kanban. */
export const PUCE_STATUT: Record<CleStatut, string> = {
  attente: "bg-ink-faint",
  relancee: "bg-warning",
  entretien: "bg-success",
  refus: "bg-danger",
};

/** Ordre des colonnes du Kanban, celui de `Statuses` côté application. */
export const ORDRE_STATUTS: readonly CleStatut[] = ["attente", "relancee", "entretien", "refus"];

export type CarteBoard = {
  readonly initiales: string;
  readonly poste: string;
  readonly entreprise: string;
  /** Type de contrat, rendu en `Tag` sur la carte réelle. */
  readonly contrat: string;
  readonly ville: string;
  /** Ancienneté en jours depuis l'envoi ; au-delà de 15 la carte passe en ambre. */
  readonly jours: number;
};

export type ColonneBoard = {
  readonly statut: CleStatut;
  readonly cartes: readonly CarteBoard[];
};

/** Les quatre colonnes, hors carte animée. Le compteur d'en-tête est calculé à
 *  l'affichage : la carte que la boucle déplace compte dans la colonne où elle se
 *  trouve, comme un compteur qui suit un glisser-déposer réel. */
export const COLONNES_BOARD: readonly ColonneBoard[] = [
  {
    statut: "attente",
    cartes: [
      { initiales: "NR", poste: "UX Researcher", entreprise: "Nord Réseaux", contrat: "CDI", ville: "Villeurbanne", jours: 9 },
      { initiales: "VC", poste: "Designer produit senior", entreprise: "Verrières & Cie", contrat: "CDI", ville: "Lyon", jours: 7 },
      { initiales: "MR", poste: "Designer d'interaction", entreprise: "Maison Rivet", contrat: "CDI", ville: "Lyon", jours: 6 },
      { initiales: "LP", poste: "Designer de système de composants", entreprise: "Laurier & Pons", contrat: "CDI", ville: "Lyon", jours: 5 },
      { initiales: "NR", poste: "Product Designer", entreprise: "Nord Réseaux", contrat: "Intérim", ville: "Villeurbanne", jours: 2 },
    ],
  },
  {
    statut: "relancee",
    cartes: [
      { initiales: "GV", poste: "Chargé de projet digital", entreprise: "Groupe Vallée", contrat: "CDI", ville: "Grenoble", jours: 36 },
      { initiales: "SI", poste: "Designer UX", entreprise: "Sablé Industries", contrat: "CDD", ville: "Saint-Étienne", jours: 14 },
      { initiales: "ES", poste: "Designer graphique", entreprise: "Éditions Sillon", contrat: "CDD", ville: "Lyon", jours: 39 },
    ],
  },
  {
    statut: "entretien",
    cartes: [
      { initiales: "SH", poste: "Designer d'interface", entreprise: "Studio Halage", contrat: "CDI", ville: "Nantes", jours: 34 },
      { initiales: "CB", poste: "Product Designer", entreprise: "Cobalt Bureau", contrat: "CDI", ville: "Paris", jours: 12 },
    ],
  },
  {
    statut: "refus",
    cartes: [
      { initiales: "LP", poste: "Lead designer", entreprise: "Laurier & Pons", contrat: "CDI", ville: "Lyon", jours: 50 },
      { initiales: "MR", poste: "Designer UI", entreprise: "Maison Rivet", contrat: "CDD", ville: "Lyon", jours: 38 },
      { initiales: "VC", poste: "Designer produit", entreprise: "Verrières & Cie", contrat: "CDI", ville: "Lyon", jours: 52 },
    ],
  },
];

export type LigneCandidature = {
  readonly initiales: string;
  readonly poste: string;
  /** Domaine professionnel, sous-titre de la cellule d'identité. */
  readonly domaine: string;
  readonly entreprise: string;
  readonly ville: string;
  readonly contrat: string;
  readonly duree: string;
  readonly type: string;
  readonly statut: CleStatut;
  readonly date: string;
};

/** Les huit colonnes réelles de la vue Liste, dans l'ordre de `ApplicationsPage`. */
export const CANDIDATURES: readonly LigneCandidature[] = [
  { initiales: "AN", poste: "Designer produit", domaine: "Communication / Multimédia", entreprise: "Atelier Nord", ville: "Lyon", contrat: "CDI", duree: "Temps plein · 35 h", type: "Offre", statut: "entretien", date: "02-08-2026" },
  { initiales: "SH", poste: "Designer d'interface", domaine: "Communication / Multimédia", entreprise: "Studio Halage", ville: "Nantes", contrat: "CDI", duree: "Temps plein · 35 h", type: "Offre", statut: "entretien", date: "30-07-2026" },
  { initiales: "CB", poste: "Product Designer", domaine: "Communication / Multimédia", entreprise: "Cobalt Bureau", ville: "Paris", contrat: "CDI", duree: "Temps plein · 35 h", type: "Offre", statut: "entretien", date: "21-08-2026" },
  { initiales: "GV", poste: "Chargé de projet digital", domaine: "Communication / Multimédia", entreprise: "Groupe Vallée", ville: "Grenoble", contrat: "CDI", duree: "Temps plein · 39 h", type: "Offre", statut: "relancee", date: "28-07-2026" },
  { initiales: "SI", poste: "Designer UX", domaine: "Communication / Multimédia", entreprise: "Sablé Industries", ville: "Saint-Étienne", contrat: "CDD", duree: "Temps plein · 35 h", type: "Offre", statut: "relancee", date: "19-08-2026" },
  { initiales: "NR", poste: "UX Researcher", domaine: "Communication / Multimédia", entreprise: "Nord Réseaux", ville: "Villeurbanne", contrat: "CDI", duree: "Temps plein · 35 h", type: "Offre", statut: "attente", date: "24-08-2026" },
  { initiales: "MR", poste: "Designer d'interaction", domaine: "Communication / Multimédia", entreprise: "Maison Rivet", ville: "Lyon", contrat: "CDI", duree: "Temps plein · 35 h", type: "Spontanée", statut: "attente", date: "27-08-2026" },
  { initiales: "LP", poste: "Lead designer", domaine: "Communication / Multimédia", entreprise: "Laurier & Pons", ville: "Lyon", contrat: "CDI", duree: "Temps plein · 35 h", type: "Offre", statut: "refus", date: "14-07-2026" },
];

/** La carte que la boucle déplace de colonne en colonne (§7.6).
 *
 *  Dans l'application, une carte ne porte pas son statut : c'est la colonne qui le porte,
 *  et on change de statut en glissant la carte. L'animation reproduit ce geste — elle
 *  déplace la carte — plutôt que de repeindre une pastille qui n'existe pas. */
export const CARTE_ANIMEE: CarteBoard = {
  initiales: "AN",
  poste: "Designer produit",
  entreprise: "Atelier Nord",
  contrat: "CDI",
  ville: "Lyon",
  jours: 31,
};

/** Boucle d'animation : En attente → Relancée → Entretien, toutes les 2 600 ms.
 *  La note de l'en-tête suit la colonne où se trouve la carte. */
export const BOUCLE_STATUT = [
  { statut: "attente" as const, note: "candidature envoyée le 02-08-2026" },
  { statut: "relancee" as const, note: "relance envoyée · réponse attendue" },
  { statut: "entretien" as const, note: "entretien programmé le 08-09-2026" },
];

export const PERIODE_BOUCLE_MS = 2600;

export const ATOUTS_SUIVI = [
  { icone: "swap_horiz", accentue: true, titre: "Le statut suit la réalité", texte: "Glissez une candidature d'une colonne à l'autre : le statut change, et l'historique garde la trace de chaque étape." },
  { icone: "view_kanban", accentue: false, titre: "Kanban ou liste, au choix", texte: "Le Kanban pour voir l'avancement, la liste pour trier, filtrer, cocher et exporter en CSV." },
  { icone: "hub", accentue: false, titre: "Entreprises et réseau liés", texte: "Une entreprise regroupe ses postes, ses contacts et les entretiens déjà passés." },
  { icone: "cloud_off", accentue: false, titre: "Tout reste hors ligne", texte: "Candilog travaille sur votre machine, et vos données s'exportent quand vous le voulez." },
];
