import type {
  ToFollowUp,
  Metrics,
  Performance,
} from "@/shared/types/generated/analytics";
import { cn } from "@/shared/lib/cn";
import { formatDate } from "./analyticsDates";
import { Button, EmptyState, Skeleton, StatusPill } from "@/shared/ui";
import type { Tone } from "@/shared/ui";

/**
 * Applications sans réponse, avec leur ancienneté et l'action de relance.
 *
 * L'ancienneté passe en rouge à quinze jours : au-delà, la maquette signale la relance comme
 * en retard plutôt que simplement due.
 */
export function FollowUpList({
  items,
  onFollowUp,
}: {
  items: readonly ToFollowUp[];
  onFollowUp: (item: ToFollowUp) => void;
}) {
  if (items.length === 0) {
    return (
      <EmptyState
        icon="task_alt"
        title="Aucune relance nécessaire"
        description="Toutes les candidatures récentes ont été traitées."
      />
    );
  }

  return (
    <ul>
      {items.map((item) => (
        <li
          key={item.id}
          className="flex items-center gap-[11px] border-b border-line px-[19px] py-3 last:border-b-0"
        >
          <span className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-meta font-strong text-ink-muted">
            {initials(item.company_name ?? item.job_title)}
          </span>
          <div className="min-w-0 flex-1">
            <p className="truncate text-body font-mid text-ink">{item.job_title}</p>
            <p className="truncate text-label text-ink-faint">
              {item.company_name ?? "Entreprise"} · envoyée le {formatDate(item.sent_date, "long")}
            </p>
          </div>
          <StatusPill tone={item.days >= 15 ? "danger" : "warning"} compact>
            {item.days} j
          </StatusPill>
          <Button icon="send" className="h-pager px-2.5 text-label" onClick={() => onFollowUp(item)}>
            Relancer
          </Button>
        </li>
      ))}
    </ul>
  );
}

/** Rythme et délais : rangées libellé / valeur séparées par un filet haut. */
export function PerformanceList({
  performance,
  metrics,
}: {
  performance: Performance;
  metrics: Metrics;
}) {
  const rows: readonly { label: string; value: string; tone?: Tone }[] = [
    {
      label: "Délai moyen de réponse",
      value:
        performance.average_response_days === null ? "—" : `${performance.average_response_days} j`,
    },
    {
      label: "Candidatures / semaine",
      value: formatCount(performance.applications_per_week),
    },
    { label: "Taux d’entretien", value: `${metrics.interview_rate} %`, tone: "success" },
    {
      label: "Relances en retard",
      value: performance.overdue_follow_ups.toString(),
      ...(performance.overdue_follow_ups > 0 ? { tone: "warning" } : {}),
    },
  ];

  return (
    <dl>
      {rows.map((row) => (
        <div
          key={row.label}
          className="flex items-center justify-between gap-3.5 border-t border-line py-2.5"
        >
          <dt className="min-w-0 text-note text-ink-muted">{row.label}</dt>
          <dd
            className={cn(
              "tabular flex-none text-body font-semibold",
              row.tone === "success"
                ? "text-success"
                : row.tone === "warning"
                  ? "text-warning"
                  : "text-ink",
            )}
          >
            {row.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}

/** Ossature affichée pendant le chargement, aux dimensions des cartes réelles. */
export function AnalyticsSkeleton() {
  return (
    <div
      role="status"
      aria-label="Chargement en cours"
      className="px-7 pt-[22px] pb-8"
    >
      <div className="mb-4 grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(200px,100%),1fr))]">
        {Array.from({ length: 4 }, (_, index) => (
          <div key={index} className="rounded-card border border-line bg-surface px-[18px] py-4">
            <Skeleton className="h-3 w-24" />
            <Skeleton className="mt-[19px] h-6 w-16" />
          </div>
        ))}
      </div>
      <div className="mb-4 grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(320px,100%),1fr))]">
        <Skeleton className="h-[232px] rounded-card" />
        <Skeleton className="h-[232px] rounded-card" />
      </div>
      <Skeleton className="mb-4 h-[124px] rounded-card" />
      <Skeleton className="h-[300px] rounded-card" />
    </div>
  );
}

/** Initials d'un intitulé, pour les pastilles des listes. */
export function initials(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}

function formatCount(value: number): string {
  return new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 }).format(value);
}
