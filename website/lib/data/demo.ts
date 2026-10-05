/**
 * Jeu de données fictif des aperçus de l'application.
 *
 * Un seul persona sur tout le site — Camille Berthier, designer produit à Lyon — et des
 * entreprises inventées. Les compteurs sont cohérents d'une section à l'autre :
 * 14 candidatures, réparties 5 / 3 / 3 / 3 entre les quatre statuts ; la date du jour
 * des aperçus est le lundi 28 septembre 2026. Dates courtes au format de l'application
 * (JJ-MM), références `CAN-nnn`.
 *
 * Aucune donnée réelle : l'adresse de contact est sur `exemple.fr`, le téléphone est un
 * numéro de fiction.
 */

/** Tonalité de statut de l'application : neutre, ambre, vert, rouge (`st-n/a/g/c`). */
export type Ton = "n" | "a" | "g" | "c";

export const STATUTS: ReadonlyArray<{ ton: Ton; libelle: string; sens: string }> = [
  { ton: "n", libelle: "En attente", sens: "envoyée, pas encore de réponse" },
  { ton: "a", libelle: "Relancée", sens: "vous avez relancé, à surveiller" },
  { ton: "g", libelle: "Entretien", sens: "un échange est prévu ou a eu lieu" },
  { ton: "c", libelle: "Refusée", sens: "clôturée, gardée pour vos statistiques" },
];

export const LIBELLE_STATUT: Record<Ton, string> = {
  n: "En attente",
  a: "Relancée",
  g: "Entretien",
  c: "Refusée",
};

export type Avatar = "av1" | "av2" | "av3";
/** Teinte de pastille : accent, vert, rouge, ou neutre. */
export type Teinte = "ac" | "g" | "c" | "neutre";

export type Candidature = {
  readonly ref: string;
  readonly poste: string;
  readonly entreprise: string;
  readonly initiales: string;
  readonly avatar: Avatar;
  readonly contrat: "CDI" | "CDD";
  readonly ville: string;
  readonly envoyee: string;
  readonly statut: Ton;
  readonly echeance?: { readonly texte: string; readonly teinte: Teinte };
  readonly type: "Réponse à une offre" | "Spontanée";
  readonly anciennete: string;
  readonly prochain: { readonly libelle: string; readonly valeur: string };
  readonly documents: readonly string[];
  readonly activite: ReadonlyArray<{ readonly date: string; readonly fait: string }>;
};

