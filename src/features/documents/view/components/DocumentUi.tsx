import type { ReactNode } from "react";
import type { AiProgress } from "@/features/ai";
import { cn } from "@/shared/lib/cn";
import { formatElapsed, formatTokens, formatTokensPerSecond, resolveTokensPerSecond } from "@/shared/lib/duration";
import { GlyphButton, StatusPill } from "@/shared/ui";

/**
 * Carte de document : filet, rayon 12 px, en-tête 13 px/600 à 14 px / 18 px.
 *
 * Sert les écrans de génération et d'analyse, où le contenu n'est pas en maître-détail.
 */
export function DocumentPanel({
  title,
  action,
  children,
  className,
}: {
  title: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section
      className={cn(
        "overflow-hidden rounded-card border border-line bg-surface",
        className,
      )}
    >
      <header className="flex items-center gap-2 border-b border-line px-[18px] py-[14px]">
        <h2 className="min-w-0 flex-1 truncate text-item font-semibold text-ink">{title}</h2>
        {action}
      </header>
      {children}
    </section>
  );
}

/**
 * Progression indéterminée d'un traitement IA.
 *
 * Aucun pourcentage : la durée dépend du fournisseur et du modèle, et le chiffre affiché
 * jusqu'ici était une constante déguisée en mesure. Le temps écoulé est la seule
 * information vraie que l'on puisse donner pendant l'attente.
 */
export function AiProgress({
  progress,
  elapsedMs,
}: {
  progress: AiProgress | null;
  elapsedMs: number;
}) {
  const rate = resolveTokensPerSecond(progress?.tokens_used, elapsedMs);
  return (
    <div role="status" className="rounded-card border border-accent-border bg-accent-tint p-4">
      <div className="flex items-center gap-2">
        <span aria-hidden className="size-3 flex-none animate-spin-ring rounded-full border-[1.5px] border-accent border-t-transparent" />
        <p className="flex-1 text-label font-medium text-ink">{progress?.step ?? "Préparation…"}</p>
        {progress?.tokens_used !== null && progress?.tokens_used !== undefined ? (
          <span className="tabular text-meta text-accent">
            {formatTokens(progress.tokens_used)} tokens
          </span>
        ) : null}
        {rate !== null ? (
          <span className="tabular text-meta text-accent">{formatTokensPerSecond(rate)}</span>
        ) : null}
        <span className="tabular text-meta text-accent">{formatElapsed(elapsedMs)}</span>
      </div>
      <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-surface">
        <div className="import-indeterminate h-full w-1/3 rounded-full bg-accent" />
      </div>
    </div>
  );
}

/**
 * Annuler/rétablir de la barre Document : deux `GlyphButton` groupés, comme dans les autres
 * barres d'outils du guide plutôt qu'un `Button` texte qui alourdirait l'en-tête.
 */
export function UndoRedoControls({
  canUndo,
  canRedo,
  onUndo,
  onRedo,
}: {
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
}) {
  return (
    <div className="flex items-center gap-1.5">
      <GlyphButton glyph="↶" label="Annuler la dernière modification" disabled={!canUndo} onClick={onUndo} />
      <GlyphButton glyph="↷" label="Rétablir la modification" disabled={!canRedo} onClick={onRedo} />
    </div>
  );
}

/**
 * État tenant sur une page A4 : vert quand c'est le cas, ambre sinon — jamais seulement une
 * couleur, l'énoncé porte toujours l'information (règle du guide, cf. `StatusPill`).
 */
export function OverflowStatus({ overflow }: { overflow: boolean }) {
  return overflow ? (
    <StatusPill tone="warning">Contenu trop long</StatusPill>
  ) : (
    <StatusPill tone="success">Une page A4</StatusPill>
  );
}

/**
 * Action de la barre d'aperçu : 29 px, rayon 7 px, comme les maquettes Documents.
 */
