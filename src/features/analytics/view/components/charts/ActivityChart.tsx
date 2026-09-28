import type { ActivityWeek } from "@/shared/types/generated/analytics";
import { EmptyState } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { formatDate } from "../analyticsDates";

/** Étiquettes d'axe visibles au plus : au-delà, elles se chevaucheraient. */
const MAX_LABELS = 7;

/**
 * Candidatures envoyées, semaine par semaine (`screens/10`, « Rythme d'envoi »).
 *
 * Dessiné avec les primitives du design (`DECISIONS.md` D7), sans bibliothèque de
 * graphiques : une barre par semaine, un socle pour une semaine à zéro, la valeur la plus
 * haute en repère. Une seule série : pas de légende, le bloc qui l'accueille la nomme déjà.
 * Chaque valeur reste lisible sans survol, dans la liste réservée aux lecteurs d'écran.
 */
export function ActivityChart({
  activity,
  height = 150,
  shortLabels = false,
}: {
  activity: readonly ActivityWeek[];
  height?: number;
  /** Étiquettes numériques `JJ/MM`, pour les séries longues ou les cartes étroites. */
  shortLabels?: boolean;
}) {
  if (activity.every((week) => week.count === 0)) {
    return (
      <EmptyState
        title="Pas encore d’activité"
        description="Les candidatures envoyées apparaîtront ici semaine après semaine."
      />
    );
  }

  const max = Math.max(...activity.map((week) => week.count));
  // Première et dernière semaines toujours étiquetées, les autres à pas régulier.
  const step = Math.max(1, Math.ceil(activity.length / MAX_LABELS));
  const labelled = (index: number) => index === 0 || index === activity.length - 1 || index % step === 0;
  const label = (start: string) => formatDate(start, shortLabels ? "numeric" : "court");

  return (
    <>
      <div role="img" aria-label="Candidatures envoyées par semaine" className="flex gap-2">
        <div aria-hidden className="flex flex-col justify-between pb-5 text-right font-mono text-caps text-tx-6" style={{ height }}>
          <span>{max}</span>
          <span>0</span>
        </div>
        <div className="min-w-0 flex-1">
          <div aria-hidden className="relative flex items-end gap-[3px] border-b border-bd-soft" style={{ height: height - 20 }}>
            <span className="absolute inset-x-0 top-0 border-t border-dashed border-bd-soft" />
            {activity.map((week) => (
              <span
                key={week.start}
                data-bar
                title={`Semaine du ${formatDate(week.start, "long")} : ${week.count}`}
                className={cn("min-w-0 flex-1 rounded-t-r2", week.count === 0 ? "bg-chip" : "bg-ac")}
                style={{ height: week.count === 0 ? 2 : `${Math.max((week.count / max) * 100, 2)}%` }}
              />
            ))}
          </div>
          <div aria-hidden className="flex h-5 items-end gap-[3px] font-mono text-caps text-tx-5">
            {activity.map((week, index) => (
              <span
                key={week.start}
                className={cn(
                  "min-w-0 flex-1 overflow-visible whitespace-nowrap",
                  // La dernière étiquette s'aligne à droite pour ne pas déborder du bloc.
                  index === activity.length - 1 && index > 0 && "flex justify-end",
                )}
              >
                {labelled(index) ? label(week.start) : ""}
              </span>
            ))}
          </div>
        </div>
      </div>
      <ol className="sr-only">
        {activity.map((week) => (
          <li key={week.start}>
            Semaine du {formatDate(week.start, "long")} : {week.count} candidature
            {week.count > 1 ? "s" : ""}
          </li>
        ))}
      </ol>
    </>
  );
}
