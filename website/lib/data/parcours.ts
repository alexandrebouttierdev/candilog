export type Etape = {
  readonly icone: string;
  readonly titre: string;
  readonly sousTitre: string;
  /** Titre et texte du panneau de gauche, sous la frise. */
  readonly detailTitre: string;
  readonly detailTexte: string;
  /** En-tête du panneau d'écran, à droite. */
  readonly panneau: { readonly icone: string; readonly titre: string; readonly fil: string };
};

/** Le parcours réel de Candilog, écran par écran.
 *
 *  Chaque étape correspond à une route qui existe : `Sections` (src/app/router/routes.tsx)
 *  ne connaît ni section « Offres », ni import d'annonce par URL, ni extraction automatique
 *  du poste et de l'entreprise. Une offre entre dans Candilog par son **texte collé**, dans
 *  « Générer un CV » ou « Analyser ». */
export const ETAPES: readonly Etape[] = [
  {
    icone: "content_paste",
    titre: "Je colle l'offre",
    sousTitre: "Son texte, rien d'autre",
    detailTitre: "L'annonce entre par son texte",
    detailTexte:
      "Collez l'intitulé, les missions et les compétences attendues dans « Générer un CV ». Le texte part au seul fournisseur que vous avez configuré — et à personne d'autre.",
    panneau: {
      icone: "auto_awesome",
      titre: "Générer un CV",
      fil: "Documents · Générer un CV",
    },
  },
  {
    icone: "query_stats",
    titre: "J'arbitre les écarts",
    sousTitre: "Une proposition à la fois",
    detailTitre: "Rien ne s'applique sans vous",
    detailTexte:
      "Candilog compare le CV à l'annonce, chiffre l'écart et propose des reformulations précises. Chaque proposition s'accepte ou s'ignore séparément, et le score suit vos décisions.",
    panneau: {
      icone: "auto_awesome",
      titre: "Générer un CV",
      fil: "Documents · Générer un CV",
    },
  },
  {
    icone: "description",
    titre: "Je garde la version",
    sousTitre: "Une par offre",
    detailTitre: "Chaque CV reste rattaché à son offre",
    detailTexte:
      "La bibliothèque garde chaque version avec son score et sa date. Vous savez toujours quel CV vous avez envoyé, et pour quelle annonce.",
    panneau: { icone: "description", titre: "Mes CV", fil: "Documents · Mes CV" },
  },
  {
    icone: "work",
    titre: "Je candidate",
    sousTitre: "Poste, entreprise, suivi",
    detailTitre: "Ce qui se répète est hérité",
    detailTexte:
      "La ville, l'adresse et le type d'entreprise viennent de la fiche entreprise ; vous ne les ressaisissez que si cette candidature en diffère.",
    panneau: {
      icone: "work",
      titre: "Nouvelle candidature",
      fil: "Candidatures · Nouvelle",
    },
  },
  {
    icone: "calendar_month",
    titre: "Je garde la main",
    sousTitre: "Relances et entretiens",
    detailTitre: "Rien ne tombe entre deux semaines",
    detailTexte:
      "Chaque candidature porte ses relances et ses entretiens. Le calendrier les reprend tous, sans que vous ayez à y penser.",
    panneau: { icone: "calendar_month", titre: "Calendrier", fil: "Calendrier · Septembre 2026" },
  },
];

/** Jours abrégés de la grille mensuelle, ceux de `DAYS` (src/features/calendar/model/month.ts). */
export const JOURS_CALENDRIER = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"] as const;

/** La grille du calendrier fait toujours six semaines de sept jours : une hauteur variable
 *  ferait sauter la mise en page d'un mois à l'autre. Septembre 2026 commence un mardi,
 *  d'où la case d'août en tête et le débord sur octobre en fin de grille. */
export const CASES_AVANT_SEPTEMBRE = 1;
export const JOURS_SEPTEMBRE = 30;
export const CASES_GRILLE = 42;

export type EvenementCalendrier = {
  readonly icone: string;
  readonly heure?: string;
  readonly libelle: string;
  /** Pastille d'événement : `PASTILLE` dans `GridMonth`, par tonalité. */
  readonly classes: string;
};

export const EVENEMENTS_CALENDRIER: Readonly<Record<number, readonly EvenementCalendrier[]>> = {
  4: [{ icone: "send", libelle: "Éditions Sillon", classes: "bg-warning-tint text-warning" }],
  8: [
    {
      icone: "event_available",
      heure: "14:30",
      libelle: "Atelier Nord",
      classes: "bg-success-tint text-success",
    },
  ],
  10: [{ icone: "send", libelle: "Groupe Vallée", classes: "bg-warning-tint text-warning" }],
  14: [{ icone: "send", libelle: "Sablé Industries", classes: "bg-warning-tint text-warning" }],
  15: [
    {
      icone: "event_available",
      heure: "10:00",
      libelle: "Studio Halage",
      classes: "bg-success-tint text-success",
    },
  ],
  21: [
    {
      icone: "event_available",
      heure: "11:00",
      libelle: "Cobalt Bureau",
      classes: "bg-success-tint text-success",
    },
  ],
};

/** Le jour marqué « aujourd'hui » dans la grille : pastille pleine en accent. */
export const JOUR_COURANT = 2;
