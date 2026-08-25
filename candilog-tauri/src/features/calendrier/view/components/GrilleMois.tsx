import type { JourGrille } from "../../model/mois";
import { JOURS } from "../../model/mois";
import type { EvenementCalendrier } from "../../model/evenement";
import { Icon } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/** Nombre d'événements affichés par case avant le repli « +N ». */
const MAX_PAR_CASE = 3;

/**
 * Classes des pastilles d'événement, par tonalité.
 *
 * Table statique et non interpolation : Tailwind n'émet que les classes qu'il trouve
 * littéralement dans les sources.
 */
const PASTILLE: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Grille mensuelle : six semaines de sept jours.
 *
 * Le nombre de cases est fixe (42) : une grille à hauteur variable ferait sauter la mise en
 * page d'un mois à l'autre. Les jours hors du mois affiché sont estompés, sans être masqués
 * — ils portent de vrais événements.
 */
export function GrilleMois({
  cases,
  parJour,
  onJourClick,
  onEvenementClick,
}: {
  cases: readonly JourGrille[];
  parJour: Map<string, EvenementCalendrier[]>;
  onJourClick: (iso: string) => void;
  onEvenementClick: (evenement: EvenementCalendrier) => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
      <div className="grid flex-none grid-cols-7 border-b border-line bg-surface-alt">
        {JOURS.map((jour) => (
          <div key={jour} className="px-2 py-2 text-eyebrow uppercase text-ink-faint">
            {jour}
          </div>
        ))}
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6">
        {cases.map((jour) => {
          const evenements = parJour.get(jour.iso) ?? [];
          const visibles = evenements.slice(0, MAX_PAR_CASE);
          const surplus = evenements.length - visibles.length;

          return (
            <div
              key={jour.iso}
              className={cn(
                "flex min-h-0 flex-col gap-1 overflow-hidden border-r border-b border-line p-1.5",
                "last-in-row:border-r-0",
                jour.dansLeMois ? "bg-surface" : "bg-surface-alt",
              )}
            >
              <button
                type="button"
                onClick={() => onJourClick(jour.iso)}
                aria-label={`Ajouter au ${jour.numero}`}
                className={cn(
                  "tabular flex size-6 flex-none items-center justify-center rounded-pill text-meta",
                  "transition-colors duration-150",
                  jour.aujourdhui
                    ? "bg-accent font-medium text-white"
                    : jour.dansLeMois
                      ? "text-ink-muted hover:bg-neutral-tint hover:text-ink"
                      : "text-ink-faint hover:bg-neutral-tint",
                )}
              >
                {jour.numero}
              </button>

              <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-hidden">
                {visibles.map((evenement) => (
                  <button
                    key={evenement.id}
                    type="button"
                    onClick={() => onEvenementClick(evenement)}
                    title={`${evenement.libelle}${evenement.detail ? ` — ${evenement.detail}` : ""}`}
                    className={cn(
                      "flex w-full items-center gap-1 rounded-pill px-1.5 py-0.5 text-left text-meta",
                      "transition-[filter] duration-150 hover:brightness-95",
                      PASTILLE[evenement.tone],
                    )}
                  >
                    <Icon name={evenement.icone} size={11} className="flex-none" />
                    {evenement.heure ? (
                      <span className="tabular flex-none">{evenement.heure}</span>
                    ) : null}
                    <span className="truncate">{evenement.libelle}</span>
                  </button>
                ))}

                {surplus > 0 ? (
                  <span className="px-1.5 text-meta text-ink-faint">+{surplus}</span>
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
