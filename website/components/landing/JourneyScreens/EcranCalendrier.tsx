import { Icon } from "@/components/ui/Icon";
import { cn } from "@/lib/cn";
import {
  CASES_AVANT_SEPTEMBRE,
  CASES_GRILLE,
  EVENEMENTS_CALENDRIER,
  JOURS_CALENDRIER,
  JOURS_SEPTEMBRE,
  JOUR_COURANT,
} from "@/lib/data/parcours";

const VUES = ["Mois", "Semaine", "Jour"] as const;

/** Les 42 cases de la grille : la fin d'août, septembre, puis le début d'octobre.
 *  Le nombre est fixe — une grille à hauteur variable ferait sauter la mise en page
 *  d'un mois à l'autre (`GridMonth`). */
const CASES = Array.from({ length: CASES_GRILLE }, (_, index) => {
  const decalage = index - CASES_AVANT_SEPTEMBRE;
  if (decalage < 0) return { numero: 31 - (CASES_AVANT_SEPTEMBRE - 1 - index), dansLeMois: false };
  if (decalage >= JOURS_SEPTEMBRE)
    return { numero: decalage - JOURS_SEPTEMBRE + 1, dansLeMois: false };
  return { numero: decalage + 1, dansLeMois: true };
});

/** Écran 05 — Suivi → Calendrier, vue Mois.
 *
 *  Entretiens en vert, relances en ambre : ce sont les deux seuls types d'événement du
 *  calendrier, et ils viennent des candidatures — rien ne se saisit deux fois. */
export function EcranCalendrier() {
  return (
    <div className="flex min-h-[320px] flex-col">
      {/* ── Barre de période ─────────────────────────────────────────────── */}
      <div className="flex flex-none flex-wrap items-center gap-3 border-b border-line-soft bg-surface-alt px-[18px] py-[10px]">
        <div className="flex items-center">
          <span className="grid size-[26px] place-items-center rounded-l-control border border-control bg-surface text-ink-faint">
            <Icon name="chevron_left" size={15} />
          </span>
          <span className="grid size-[26px] place-items-center rounded-r-control border border-l-0 border-control bg-surface text-ink-muted">
            <Icon name="chevron_right" size={15} />
          </span>
        </div>

        <span className="inline-flex h-[26px] items-center gap-[6px] rounded-control border border-control bg-surface px-[9px] text-[12px] font-semibold text-ink">
          <Icon name="today" size={14} />
          Aujourd&apos;hui
        </span>

        <h3 className="text-[13px] font-semibold text-ink">septembre 2026</h3>

        <span className="flex-1" />

        <span className="inline-flex h-[19px] items-center gap-[4px] whitespace-nowrap rounded-pill border border-success-border bg-success-tint px-[7px] text-[11px] font-semibold text-success-text">
          <Icon name="event_available" size={12} />3 entretiens
        </span>
        <span className="inline-flex h-[19px] items-center gap-[4px] whitespace-nowrap rounded-pill border border-warning-border bg-warning-tint px-[7px] text-[11px] font-semibold text-warning-text">
          <Icon name="send" size={12} />3 relances
        </span>

        <div className="flex items-center gap-[2px] rounded-[9px] bg-page p-[2px]">
          {VUES.map((vue, index) => (
            <span
              key={vue}
              className={cn(
                "inline-flex h-7 items-center rounded-[6px] px-[10px] text-[11.5px] font-medium",
                index === 0 ? "bg-surface text-ink" : "text-ink-muted",
              )}
            >
              {vue}
            </span>
          ))}
        </div>
      </div>

      {/* ── Grille mensuelle : six semaines de sept jours ─────────────────── */}
      <div className="flex min-h-0 flex-1 flex-col p-[18px]">
        <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-card border border-line bg-surface">
          <div className="grid flex-none grid-cols-7 border-b border-line bg-surface-alt">
            {JOURS_CALENDRIER.map((jour) => (
              <div
                key={jour}
                className="px-2 py-2 font-mono text-[10px] font-semibold uppercase tracking-[0.07em] text-ink-faint"
              >
                {jour}
              </div>
            ))}
          </div>

          <div className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6">
            {CASES.map((jour, index) => {
              const evenements = jour.dansLeMois
                ? (EVENEMENTS_CALENDRIER[jour.numero] ?? [])
                : [];
              const aujourdhui = jour.dansLeMois && jour.numero === JOUR_COURANT;

              return (
                <div
                  key={index}
                  className={cn(
                    "flex min-h-[46px] min-w-0 flex-col gap-1 overflow-hidden border-r border-b border-line p-[6px] [&:nth-child(7n)]:border-r-0",
                    jour.dansLeMois ? "bg-surface" : "bg-surface-alt",
                  )}
                >
                  <span
                    className={cn(
                      "grid size-6 flex-none place-items-center rounded-pill text-[10.5px] tabular-nums",
                      aujourdhui
                        ? "bg-accent font-medium text-on-accent"
                        : jour.dansLeMois
                          ? "text-ink-muted"
                          : "text-ink-faint",
                    )}
                  >
                    {jour.numero}
                  </span>

                  <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-hidden">
                    {evenements.map((evenement) => (
                      <span
                        key={evenement.libelle}
                        className={cn(
                          "flex w-full items-center gap-1 rounded-pill px-[6px] py-px text-[9.5px]",
                          evenement.classes,
                        )}
                      >
                        <Icon name={evenement.icone} size={11} className="flex-none" />
                        {evenement.heure ? (
                          <span className="flex-none tabular-nums">{evenement.heure}</span>
                        ) : null}
                        <span className="truncate">{evenement.libelle}</span>
                      </span>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
