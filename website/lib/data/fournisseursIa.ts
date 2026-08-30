export type FournisseurIa = {
  readonly nom: string;
  /** Nom du fichier dans public/providers, sans extension. */
  readonly logo: string;
  /** Dégradé de marque à 155°, valeurs exactes du design. */
  readonly degrade: string;
  /** Ombre portée colorée : au repos puis au survol. */
  readonly ombre: string;
  readonly ombreSurvol: string;
  /** Rotation initiale et décalage vertical de la vague (§7.7). */
  readonly rotation: number;
  readonly decalage: number;
  readonly badge?: string;
};

export const FOURNISSEURS_IA: readonly FournisseurIa[] = [
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
    nom: "Anthropic",
    logo: "anthropic",
    degrade: "linear-gradient(155deg,#eda184,#bf5029)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(191,80,41,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(191,80,41,0.34)",
    rotation: 5,
    decalage: 4,
  },
  {
    nom: "Gemini",
    logo: "googlegemini",
    degrade: "linear-gradient(155deg,#6f9bf5,#2450cf)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(36,80,207,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(36,80,207,0.34)",
    rotation: -4,
    decalage: 32,
  },
  {
    nom: "Mistral AI",
    logo: "mistralai",
    degrade: "linear-gradient(155deg,#ff9a52,#dc4302)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 22px rgba(220,67,2,0.28)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 34px rgba(220,67,2,0.34)",
    rotation: 8,
    decalage: 8,
  },
  {
    nom: "Ollama",
    logo: "ollama",
    degrade: "linear-gradient(155deg,#7d8cff,#3a48c4)",
    ombre: "inset 0 1.5px 0 rgba(255,255,255,0.32), 0 10px 24px rgba(61,75,199,0.36)",
    ombreSurvol: "inset 0 1.5px 0 rgba(255,255,255,0.34), 0 18px 36px rgba(61,75,199,0.42)",
    rotation: -3,
    decalage: 34,
    badge: "En local",
  },
];
