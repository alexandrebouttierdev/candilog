import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { aiService } from "@/features/ai";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { AppError } from "@/shared/types/app-error";
import type { LetterRecommendation, ProfileSection } from "@/shared/types/generated/ai";

const LETTER_FIT_KEY = ["lettre-adequation"] as const;

/**
 * Adéquation de la lettre à l'offre (`screens/16`, colonne droite) : l'offre est lue une
 * fois par le modèle de la tâche « Extraire une offre », puis chaque version de la lettre
 * est mesurée localement. Une recommandation ignorée ne revient pas tant que l'écran vit.
 */
export function useLetterFit({
  letter,
  context,
  excludedSections,
  enabled,
}: {
  letter: string;
  context: string;
  excludedSections: readonly ProfileSection[];
  /** Une lettre existe et aucune rédaction n'est en cours. */
  enabled: boolean;
}) {
  const offerText = context.trim();
  const listing = useQuery({
    queryKey: [...LETTER_FIT_KEY, "offre", offerText],
    queryFn: () => aiService.analyzeListing(offerText),
    enabled: enabled && offerText !== "",
    staleTime: Number.POSITIVE_INFINITY,
    retry: false,
  });
  const jobOffer = listing.data?.output.job_offer ?? null;
  const measured = useDebounce(letter, 400);
  const fit = useQuery({
    queryKey: [...LETTER_FIT_KEY, measured, jobOffer, excludedSections],
    queryFn: () => {
      // `enabled` garantit l'offre : la requête ne part pas sans elle.
      if (jobOffer === null) throw new Error("Offre non lue");
      return aiService.evaluateCoverLetter({
        letter: measured,
        job_offer: jobOffer,
        excluded_sections: [...excludedSections],
      });
    },
    enabled: enabled && jobOffer !== null && measured.trim() !== "",
    placeholderData: (previous) => previous,
  });
  const [ignored, setIgnored] = useState<ReadonlySet<string>>(new Set());

  const recommendations: LetterRecommendation[] = (fit.data?.recommendations ?? []).filter(
    (recommendation) => !ignored.has(recommendation.id),
  );
  const potential =
    fit.data === undefined
      ? null
      : Math.min(100, fit.data.score + recommendations.reduce((sum, recommendation) => sum + recommendation.impact, 0));
  const failure = listing.error ?? fit.error;

  return {
    /** Pas d'offre à lire : l'adéquation n'a pas de sens. */
    available: offerText !== "",
    reading: listing.isFetching,
    fit: fit.data ?? null,
    potential,
    recommendations,
    error: failure ? (failure instanceof AppError ? failure.message : "L’adéquation n’a pas pu être mesurée.") : null,
    ignore: (id: string) => setIgnored((current) => new Set(current).add(id)),
  };
}
