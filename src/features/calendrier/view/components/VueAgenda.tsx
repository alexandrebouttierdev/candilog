import type { JourGrille } from "../../model/mois";
import { JOURS } from "../../model/mois";
import type { EvenementCalendrier } from "../../model/evenement";
import { Button, EmptyState, Icon } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

const PASTILLE: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Vue semaine : une rangée de sept jours, mêmes pastilles que la grille mensuelle.
 */
export function VueSemaine({
  jours,
  parJour,
  selection,
  onJourClick,
  onEvenementClick,
}: {
  jours: readonly JourGrille[];
  parJour: Map<string, EvenementCalendrier[]>;
  selection: string;
  onJourClick: (iso: string) => void;
  onEvenementClick: (evenement: EvenementCalendrier) => void;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
      <div className="grid flex-none grid-cols-7 border-b border-line bg-surface-alt">
        {JOURS.map((jour) => (
          <div key={jour} className="px-2 py-2 text-center text-eyebrow uppercase text-ink-faint">
            {jour}
          </div>
        ))}
      </div>
      <div className="grid min-h-0 flex-1 grid-cols-7">
        {jours.map((jour) => {
          const evenements = parJour.get(jour.iso) ?? [];
          return (
            <div
              key={jour.iso}
              className={cn(
                "flex min-h-0 flex-col gap-1 overflow-hidden border-r border-line p-2 last:border-r-0",
                jour.iso === selection ? "bg-accent-tint/40" : jour.dansLeMois ? "bg-surface" : "bg-surface-alt",
              )}
            >
              <button
                type="button"
                onClick={() => onJourClick(jour.iso)}
                aria-label={`Ajouter au ${jour.numero}`}
                aria-pressed={jour.iso === selection}
                className={cn(
                  "tabular flex size-6 flex-none items-center justify-center rounded-pill text-meta",
                  jour.aujourdhui
                    ? "bg-accent font-medium text-white"
                    : jour.dansLeMois
                      ? "text-ink-muted hover:bg-neutral-tint hover:text-ink"
                      : "text-ink-faint hover:bg-neutral-tint",
                )}
              >
                {jour.numero}
              </button>
              <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto">
                {evenements.map((evenement) => (
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
                    {evenement.heure ? <span className="tabular flex-none">{evenement.heure}</span> : null}
                    <span className="truncate">{evenement.libelle}</span>
                  </button>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

/**
 * Vue jour : liste des événements de la journée sélectionnée.
 */
export function VueJour({
  evenements,
  onJourClick,
  onEvenementClick,
  jour,
}: {
  evenements: readonly EvenementCalendrier[];
  jour: string;
  onJourClick: (iso: string) => void;
  onEvenementClick: (evenement: EvenementCalendrier) => void;
}) {
  if (evenements.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center overflow-hidden rounded-card border border-line bg-surface">
        <EmptyState
          icon="event_available"
          title="Rien de prévu"
          description="Ajoutez un entretien ou une relance pour cette journée."
          action={<Button icon="add" onClick={() => onJourClick(jour)}>Ajouter un entretien</Button>}
        />
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
      <ul className="min-h-0 flex-1 overflow-y-auto p-3">
        {evenements.map((evenement) => (
          <li key={evenement.id} className="mb-1.5">
            <button
              type="button"
              onClick={() => onEvenementClick(evenement)}
              className={cn(
                "flex w-full items-center gap-3 rounded-[10px] px-3 py-2.5 text-left",
                "transition-colors duration-150 hover:brightness-95",
                PASTILLE[evenement.tone],
              )}
            >
              <Icon name={evenement.icone} size={18} className="flex-none" />
              <span className="min-w-0 flex-1">
                <span className="block truncate text-body font-medium">{evenement.libelle}</span>
                {evenement.detail ? (
                  <span className="mt-0.5 block truncate text-meta opacity-80">{evenement.detail}</span>
                ) : null}
              </span>
              {evenement.heure ? <span className="tabular flex-none text-meta">{evenement.heure}</span> : null}
            </button>
          </li>
        ))}
      </ul>
      <button
        type="button"
        onClick={() => onJourClick(jour)}
        className="flex h-11 flex-none items-center justify-center gap-1.5 border-t border-line text-body font-medium text-accent hover:bg-accent-tint"
      >
        <Icon name="add" size={16} />
        Ajouter un entretien
      </button>
    </div>
  );
}