export const CANDIDATURES: readonly Candidature[] = [
  {
    ref: "CAN-214", poste: "Designer produit senior", entreprise: "Atelier Nord", initiales: "AN", avatar: "av1",
    contrat: "CDI", ville: "Lyon 2e", envoyee: "22-09", statut: "n", echeance: { texte: "↻ 06-10", teinte: "ac" },
    type: "Réponse à une offre", anciennete: "22-09-26 · 6 j", prochain: { libelle: "Relance", valeur: "06-10-26" },
    documents: ["CV — Designer produit senior", "Lettre — Atelier Nord"],
    activite: [{ date: "23-09", fait: "Lettre en brouillon" }, { date: "22-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-212", poste: "Designer UX", entreprise: "Cobalt Bureau", initiales: "CB", avatar: "av2",
    contrat: "CDI", ville: "Villeurbanne", envoyee: "18-09", statut: "n", echeance: { texte: "↻ 30-09", teinte: "ac" },
    type: "Réponse à une offre", anciennete: "18-09-26 · 10 j", prochain: { libelle: "Relance", valeur: "30-09-26" },
    documents: ["CV — Designer UX"], activite: [{ date: "18-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-209", poste: "Product designer", entreprise: "Éditions Sillon", initiales: "ÉS", avatar: "av3",
    contrat: "CDD", ville: "Lyon 7e", envoyee: "15-09", statut: "n", echeance: { texte: "↻ 02-10", teinte: "ac" },
    type: "Spontanée", anciennete: "15-09-26 · 13 j", prochain: { libelle: "Relance", valeur: "02-10-26" },
    documents: ["Lettre — Éditions Sillon"], activite: [{ date: "15-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-205", poste: "Designer d'interface", entreprise: "Nord Réseaux", initiales: "NR", avatar: "av1",
    contrat: "CDI", ville: "Lyon 3e", envoyee: "09-09", statut: "n", echeance: { texte: "19 j", teinte: "c" },
    type: "Réponse à une offre", anciennete: "09-09-26 · 19 j", prochain: { libelle: "Relance", valeur: "23-09-26 · en retard" },
    documents: ["CV — Designer produit senior"], activite: [{ date: "09-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-198", poste: "Designer produit", entreprise: "Maison Rivet", initiales: "MR", avatar: "av2",
    contrat: "CDI", ville: "Annecy", envoyee: "02-09", statut: "n", echeance: { texte: "26 j", teinte: "c" },
    type: "Réponse à une offre", anciennete: "02-09-26 · 26 j", prochain: { libelle: "Relance", valeur: "non programmée" },
    documents: ["CV — Socle générique"], activite: [{ date: "02-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-201", poste: "Lead designer", entreprise: "Groupe Vallée", initiales: "GV", avatar: "av3",
    contrat: "CDI", ville: "Lyon 9e", envoyee: "04-09", statut: "a", echeance: { texte: "↻ 05-10", teinte: "ac" },
    type: "Réponse à une offre", anciennete: "04-09-26 · 24 j", prochain: { libelle: "Relance", valeur: "05-10-26" },
    documents: ["CV — Lead designer", "Lettre — Groupe Vallée"],
    activite: [{ date: "21-09", fait: "Relance faite · courriel" }, { date: "04-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-196", poste: "Designer UX/UI", entreprise: "Laurier & Pons", initiales: "LP", avatar: "av1",
    contrat: "CDI", ville: "Lyon 6e", envoyee: "28-08", statut: "a",
    type: "Spontanée", anciennete: "28-08-26 · 31 j", prochain: { libelle: "Relance", valeur: "faite le 15-09" },
    documents: ["Lettre — Laurier & Pons"],
    activite: [{ date: "15-09", fait: "Relance faite · téléphone" }, { date: "28-08", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-190", poste: "Designer design system", entreprise: "Verrières & Cie", initiales: "VC", avatar: "av2",
    contrat: "CDD", ville: "Grenoble", envoyee: "21-08", statut: "a", echeance: { texte: "38 j", teinte: "c" },
    type: "Réponse à une offre", anciennete: "21-08-26 · 38 j", prochain: { libelle: "Relance", valeur: "faite le 08-09" },
    documents: ["CV — Designer design system"],
    activite: [{ date: "08-09", fait: "Relance faite · courriel" }, { date: "21-08", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-207", poste: "Product designer", entreprise: "Studio Halage", initiales: "SH", avatar: "av3",
    contrat: "CDI", ville: "Lyon 1er", envoyee: "14-09", statut: "g", echeance: { texte: "auj. 14:30", teinte: "g" },
    type: "Réponse à une offre", anciennete: "14-09-26 · 14 j", prochain: { libelle: "Entretien", valeur: "28-09-26 · 14:30" },
    documents: ["CV — Product designer", "Lettre — Studio Halage"],
    activite: [{ date: "24-09", fait: "Entretien programmé" }, { date: "14-09", fait: "Candidature envoyée" }, { date: "13-09", fait: "CV généré · 16,2 s" }],
  },
  {
    ref: "CAN-203", poste: "Designer produit", entreprise: "Sablé Industries", initiales: "SI", avatar: "av1",
    contrat: "CDI", ville: "Vénissieux", envoyee: "07-09", statut: "g", echeance: { texte: "jeu. 10:00", teinte: "g" },
    type: "Réponse à une offre", anciennete: "07-09-26 · 21 j", prochain: { libelle: "Entretien", valeur: "01-10-26 · 10:00" },
    documents: ["CV — Designer produit senior"],
    activite: [{ date: "22-09", fait: "Entretien programmé" }, { date: "07-09", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-193", poste: "Designer de service", entreprise: "Groupe Vallée", initiales: "GV", avatar: "av3",
    contrat: "CDI", ville: "Lyon 9e", envoyee: "26-08", statut: "g",
    type: "Réponse à une offre", anciennete: "26-08-26 · 33 j", prochain: { libelle: "Entretien", valeur: "passé le 17-09" },
    documents: ["CV — Designer de service"],
    activite: [{ date: "17-09", fait: "Entretien · sur place" }, { date: "26-08", fait: "Candidature envoyée" }],
  },
  {
    ref: "CAN-188", poste: "Designer produit", entreprise: "Cobalt Bureau", initiales: "CB", avatar: "av2",
    contrat: "CDI", ville: "Villeurbanne", envoyee: "19-08", statut: "c",
    type: "Réponse à une offre", anciennete: "19-08-26", prochain: { libelle: "Réponse", valeur: "refus le 03-09" },
    documents: ["CV — Socle générique"], activite: [{ date: "03-09", fait: "Refus reçu" }],
  },
  {
    ref: "CAN-184", poste: "UI designer", entreprise: "Sablé Industries", initiales: "SI", avatar: "av1",
    contrat: "CDD", ville: "Vénissieux", envoyee: "12-08", statut: "c",
    type: "Spontanée", anciennete: "12-08-26", prochain: { libelle: "Réponse", valeur: "refus le 29-08" },
    documents: [], activite: [{ date: "29-08", fait: "Refus reçu" }],
  },
  {
    ref: "CAN-180", poste: "Designer interaction", entreprise: "Maison Rivet", initiales: "MR", avatar: "av2",
    contrat: "CDI", ville: "Annecy", envoyee: "05-08", statut: "c",
    type: "Réponse à une offre", anciennete: "05-08-26", prochain: { libelle: "Réponse", valeur: "refus le 20-08" },
    documents: [], activite: [{ date: "20-08", fait: "Refus reçu" }],
  },
];

export const candidaturesParStatut = (ton: Ton) => CANDIDATURES.filter((c) => c.statut === ton);

/** Bandeaux que le Kanban pose en tête de colonne. */
export const BANDEAUX: Partial<Record<Ton, { texte: string; teinte: "c" | "g" }>> = {
  n: { texte: "2 sans réponse depuis plus de 14 jours", teinte: "c" },
  g: { texte: "1 entretien aujourd'hui à 14:30", teinte: "g" },
};

/* ── Aujourd'hui ─────────────────────────────────────────────────────────── */

export type LigneJour = {
  readonly ton: Ton;
  readonly ref: string;
  readonly titre: string;
  readonly detail: string;
  readonly pastille?: { readonly texte: string; readonly teinte: Teinte };
  readonly action?: { readonly libelle: string; readonly touche: string; readonly primaire?: boolean };
  readonly actionSecondaire?: string;
  readonly jour?: string;
};

export const AUJOURDHUI: ReadonlyArray<{ libelle: string; ton: Ton; lignes: readonly LigneJour[] }> = [
  {
    libelle: "En retard", ton: "c",
    lignes: [
      { ton: "a", ref: "CAN-205", titre: "Relance à envoyer", detail: "Nord Réseaux · Designer d'interface",
        pastille: { texte: "5 j de retard", teinte: "c" }, actionSecondaire: "Rédiger",
        action: { libelle: "Faire", touche: "⏎", primaire: true } },
    ],
  },
  {
    libelle: "Aujourd'hui", ton: "g",
    lignes: [
      { ton: "g", ref: "CAN-207", titre: "Entretien — Product designer", detail: "Studio Halage · visioconférence",
        pastille: { texte: "14:30", teinte: "g" }, action: { libelle: "Préparer", touche: "P" } },
      { ton: "n", ref: "CAN-214", titre: "Finir la lettre ciblée", detail: "Atelier Nord · Designer produit senior",
        pastille: { texte: "brouillon", teinte: "neutre" }, action: { libelle: "Reprendre", touche: "O" } },
    ],
  },
  {
    libelle: "Cette semaine", ton: "n",
    lignes: [
      { ton: "n", ref: "CAN-212", titre: "Relance — Cobalt Bureau", detail: "Designer UX", jour: "mer. 30" },
      { ton: "g", ref: "CAN-203", titre: "Entretien — Sablé Industries", detail: "Designer produit · 10:00", jour: "jeu. 01" },
      { ton: "n", ref: "CAN-209", titre: "Relance — Éditions Sillon", detail: "Product designer", jour: "ven. 02" },
    ],
  },
];

export const SANS_REPONSE = [
  { ref: "CAN-198", entreprise: "Maison Rivet", jours: "26 j" },
  { ref: "CAN-205", entreprise: "Nord Réseaux", jours: "19 j" },
  { ref: "CAN-212", entreprise: "Cobalt Bureau", jours: "10 j" },
] as const;

/* ── Documents ───────────────────────────────────────────────────────────── */

export const DOCUMENTS = [
  {
    groupe: "CV",
    lignes: [
      { nom: "CV — Designer produit senior", detail: "Atelier Nord · généré le 23-09", pastille: { texte: "ATS 84", teinte: "g" as Teinte }, date: "26-09", initiales: "AN", avatar: "av1" as Avatar, choisi: true },
      { nom: "CV — Product designer", detail: "Studio Halage · généré le 13-09", pastille: { texte: "ATS 78", teinte: "ac" as Teinte }, date: "13-09", initiales: "SH", avatar: "av3" as Avatar },
      { nom: "CV — Socle générique", detail: "Non rattaché · base de départ", pastille: { texte: "ATS 61", teinte: "c" as Teinte }, date: "28-07", initiales: "—", avatar: "av2" as Avatar },
    ],
  },
  {
    groupe: "Lettres de motivation",
    lignes: [
      { nom: "Lettre — Atelier Nord", detail: "Designer produit senior · ton professionnel", pastille: { texte: "Brouillon", teinte: "ac" as Teinte }, date: "23-09", initiales: "AN", avatar: "av1" as Avatar },
      { nom: "Lettre — Studio Halage", detail: "Product designer · ton direct", date: "13-09", initiales: "SH", avatar: "av3" as Avatar },
      { nom: "Lettre — Groupe Vallée", detail: "Lead designer · ton professionnel", date: "04-09", initiales: "GV", avatar: "av3" as Avatar },
    ],
  },
] as const;

/* ── Analyse ─────────────────────────────────────────────────────────────── */

export const INDICATEURS = [
  { libelle: "Candidatures envoyées", valeur: "14", unite: "", detail: "depuis le 05-08" },
  { libelle: "Taux de réponse", valeur: "43", unite: "%", detail: "6 réponses sur 14" },
  { libelle: "Entretiens obtenus", valeur: "3", unite: "", detail: "2 à venir" },
  { libelle: "Délai moyen de réponse", valeur: "9", unite: "jours", detail: "sur 6 réponses" },
] as const;

export const PARCOURS_CANDIDATURES = [
  { ton: "n" as Ton, libelle: "Envoyées", nombre: 14, part: 100, note: "" },
  { ton: "a" as Ton, libelle: "Réponses reçues", nombre: 6, part: 43, note: "8 sans réponse, dont 2 au-delà de 14 jours." },
  { ton: "g" as Ton, libelle: "Entretiens", nombre: 3, part: 21, note: "3 réponses sur 6 ont mené à un entretien." },
] as const;

export const RYTHME = [
  { semaine: "03-08", nombre: 1 },
  { semaine: "10-08", nombre: 2 },
  { semaine: "17-08", nombre: 1 },
  { semaine: "24-08", nombre: 3 },
  { semaine: "31-08", nombre: 2 },
  { semaine: "07-09", nombre: 2 },
  { semaine: "14-09", nombre: 1 },
  { semaine: "21-09", nombre: 2 },
] as const;

export const CANAUX = [
  { libelle: "Réseau personnel", part: 67, fraction: "2/3" },
  { libelle: "Site de l'entreprise", part: 50, fraction: "2/4" },
  { libelle: "Candidature spontanée", part: 33, fraction: "1/3" },
  { libelle: "Offre en ligne", part: 25, fraction: "1/4" },
] as const;

/* ── Analyse face à l'offre, CV, lettre ──────────────────────────────────── */

export const EXIGENCES: ReadonlyArray<{
  ton: Ton;
  exigence: string;
  etat: "couverte" | "partielle" | "absente";
  source: string;
  preuve: string;
}> = [
  { ton: "g", exigence: "Cinq ans d'expérience en design produit", etat: "couverte", source: "CV · Expériences", preuve: "Designer produit, Parcelle — 2022 → aujourd'hui ; Designer UX, Agence Méridienne — 2019 → 2022." },
  { ton: "g", exigence: "Construire et faire vivre un design system", etat: "couverte", source: "CV · Expériences", preuve: "« Refonte du design system : 140 composants documentés, adoptés par 4 équipes. »" },
  { ton: "g", exigence: "Conduire des entretiens utilisateurs", etat: "couverte", source: "CV · Expériences", preuve: "« Entretiens mensuels avec les exploitants pour arbitrer la feuille de route. »" },
  { ton: "a", exigence: "Travailler avec les développeurs front-end", etat: "partielle", source: "CV · Compétences", preuve: "Le CV cite « Figma » et « Prototypage », sans méthode de passation ni outil partagé." },
  { ton: "a", exigence: "Mesurer l'impact des évolutions", etat: "partielle", source: "CV · Expériences", preuve: "Un seul résultat chiffré : « temps de saisie réduit de 35 % »." },
  { ton: "c", exigence: "Connaissance du RGAA", etat: "absente", source: "introuvable", preuve: "Rien dans le CV ne mentionne l'accessibilité." },
  { ton: "g", exigence: "Anglais professionnel", etat: "couverte", source: "CV · Langues", preuve: "« Anglais, courant (C1). »" },
];

export const DETAIL_SCORE = [
  { libelle: "Mots de l'offre", valeur: "11 / 15", part: 73, ton: "ac" as const },
  { libelle: "Expérience attendue", valeur: "4 / 4", part: 100, ton: "g" as const },
  { libelle: "Compétences exigées", valeur: "6 / 8", part: 75, ton: "ac" as const },
  { libelle: "Forme et lisibilité", valeur: "19 / 20", part: 95, ton: "g" as const },
] as const;

export const ETAPES_CV = [
  { libelle: "Lecture de l'offre", duree: "2,1 s" },
  { libelle: "Choix des expériences", duree: "3,4 s" },
  { libelle: "Rédaction", duree: "9,8 s" },
  { libelle: "Mise en page", duree: "0,9 s" },
] as const;

export const ETAPES_LETTRE = [
  { libelle: "Lecture de l'offre", duree: "1,8 s" },
  { libelle: "Choix des arguments", duree: "2,2 s" },
  { libelle: "Rédaction", duree: "7,9 s" },
  { libelle: "Relecture du ton", duree: "1,4 s" },
] as const;

export const SECTIONS_AUTORISEES = [
  { libelle: "Expériences", nombre: 3, actif: true },
  { libelle: "Formations", nombre: 1, actif: true },
  { libelle: "Compétences", nombre: 12, actif: true },
  { libelle: "Projets", nombre: 2, actif: true },
  { libelle: "Langues", nombre: 2, actif: true },
  { libelle: "Centres d'intérêt", nombre: 3, actif: false },
] as const;

export const PERSONA = {
  nom: "Camille Berthier",
  titre: "Designer produit senior",
  ville: "Lyon",
  courriel: "camille.berthier@exemple.fr",
  telephone: "06 12 34 56 78",
} as const;

/* ── IA ──────────────────────────────────────────────────────────────────── */

/** Les cinq tâches routables de l'application (`src/features/settings/model/taskRouting.ts`)
 *  et un routage d'exemple. L'étiquette reprend la forme de l'application : éditeur ·
 *  modèle pour un fournisseur distant, nom du modèle pour l'IA locale. */
export const ROUTAGE = [
  { tache: "Générer un CV ciblé", modele: "Ministral 3 · 3B", ou: "local" },
  { tache: "Rédiger une lettre", modele: "Ministral 3 · 3B", ou: "local" },
  { tache: "Analyser un CV", modele: "Anthropic · Claude Sonnet", ou: "distant" },
  { tache: "Extraire une offre d'emploi", modele: "Choisir un modèle", ou: "aucun" },
  { tache: "Lire un CV importé", modele: "Ministral 3 · 3B", ou: "local" },
] as const;

export const FOURNISSEURS_APERCU = [
  { nom: "IA locale", detail: "Local · 2 modèles", pret: true, choisi: true, logo: null },
  { nom: "Anthropic", detail: "Clé enregistrée", pret: true, choisi: false, logo: "claude" },
  { nom: "Mistral AI", detail: "Aucune clé", pret: false, choisi: false, logo: "mistralai" },
  { nom: "OpenAI", detail: "Aucune clé", pret: false, choisi: false, logo: "openai" },
] as const;
