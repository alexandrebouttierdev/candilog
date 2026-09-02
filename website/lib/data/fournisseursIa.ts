export type FournisseurIa = {
  readonly nom: string;
  /** Nom du fichier dans public/providers, sans extension. */
  readonly logo: string;
  /** Dégradé de marque à 155°, valeurs exactes du design. */
  readonly degrade: string;
  /** Ombre portée colorée : au repos puis au survol. */
  readonly ombre: string;
  readonly ombreSurvol: string;
  /** Rotation initiale et décalage vertical de la vague (§7.7). Le décalage alterne
   *  entre deux valeurs : avec sept pastilles la ligne se replie selon la largeur, et
   *  une vague irrégulière donnerait des marches au point de repli. */
  readonly rotation: number;
  readonly decalage: number;
  readonly badge?: string;
};

/** Les sept fournisseurs de `FOURNISSEURS` (src/features/settings/model/providers.ts),
 *  dans le même ordre que la grille des réglages.
 *
 *  Les noms sont ceux que l'application affiche : « Claude », pas « Anthropic ». Les deux
 *  derniers ne sont pas des marques — NVIDIA passe par ses NIM, et « Personnalisé » désigne
 *  n'importe quel point d'accès compatible OpenAI. */
export const FOURNISSEURS_IA: readonly FournisseurIa[] = [
  {
    nom: "Ollama",
    logo: "ollama",
    degrade: "linear-gradient(155deg,#7d8cff,#3a48c4)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 24px rgba(61,75,199,0.36)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 36px rgba(61,75,199,0.42)",
    rotation: -3,
    decalage: 26,
    badge: "En local",
  },
  {
    nom: "Claude",
    logo: "claude",
    degrade: "linear-gradient(155deg,#eda184,#bf5029)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(191,80,41,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(191,80,41,0.34)",
    rotation: 5,
    decalage: 4,
  },
  {
    nom: "OpenAI",
    logo: "openai",
    degrade: "linear-gradient(155deg,#3f3f3f,#0a0a0a)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.22), 0 10px 22px rgba(16,18,24,0.26)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.24), 0 18px 34px rgba(16,18,24,0.32)",
    rotation: -7,
    decalage: 26,
  },
  {
    nom: "Gemini",
    logo: "googlegemini",
    degrade: "linear-gradient(155deg,#6f9bf5,#2450cf)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(36,80,207,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(36,80,207,0.34)",
    rotation: -4,
    decalage: 4,
  },
  {
    nom: "Mistral",
    logo: "mistralai",
    degrade: "linear-gradient(155deg,#ff9a52,#dc4302)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(220,67,2,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(220,67,2,0.34)",
    rotation: 8,
    decalage: 26,
  },
  {
    nom: "NVIDIA",
    logo: "nvidia",
    degrade: "linear-gradient(155deg,#a5e05a,#5c9200)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(92,146,0,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(92,146,0,0.34)",
    rotation: -6,
    decalage: 4,
  },
  {
    nom: "Personnalisé",
    logo: "custom",
    degrade: "linear-gradient(155deg,#7c8496,#454b58)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.24), 0 10px 22px rgba(45,50,60,0.26)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.26), 0 18px 34px rgba(45,50,60,0.32)",
    rotation: 4,
    decalage: 26,
    badge: "Compatible OpenAI",
  },
];
