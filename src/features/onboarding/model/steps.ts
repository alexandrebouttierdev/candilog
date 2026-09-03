import type { IconName } from "@/shared/ui/icon-names";

/** Identifie l'aperçu schématique associé à une étape (`OnboardingPreview`). */
export type OnboardingPreviewKind =
  | "welcome"
  | "today"
  | "kanban"
  | "network"
  | "documents"
  | "analytics"
  | "profile"
  | "ai"
  | "closing";

export interface OnboardingStep {
  readonly kind: OnboardingPreviewKind;
  readonly icon: IconName | null;
  readonly eyebrow: string;
  readonly title: string;
  readonly description: string;
}

/**
 * Contenu du tour d'accueil, une étape par section du rail plus une ouverture et une clôture.
 *
 * L'ordre suit exactement `app/router/routes.ts` : découvrir l'application dans l'ordre où
 * le rail la présente, pas dans un ordre pédagogique qui s'en écarterait.
 */
export const ONBOARDING_STEPS: readonly OnboardingStep[] = [
  {
    kind: "welcome",
    icon: null,
    eyebrow: "Bienvenue",
    title: "Bienvenue dans Candilog",
    description:
      "Votre espace de suivi de recherche d'emploi, pensé pour rester rapide et confidentiel : candidatures, entreprises, entretiens, CV et lettres — tout reste sur votre machine. Ce court tour présente chaque écran avant de commencer.",
  },
  {
    kind: "today",
    icon: "today",
    eyebrow: "Aujourd'hui",
    title: "Un tableau de bord qui sait ce qui compte",
    description:
      "Chaque ouverture affiche vos prochains entretiens et relances, vos candidatures récentes, votre activité des dernières semaines et la répartition de votre pipeline — sans qu'il y ait rien à configurer.",
  },
  {
    kind: "kanban",
    icon: "work",
    eyebrow: "Candidatures et calendrier",
    title: "Un suivi dense, à votre rythme",
    description:
      "Faites glisser vos candidatures entre En attente, Relancée, Entretien et Refus dans un Kanban, ou basculez en liste triable. Le calendrier réunit vos entretiens et vos relances au même endroit.",
  },
  {
    kind: "network",
    icon: "hub",
    eyebrow: "Entreprises et réseau",
    title: "Entreprises et contacts, centralisés",
    description:
      "Gardez une fiche par entreprise visée et par contact — recruteur, manager — avec leurs coordonnées, reliées automatiquement aux candidatures concernées.",
  },
  {
    kind: "documents",
    icon: "description",
    eyebrow: "CV et lettres de motivation",
    title: "Des documents ciblés, assistés par l'IA",
    description:
      "Générez un CV adapté à une offre et une lettre de motivation cohérente, en validant chaque proposition avant de l'enregistrer. La mise en page A4 reste maîtrisée du premier au dernier document.",
  },
  {
    kind: "analytics",
    icon: "monitoring",
    eyebrow: "Statistiques",
    title: "Votre recherche, en chiffres",
    description:
      "Taux de réponse, funnel de conversion et candidatures à relancer, sur 30 jours, 90 jours ou depuis le début — pour ajuster votre stratégie plutôt que la deviner.",
  },
  {
    kind: "profile",
    icon: "account_circle",
    eyebrow: "Profil professionnel",
    title: "La source de vérité de vos documents",
    description:
      "Identité, expériences, compétences, formations — avec une photo facultative. Importez directement un profil depuis un CV existant, l'IA propose et vous validez chaque champ.",
  },
  {
    kind: "ai",
    icon: "smart_toy",
    eyebrow: "Intelligence artificielle",
    title: "Votre moteur, votre choix",
    description:
      "Claude, OpenAI, Gemini, Mistral, NVIDIA — ou Ollama en local, sans clé API ni connexion, pour que vos données ne quittent jamais votre machine.",
  },
  {
    kind: "closing",
    icon: "rocket_launch",
    eyebrow: "C'est parti",
    title: "Vous êtes prêt à démarrer",
    description:
      "Retrouvez à tout moment le fournisseur IA, les sauvegardes et les mises à jour dans Réglages. Bonne recherche !",
  },
] as const;
