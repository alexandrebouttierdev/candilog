import type { ReactNode } from "react";
import type { GeneratedResume, AiProgress } from "@/features/ai/model/types";
import { cn } from "@/shared/lib/cn";
import { formatElapsed, formatTokens } from "@/shared/lib/duration";
import { Icon, IconButton, StatusPill } from "@/shared/ui";
import { PaperPreview } from "./PaperPreview";
import type { IconName } from "@/shared/ui/icon-names";

/**
 * Carte de document : filet, rayon 12 px, en-tête 13 px/600 à 14 px / 18 px.
 *
 * Sert les écrans de génération et d'analyse, où le contenu n'est pas en maître-détail.
 */
export function DocumentPanel({
  title,
  icon,
  action,
  children,
  className,
}: {
  title: string;
  icon: IconName;
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
        <Icon name={icon} size={17} className="flex-none text-ink-faint" />
        <h2 className="min-w-0 flex-1 truncate text-item font-semibold text-ink">{title}</h2>
        {action}
      </header>
      {children}
    </section>
  );
}

/**
 * Feuille A4 des maquettes Documents : 560 px, 38 px / 42 px, ombre papier, rayon 4 px.
 *
 * Le fond reste blanc en thème sombre : un CV imprimé n'inverse pas ses encres.
 */
export function A4Preview({
  resume,
  title = "Aperçu du document",
  children,
}: {
  resume?: GeneratedResume | undefined;
  title?: string;
  children?: ReactNode;
}) {
  return (
    <div className="flex min-h-0 flex-1 justify-center overflow-auto bg-page p-[26px]">
      <PaperPreview title={title}>
        {children ??
          (resume ? (
            <>
              <h2 className="text-display text-paper-ink">{resume.resume || "Profil professionnel"}</h2>
              <Section title="Compétences">
                <p>{resume.skills.join(" · ")}</p>
              </Section>
              <Section title="Expériences">
                {resume.experiences.map((item, index) => (
                  <div key={`${item.title}-${index}`} className="mb-[9px]">
                    <p className="text-label font-semibold">{item.title} · {item.company}</p>
                    <p className="mt-[3px] text-[10.5px] leading-[1.55] text-paper-muted">
                      {item.description}
                    </p>
                  </div>
                ))}
              </Section>
              <Section title="Formations">
                {resume.education.map((item, index) => (
                  <p key={`${item.degree}-${index}`} className="text-label">
                    <span className="font-semibold">{item.degree}</span> · {item.school}
                  </p>
                ))}
              </Section>
            </>
          ) : (
            <div className="flex min-h-[590px] items-center justify-center text-center text-[12px] text-paper-muted">
              Le document apparaîtra ici après la génération.
            </div>
          ))}
      </PaperPreview>
    </div>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mt-[18px]">
      <h3 className="mb-[9px] border-b border-paper-border pb-[5px] text-[10px] font-bold tracking-[0.1em] text-paper-muted uppercase">
        {title}
      </h3>
      <div className="text-[11.5px] leading-[1.55]">{children}</div>
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
  return (
    <div role="status" className="rounded-card border border-accent-border bg-accent-tint p-4">
      <div className="flex items-center gap-2">
        <Icon name="progress_activity" size={17} className="animate-spin text-accent" />
        <p className="flex-1 text-label font-medium text-ink">{progress?.step ?? "Préparation…"}</p>
        {progress?.tokens_used !== null && progress?.tokens_used !== undefined ? (
          <span className="tabular text-meta text-accent">
            {formatTokens(progress.tokens_used)} tokens
          </span>
        ) : null}
        <span className="tabular text-meta text-accent">{formatElapsed(elapsedMs)}</span>
      </div>
      <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-surface">
        <div className="import-indeterminate h-full w-1/3 rounded-full bg-accent" />
      </div>
    </div>
  );
}

export function ScoreBadge({ value, label = "Score ATS" }: { value: number; label?: string }) {
  return (
    <div className="flex items-center gap-3">
      <span
        className={cn(
          "tabular flex size-12 items-center justify-center rounded-full border-4 text-label font-semibold",
          value >= 70
            ? "border-success text-success"
            : value >= 45
              ? "border-warning text-warning"
              : "border-danger text-danger",
        )}
      >
        {value}
      </span>
      <div>
        <p className="text-label font-medium text-ink">{label}</p>
        <p className="text-meta text-ink-faint">sur 100</p>
      </div>
    </div>
  );
}

/**
 * Annuler/rétablir de la barre Document : deux `IconButton` groupées, comme dans les autres
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
      <IconButton icon="undo" label="Annuler la dernière modification" disabled={!canUndo} onClick={onUndo} />
      <IconButton icon="redo" label="Rétablir la modification" disabled={!canRedo} onClick={onRedo} />
    </div>
  );
}

/**
 * État tenant sur une page A4 : vert quand c'est le cas, ambre sinon — jamais seulement une
 * couleur, l'énoncé porte toujours l'information (règle du guide, cf. `StatusPill`).
 */
export function OverflowStatus({ overflow }: { overflow: boolean }) {
  return overflow ? (
    <StatusPill tone="warning" icon="warning">Contenu trop long</StatusPill>
  ) : (
    <StatusPill tone="success" icon="check_circle">Une page A4</StatusPill>
  );
}

/**
 * Action de la barre d'aperçu : 29 px, rayon 7 px, comme les maquettes Documents.
 */
export function PreviewAction({
  icon,
  children,
  onClick,
  disabled,
  tone,
}: {
  icon: IconName;
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  tone?: "danger";
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "inline-flex h-[29px] items-center gap-1.5 rounded-control border px-[11px]",
        "text-note font-medium transition-colors duration-150",
        "disabled:pointer-events-none disabled:text-ink-faint",
        tone === "danger"
          ? "border-danger-border bg-surface text-danger hover:bg-danger-tint"
          : "border-line bg-surface text-ink-muted hover:bg-neutral-tint",
      )}
    >
      <Icon name={icon} size={15} />
      {children}
    </button>
  );
}
