import type { LetterFit, LetterRecommendation } from "@/shared/types/generated/ai";
import { Button, StatusGlyph } from "@/shared/ui";
import { PaneSection } from "./GeneratorFrame";

/**
 * Adéquation et recommandations de la lettre (`screens/16`, colonne droite). Le score dit
 * quelle part des exigences de l'offre la lettre aborde ; chaque recommandation cite le fait
 * du profil qui permet d'en aborder une de plus. Appliquer passe par la correction de la
 * lettre, bornée aux faits vérifiés.
 */
export function LetterFitPanel({
  reading,
  fit,
  potential,
  recommendations,
  error,
  busy,
  onApply,
  onIgnore,
}: {
  reading: boolean;
  fit: LetterFit | null;
  potential: number | null;
  recommendations: readonly LetterRecommendation[];
  error: string | null;
  busy: boolean;
  onApply: (recommendation: LetterRecommendation) => void;
  onIgnore: (id: string) => void;
}) {
  if (error) {
    return (
      <PaneSection title="Adéquation">
        <p className="text-small leading-[1.5] text-tx-5">Adéquation indisponible : {error}</p>
      </PaneSection>
    );
  }
  if (reading || fit === null) {
    return (
      <PaneSection title="Adéquation">
        <p role="status" className="text-small text-tx-5">
          Lecture de l’offre…
        </p>
      </PaneSection>
    );
  }
  if (fit.requirements === 0) {
    return (
      <PaneSection title="Adéquation">
        <p className="text-small leading-[1.5] text-tx-5">Aucune exigence lue dans l’offre : rien à mesurer.</p>
      </PaneSection>
    );
  }

  return (
    <>
      <PaneSection title="Adéquation">
        <p className="flex items-baseline gap-1.5">
          <span className="serif-title text-[24px] leading-none text-tx">{fit.score}</span>
          <span className="text-small text-tx-4">/ 100 · adéquation</span>
          {potential !== null && potential > fit.score ? (
            <span className="ml-auto font-mono text-caps text-tx-5">jusqu’à {potential}</span>
          ) : null}
        </p>
        <div aria-hidden className="relative mt-2 h-[3px] overflow-hidden rounded-r2 bg-chip">
          {potential !== null ? (
            <div className="absolute inset-y-0 left-0 rounded-r2 bg-tint-ac-bg" style={{ width: `${potential}%` }} />
          ) : null}
          <div className="absolute inset-y-0 left-0 rounded-r2 bg-ac" style={{ width: `${fit.score}%` }} />
        </div>
        <p className="mt-2 text-tiny leading-[1.45] text-tx-5">
          Part des {fit.requirements} exigences de l’offre que la lettre aborde, pondérée par leur importance.
        </p>
      </PaneSection>

      {recommendations.length > 0 ? (
        <PaneSection title="Recommandations" aside={`${recommendations.length} à traiter`}>
          <ul className="flex flex-col gap-2">
            {recommendations.map((recommendation) => (
              <li key={recommendation.id} className="rounded-r8 bg-group px-3 py-2.5">
                <p className="flex items-start gap-2">
                  <span className="mt-[5px]">
                    <StatusGlyph tone="a" small />
                  </span>
                  <span className="flex-1 text-small font-medium text-tx">
                    Aborder « {recommendation.requirement} »
                  </span>
                  <span className="rounded-r5 bg-tint-ac-bg px-1.5 font-mono text-caps text-tint-ac-tx">
                    +{recommendation.impact}
                  </span>
                </p>
                <p className="mt-1 line-clamp-3 pl-[17px] text-tiny leading-[1.45] text-tx-4">
                  L’offre le demande et votre profil le montre : « {recommendation.evidence} »
                </p>
                <p className="mt-2 flex gap-1.5 pl-[17px]">
                  <Button variant="primary" size="compact" disabled={busy} onClick={() => onApply(recommendation)}>
                    Appliquer
                  </Button>
                  <Button size="compact" disabled={busy} onClick={() => onIgnore(recommendation.id)}>
                    Ignorer
                  </Button>
                </p>
              </li>
            ))}
          </ul>
        </PaneSection>
      ) : null}

      {fit.unsupported.length > 0 ? (
        <PaneSection title="Absent de votre profil">
          <p className="text-tiny leading-[1.45] text-tx-5">
            {fit.unsupported.join(", ")} — la lettre ne l’invente pas. Complétez votre profil si c’est vrai.
          </p>
        </PaneSection>
      ) : null}
    </>
  );
}
