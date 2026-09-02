import Link from "next/link";
import type { ReactNode } from "react";

export type EntreeFaq = { readonly question: string; readonly reponse: ReactNode };

/* ⚠️ Les réponses sur la confidentialité, la licence et l'ATS sont juridiquement
   calibrées : elles ne promettent ni « aucune donnée ne quitte votre ordinateur »
   ni un résultat de recrutement. Ne pas les reformuler. */
export const FAQ: readonly EntreeFaq[] = [
  {
    question: "Qu'est-ce que Candilog ?",
    reponse:
      "Candilog est une application desktop conçue pour organiser votre recherche d'emploi. Elle centralise vos candidatures, entreprises, contacts, entretiens, relances et documents afin de garder une vue claire sur l'ensemble de vos démarches.",
  },
  {
    question: "Candilog est-il gratuit ?",
    reponse: (
      <>
        Oui, pour un usage personnel. Candilog se télécharge et s&apos;utilise gratuitement, sans
        compte ni abonnement : les usages autorisés non commerciaux sont couverts par la PolyForm
        Noncommercial License 1.0.0. Si vous connectez un fournisseur d&apos;IA en ligne, c&apos;est
        vous qui réglez directement votre consommation auprès de lui ; un modèle local avec Ollama
        n&apos;entraîne aucun coût. Une utilisation commerciale nécessite une{" "}
        <Link href="/licence">licence commerciale séparée</Link>.
      </>
    ),
  },
  {
    question: "Que puis-je faire avec Candilog ?",
    reponse:
      "Vous pouvez suivre vos candidatures en Kanban ou en liste, tenir un répertoire d'entreprises et de contacts, organiser vos entretiens et relances dans un calendrier, gérer votre profil professionnel, générer des CV et des lettres ciblés, les exporter en PDF et suivre vos statistiques.",
  },
  {
    question: "À quoi sert l'analyse ATS ?",
    reponse:
      "L'analyse ATS vous aide à comparer votre CV avec une offre d'emploi, à identifier les compétences et mots-clés importants et à repérer les éléments qui pourraient être améliorés. Il s'agit d'une aide à la préparation de votre candidature, pas d'une garantie de passer un système ATS ou d'obtenir un entretien.",
  },
  {
    question: "Quel rôle joue l'IA dans Candilog ?",
    reponse:
      "L'IA intervient comme un assistant pour certaines tâches, notamment l'analyse d'offres, l'adaptation de documents et la génération de suggestions. Elle ne remplace pas vos décisions : vous gardez le contrôle sur le contenu utilisé dans vos candidatures.",
  },
  {
    question: "Mes données restent-elles privées ?",
    reponse:
      "Vos candidatures, documents et notes sont enregistrés sur votre ordinateur. Si vous connectez un fournisseur d'IA en ligne, le contenu nécessaire à la tâche demandée (une offre, un CV) est transmis à ce fournisseur au moment où vous lancez l'analyse. En choisissant un modèle local avec Ollama, aucune donnée n'est envoyée à un service externe.",
  },
  {
    question: "Sur quels systèmes Candilog est-il disponible ?",
    reponse:
      "Candilog est conçu pour fonctionner sur Windows, macOS et Linux, avec notamment une prise en charge de distributions comme Ubuntu et Fedora.",
  },
  {
    question: "Le code source est-il disponible, et puis-je le modifier ?",
    reponse:
      "Oui. Le code source de Candilog est publiquement accessible : c'est un projet source available, distribué pour les usages autorisés sous la PolyForm Noncommercial License 1.0.0. Vous pouvez le consulter, l'étudier, proposer des améliorations et effectuer les modifications prévues par cette licence.",
  },
  {
    question: "Puis-je utiliser ou revendre Candilog dans un cadre commercial ?",
    reponse:
      "L'utilisation commerciale n'est pas accordée par défaut. Toute exploitation commerciale, intégration dans une offre payante ou commercialisation de Candilog nécessite une licence commerciale séparée et l'autorisation du titulaire des droits.",
  },
];
