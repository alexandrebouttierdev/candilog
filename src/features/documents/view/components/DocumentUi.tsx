import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
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
        "overflow-hidden rounded-card border border-bd bg-panel",
        className,
      )}
    >
      <header className="flex items-center gap-2 border-b border-bd px-[18px] py-[14px]">
        <h2 className="min-w-0 flex-1 truncate text-item font-semibold text-tx">{title}</h2>
        {action}
      </header>
      {children}
    </section>
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
