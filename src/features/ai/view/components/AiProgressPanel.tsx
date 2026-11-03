import type { AiProgress } from "../../model/types";
import { formatElapsed, formatTokens, formatTokensPerSecond, resolveTokensPerSecond } from "@/shared/lib/duration";

/**
 * Progression indéterminée d'un traitement IA.
 *
 * Aucun pourcentage : la durée dépend du fournisseur et du modèle, et le chiffre affiché
 * jusqu'ici était une constante déguisée en mesure. Le temps écoulé est la seule
 * information vraie que l'on puisse donner pendant l'attente.
 */
export function AiProgressPanel({
  progress,
  elapsedMs,
}: {
  progress: AiProgress | null;
  elapsedMs: number;
}) {
  const rate = resolveTokensPerSecond(progress?.tokens_used, elapsedMs);
  return (
    <div role="status" className="rounded-card border border-accent-border bg-tint-ac-bg p-4">
      <div className="flex items-center gap-2">
        <span aria-hidden className="size-3 flex-none animate-spin-ring rounded-full border-[1.5px] border-ac border-t-transparent" />
        <p className="flex-1 text-label font-medium text-tx">{progress?.step ?? "Préparation…"}</p>
        {progress?.tokens_used !== null && progress?.tokens_used !== undefined ? (
          <span className="tabular text-meta text-ac-tx">
            {formatTokens(progress.tokens_used)} tokens
          </span>
        ) : null}
        {rate !== null ? (
          <span className="tabular text-meta text-ac-tx">{formatTokensPerSecond(rate)}</span>
        ) : null}
        <span className="tabular text-meta text-ac-tx">{formatElapsed(elapsedMs)}</span>
      </div>
      <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-panel">
        <div className="import-indeterminate h-full w-1/3 rounded-full bg-ac" />
      </div>
    </div>
  );
}
