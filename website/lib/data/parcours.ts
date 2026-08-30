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

export const ETAPES: readonly Etape[] = [
  {
    icone: "travel_explore",
    titre: "Je trouve une offre",
    sousTitre: "Je l'enregistre entière",
    detailTitre: "L'offre entre dans Candilog, pas dans un onglet",
    detailTexte:
      "Collez le lien ou le texte d'une annonce. Candilog en extrait le poste, l'entreprise, le lieu et les éléments attendus, puis garde l'offre complète, même si elle disparaît du site.",
    panneau: { icone: "content_paste", titre: "Importer une offre", fil: "Offres · Nouvelle" },
  },
  {
    icone: "query_stats",
    titre: "Je l'analyse",
    sousTitre: "Attentes, mots-clés, écarts",
    detailTitre: "Ce que l'annonce demande vraiment",
    detailTexte:
      "L'analyse met en avant les compétences citées, celles que votre profil couvre déjà et celles à formuler autrement. À vous de décider quoi retenir.",
    panneau: { icone: "query_stats", titre: "Analyse de l'offre", fil: "Offres · Atelier Nord" },
  },
  {
    icone: "edit_document",
    titre: "J'adapte mon CV",
    sousTitre: "Une version par offre",
    detailTitre: "Un CV par offre, sans repartir de zéro",
    detailTexte:
      "Partez de votre profil enregistré, adaptez les sections utiles pour cette annonce et gardez la version attachée à la candidature.",
    panneau: { icone: "description", titre: "Documents", fil: "Documents · CV" },
  },
  {
    icone: "send",
    titre: "Je candidate",
    sousTitre: "Poste, contact, documents",
    detailTitre: "La candidature se remplit presque seule",
    detailTexte:
      "Le poste, l'entreprise et les documents viennent de l'offre déjà enregistrée. Vous ajoutez la date d'envoi et le contact, c'est tout.",
    panneau: { icone: "send", titre: "Candidatures", fil: "Candidatures · Nouvelle" },
  },
  {
    icone: "event_repeat",
    titre: "Je garde la main",
    sousTitre: "Relances et entretiens",
    detailTitre: "Rien ne tombe entre deux semaines",
    detailTexte:
      "Chaque candidature porte sa date de relance et ses entretiens. Le calendrier reprend l'ensemble, sans que vous ayez à y penser.",
    panneau: { icone: "calendar_month", titre: "Calendrier", fil: "Calendrier · Août 2026" },
  },
];

/** Jours marqués du calendrier d'août 2026 (écran 05). Le mois commence un samedi,
 *  d'où les 5 cellules vides en tête de grille. */
export const CASES_VIDES_AOUT = 5;

export const MARQUES_CALENDRIER: Readonly<
  Record<number, { readonly libelle: string; readonly classes: string }>
> = {
  5: { libelle: "Entretien 14:30", classes: "bg-tint-12 text-accent-text" },
  8: { libelle: "Relance", classes: "bg-warning-tint text-warning-text" },
  12: { libelle: "Entretien 10:00", classes: "bg-tint-12 text-accent-text" },
  20: { libelle: "Réponse", classes: "bg-success-tint text-success-text" },
};
