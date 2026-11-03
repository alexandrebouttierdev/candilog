import type { ProviderOption } from "./providers";

/** Faits affichés sous le nom d'un fournisseur : aucune promesse de qualité, des faits. */
export interface ProviderFacts {
  readonly description: string;
  readonly meters: ReadonlyArray<{ label: string; value: string; level: number; tone: string }>;
}

const GOOD = "bg-st-g";
const WARN = "bg-st-a";

function remoteFacts(label: string): ProviderFacts {
  return {
    description: `Les données de chaque tâche confiée à ${label} lui sont envoyées pour être traitées. La rédaction est généralement plus rapide et plus fine qu'en local ; l'usage est facturé par ${label} selon votre compte.`,
    meters: [
      { label: "Confidentialité", value: `Envoyé à ${label}`, level: 1, tone: WARN },
      { label: "Coût", value: "Facturé à l'usage", level: 1, tone: WARN },
      { label: "Hors connexion", value: "Impossible", level: 0, tone: WARN },
    ],
  };
}

export const FACTS: Record<ProviderOption["id"], (label: string) => ProviderFacts> = {
  candilog_local: () => ({
    description:
      "Les modèles tournent sur votre machine. Vos CV, vos lettres et vos candidatures ne sortent jamais de l'ordinateur. C'est gratuit et utilisable hors connexion ; en contrepartie, c'est plus lent et la qualité dépend de la taille du modèle installé.",
    meters: [
      { label: "Confidentialité", value: "Rien ne sort", level: 3, tone: GOOD },
      { label: "Coût", value: "Gratuit", level: 3, tone: GOOD },
      { label: "Hors connexion", value: "Oui", level: 3, tone: GOOD },
    ],
  }),
  ollama: () => ({
    description:
      "Votre propre installation d'Ollama, sur cet ordinateur ou sur votre réseau. Les données restent chez vous, sans service tiers.",
    meters: [
      { label: "Confidentialité", value: "Reste chez vous", level: 3, tone: GOOD },
      { label: "Coût", value: "Gratuit", level: 3, tone: GOOD },
      { label: "Hors connexion", value: "Selon l'installation", level: 2, tone: GOOD },
    ],
  }),
  claude: remoteFacts,
  openai: remoteFacts,
  gemini: remoteFacts,
  mistral: remoteFacts,
  deepseek: remoteFacts,
  custom: () => ({
    description:
      "Un service compatible OpenAI — LM Studio, un serveur interne… Les données vont à l'adresse que vous indiquez : locale ou distante, c'est elle qui décide.",
    meters: [
      { label: "Confidentialité", value: "Selon l'adresse", level: 2, tone: WARN },
      { label: "Coût", value: "Selon le service", level: 2, tone: WARN },
      { label: "Hors connexion", value: "Selon l'adresse", level: 1, tone: WARN },
    ],
  }),
};
