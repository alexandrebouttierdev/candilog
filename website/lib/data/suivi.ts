/** Palette de statuts du §7.6, partagée par le board, la liste et la fenêtre du hero.
 *
 *  « Envoyée » est en `--page` et non en `--surface-alt` : la table du §7.6 et
 *  `reference/EXAMPLE_StatusBadge.tsx` disent `--surface-alt`, mais le prototype dit
 *  `--page` et c'est lui qui fait foi ici (arbitré avec l'auteur). Le statut neutre
 *  se creuse donc légèrement dans la carte en thème sombre, là où les quatre autres
 *  ressortent en teinte — c'est voulu. */
export const STATUT = {
  envoyee: "bg-page text-ink-muted border-line",
  relance: "bg-warning-tint text-warning-text border-warning-border",
  entretien: "bg-tint-10 text-accent-text border-tint-border",
  offre: "bg-success-tint text-success-text border-success-border",
  refus: "bg-danger-tint text-danger-text border-danger-border",
} as const;

export type CleStatut = keyof typeof STATUT;

export const LIBELLE_STATUT: Record<CleStatut, string> = {
  envoyee: "Envoyée",
  relance: "Relance",
  entretien: "Entretien",
  offre: "Offre reçue",
  refus: "Refus",
};

export type CarteBoard = {
  readonly poste: string;
  readonly entreprise: string;
  readonly icone?: string;
  readonly detail?: string;
  /** Carte « fantôme » en fin de colonne, à 60 % d'opacité dans le design. */
  readonly attenuee?: boolean;
  /** La carte pilotée par la boucle d'animation (§7.6). */
  readonly animee?: boolean;
};

export type ColonneBoard = {
  readonly statut: CleStatut;
  readonly puce: string;
  readonly total: number;
  /** Les colonnes du design n'ont pas toutes le fond creusé. */
  readonly fondCreuse: boolean;
  readonly cartes: readonly CarteBoard[];
};

export const COLONNES_BOARD: readonly ColonneBoard[] = [
  {
    statut: "envoyee",
    puce: "bg-ink-faint",
    total: 4,
    fondCreuse: true,
    cartes: [
      { poste: "Assistant de direction", entreprise: "Maison Rivet", icone: "schedule", detail: "26 juil." },
      { poste: "Chargée de communication", entreprise: "Éditions Sillon", icone: "schedule", detail: "25 juil." },
      { poste: "Technicien support", entreprise: "Nord Réseaux", attenuee: true },
    ],
  },
  {
    statut: "relance",
    puce: "bg-warning",
    total: 2,
    fondCreuse: true,
    cartes: [
      { poste: "Chargé de projet", entreprise: "Groupe Vallée", icone: "notifications_active", detail: "Relancer le 08 août" },
      { poste: "Gestionnaire ADV", entreprise: "Sablé Industries", attenuee: true },
    ],
  },
  {
    statut: "entretien",
    puce: "bg-accent",
    total: 3,
    fondCreuse: false,
    cartes: [
      { poste: "Designer produit", entreprise: "Atelier Nord", detail: "05 août", animee: true },
      { poste: "Designer d'interface", entreprise: "Studio Halage", icone: "event", detail: "12 août · 10:00" },
    ],
  },
  {
    statut: "offre",
    puce: "bg-success",
    total: 1,
    fondCreuse: true,
    cartes: [
      { poste: "Coordinateur logistique", entreprise: "Cobalt Bureau", icone: "check_circle", detail: "Réponse à donner" },
    ],
  },
  {
    statut: "refus",
    puce: "bg-danger",
    total: 2,
    fondCreuse: true,
    cartes: [
      { poste: "Gestionnaire de paie", entreprise: "Laurier & Pons", detail: "Sans suite après entretien" },
      { poste: "Assistant RH", entreprise: "Verrières & Cie", attenuee: true },
    ],
  },
];

export type LigneCandidature = {
  readonly initiales: string;
  readonly poste: string;
  readonly entreprise: string;
  readonly statut: CleStatut;
  readonly etape: string;
  readonly date: string;
};

export const CANDIDATURES: readonly LigneCandidature[] = [
  { initiales: "AN", poste: "Designer produit", entreprise: "Atelier Nord", statut: "entretien", etape: "05 août · 14:30", date: "02 août" },
  { initiales: "SH", poste: "Designer d'interface", entreprise: "Studio Halage", statut: "entretien", etape: "12 août · 10:00", date: "30 juil." },
  { initiales: "GV", poste: "Chargé de projet", entreprise: "Groupe Vallée", statut: "relance", etape: "Relancer le 08 août", date: "28 juil." },
  { initiales: "MR", poste: "Assistant de direction", entreprise: "Maison Rivet", statut: "envoyee", etape: "—", date: "26 juil." },
  { initiales: "ES", poste: "Chargée de communication", entreprise: "Éditions Sillon", statut: "envoyee", etape: "—", date: "25 juil." },
  { initiales: "CB", poste: "Coordinateur logistique", entreprise: "Cobalt Bureau", statut: "offre", etape: "Réponse à donner", date: "21 juil." },
  { initiales: "SI", poste: "Gestionnaire ADV", entreprise: "Sablé Industries", statut: "relance", etape: "Relancer le 11 août", date: "19 juil." },
  { initiales: "LP", poste: "Gestionnaire de paie", entreprise: "Laurier & Pons", statut: "refus", etape: "—", date: "14 juil." },
];

/** Boucle d'animation de la carte « Designer produit » : Envoyée → Relance →
 *  Entretien, toutes les 2 600 ms. La note de l'en-tête suit le statut. */
export const BOUCLE_STATUT = [
  { statut: "envoyee" as const, note: "candidature envoyée le 02 août" },
  { statut: "relance" as const, note: "relance envoyée · réponse attendue" },
  { statut: "entretien" as const, note: "entretien programmé le 05 août" },
];

export const PERIODE_BOUCLE_MS = 2600;

export const ATOUTS_SUIVI = [
  { icone: "swap_horiz", accentue: true, titre: "Le statut suit la réalité", texte: "Déplacez une candidature d'une colonne à l'autre : dates, relances et entretiens se mettent à jour avec elle." },
  { icone: "view_kanban", accentue: false, titre: "Board ou liste, au choix", texte: "Le board pour voir l'avancement, la liste pour trier, filtrer et retrouver une candidature précise." },
  { icone: "domain", accentue: false, titre: "Entreprises et contacts liés", texte: "Une entreprise regroupe ses postes, ses échanges et les personnes rencontrées." },
  { icone: "cloud_off", accentue: false, titre: "Tout reste hors ligne", texte: "Candilog travaille sur votre machine, et vos données s'exportent quand vous le voulez." },
];
