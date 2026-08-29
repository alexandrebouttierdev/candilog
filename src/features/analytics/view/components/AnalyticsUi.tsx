import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import type {
  ToFollowUp,
  UpcomingItem,
  Step,
  Metrics,
  Performance,
  ActivityWeek,
} from "@/shared/types/generated/analytics";
import type { Application } from "@/shared/types/generated/applications";
import { contract_label, status_meta } from "@/features/applications/model/statuses";
import { cn } from "@/shared/lib/cn";
import {
  Button,
  CellIdentity,
  DataTable,
  EmptyState,
  Skeleton,
  StatusPill,
} from "@/shared/ui";
import type { Column, Tone } from "@/shared/ui";

/** Couleur de remplissage des barres et des pastilles, par tonalité. */
const FILL: Record<Tone, string> = {
  neutral: "bg-ink-faint",
  accent: "bg-accent",
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-danger",
};

const TINT: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Prochains entretiens et relances.
 *
 * Rangées de 11 px séparées par un filet haut, tuile de date de 36 px reprenant la teinte du
 * genre d'événement, pastille de genre à droite : la liste « Prochains événements » de la
 * maquette du tableau de bord.
 */
export function UpcomingList({ upcoming_items }: { upcoming_items: readonly UpcomingItem[] }) {
  if (upcoming_items.length === 0) {
    return (
      <EmptyState
        icon="event_available"
        title="Rien à venir"
        description="Aucun entretien ni relance n’est programmé."
      />
    );
  }

  return (
    <ul className="flex flex-col">
      {upcoming_items.slice(0, 4).map((upcoming_item) => {
        const interview = upcoming_item.kind === "entretien";
        const tone: Tone = interview ? "success" : "warning";
        return (
          <li key={`${upcoming_item.kind}-${upcoming_item.id}`} className="border-t border-line">
            <Link
              to="/tracking/calendar"
              className="flex items-center gap-3 py-[11px] transition-colors duration-hover hover:bg-surface-hover"
            >
              <span
                className={cn(
                  "flex size-9 flex-none flex-col items-center justify-center rounded-tile leading-[1.05]",
                  TINT[tone],
                )}
              >
                <span className="tabular text-item font-strong">{day(upcoming_item.date)}</span>
                <span className="text-[8.5px] font-semibold tracking-[0.04em] uppercase">
                  {month(upcoming_item.date)}
                </span>
              </span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-item font-mid text-ink">
                  {interview ? "Entretien" : "Relance"} — {upcoming_item.company_name ?? "Entreprise"}
                </span>
                <span className="mt-0.5 block truncate text-label text-ink-faint">
                  {upcoming_item.job_title ?? "Candidature"} · {upcoming_item.detail}
                </span>
              </span>
              <StatusPill tone={tone} icon={interview ? "videocam" : "send"}>
                {interview ? "Entretien" : "Relance"}
              </StatusPill>
            </Link>
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Histogramme des candidatures envoyées, semaine par semaine.
 *
 * Barres proportionnelles surmontées de leur valeur et suivies de leur libellé, comme dans
 * les maquettes. La hauteur du bloc de barres est un paramètre : le tableau de bord lui
 * donne 98 px, l'écran Analytics 150 px.
 *
 * Minutes en `div` plutôt qu'en SVG parce que les hauteurs sont exprimées en pourcentage du
 * conteneur : le graphique suit alors la largeur de la carte sans recalcul au
 * redimensionnement, ce qu'un `viewBox` fixe ne permettrait pas.
 */
export function ActivityChart({
  activity,
  height = 98,
  gap = 8,
  showCounts = true,
  shortLabels = false,
}: {
  activity: readonly ActivityWeek[];
  height?: number;
  gap?: number;
  showCounts?: boolean;
  shortLabels?: boolean;
}) {
  if (activity.every((week) => week.count === 0)) {
    return (
      <EmptyState
        icon="bar_chart"
        title="Pas encore d’activité"
        description="Les candidatures envoyées apparaîtront ici semaine après semaine."
      />
    );
  }

  const maximum = Math.max(...activity.map((week) => week.count), 1);

  return (
    <>
      <div
        role="img"
        aria-label="Candidatures envoyées par semaine"
        style={{ height, gap }}
        className="mb-[9px] flex items-end"
      >
        {activity.map((week) => (
          <div
            key={week.start}
            className="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-1.5"
          >
            {showCounts ? (
              <span className="font-mono tabular text-meta text-ink">{week.count}</span>
            ) : null}
            <div
              style={{ height: `${(week.count / maximum) * 100}%` }}
              className={cn(
                "min-h-1 w-full rounded-chip",
                week.count === 0 ? "bg-neutral-tint" : "bg-accent",
              )}
            />
          </div>
        ))}
      </div>
      <div style={{ gap }} className="flex" aria-hidden="true">
        {activity.map((week) => (
          <span
            key={week.start}
            className="min-w-0 flex-1 truncate text-center text-eyebrow font-normal tracking-normal text-ink-faint"
          >
            {formatDate(week.start, shortLabels ? "numeric" : "court")}
          </span>
        ))}
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

/** Tonalités de l'entonnoir et du pipeline, dans l'ordre des étapes renvoyées par le backend. */
const STEPS_PIPELINE: readonly Tone[] = ["neutral", "warning", "success", "danger"];
const STEPS_FUNNEL: readonly Tone[] = ["accent", "accent", "success", "danger"];

/**
 * Funnel de conversion : une barre par étape, valeur et part à droite du libellé.
 */
export function FunnelChart({ steps }: { steps: readonly Step[] }) {
  if (steps.every((step) => step.count === 0)) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Entonnoir vide"
        description="Il se construira dès la première candidature."
      />
    );
  }

  return (
    <div>
      {steps.map((step, index) => (
        <div key={step.label} className="mb-3.5 last:mb-0">
          <div className="mb-1.5 flex items-baseline justify-between gap-3">
            <span className="text-note font-medium text-ink-muted">{step.label}</span>
            <span className="tabular text-note font-semibold text-ink">
              {step.count}{" "}
              <span className="font-medium text-ink-faint">· {step.percentage} %</span>
            </span>
          </div>
          <div className="h-2 overflow-hidden rounded-tag bg-neutral-tint">
            <div
              style={{ width: `${step.percentage}%` }}
              className={cn("h-full rounded-tag", FILL[STEPS_FUNNEL[index] ?? "neutral"])}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

/**
 * Répartition du pipeline : bande segmentée puis légende chiffrée.
 *
 * Les segments sont dimensionnés par `flex-grow` sur le nombre de candidatures, exactement
 * comme la maquette : une étape vide disparaît d'elle-même au lieu de laisser un filet.
 */
export function PipelineBar({ steps }: { steps: readonly Step[] }) {
  const total = steps.reduce((somme, step) => somme + step.count, 0);

  if (total === 0) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Pipeline vide"
        description="Les statuts de vos candidatures formeront cette répartition."
      />
    );
  }

  return (
    <div>
      <div className="mb-[13px] flex h-[9px] gap-[3px]">
        {steps.map((step, index) => (
          <span
            key={step.label}
            style={{ flexGrow: step.count, flexBasis: 0 }}
            className={cn("rounded-[3px]", FILL[STEPS_PIPELINE[index] ?? "neutral"])}
          />
        ))}
      </div>
      <div className="flex flex-wrap gap-x-[22px] gap-y-2">
        {steps.map((step, index) => (
          <div key={step.label} className="flex items-center gap-[7px]">
            <span
              className={cn(
                "size-1.5 rounded-full",
                FILL[STEPS_PIPELINE[index] ?? "neutral"],
              )}
            />
            <span className="text-note text-ink-muted">{step.label}</span>
            <span className="tabular text-note font-semibold text-ink">{step.count}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

/** Columns du tableau des candidatures récentes, aux proportions de la maquette. */
function columnsRecent(): readonly Column<Application>[] {
  return [
    {
      key: "poste",
      header: "Poste",
      grow: 2.1,
      render: (application) => (
        <CellIdentity
          initials={initials(application.company_name ?? application.job_title)}
          title={application.job_title}
          subtitle={`${contract_label(application.contract_type)}${
            application.company_city ? ` · ${application.company_city}` : ""
          }`}
        />
      ),
    },
    {
      key: "entreprise",
      header: "Entreprise",
      grow: 1.4,
      render: (application) => (
        <span className="truncate text-body text-ink-muted">
          {application.company_name ?? "—"}
        </span>
      ),
    },
    {
      key: "statut",
      header: "Statut",
      grow: 1.1,
      render: (application) => {
        const status = status_meta(application.status);
        return (
          <StatusPill tone={status.tone} icon={status.icon}>
            {status.label}
          </StatusPill>
        );
      },
    },
    {
      key: "maj",
      header: "Mise à jour",
      grow: 0.9,
      numeric: true,
      render: (application) => (
        <span className="text-note text-ink-faint">
          {formatDate(application.updated_at, "long")}
        </span>
      ),
    },
  ];
}

/** Table des dernières candidatures, en pied du tableau de bord. */
export function RecentApplications({
  applications,
  header,
  onOuvrir,
}: {
  applications: readonly Application[];
  header: ReactNode;
  onOuvrir: (application: Application) => void;
}) {
  return (
    <DataTable
      columns={columnsRecent()}
      rows={applications}
      row_key={(application) => application.id}
      onRowClick={onOuvrir}
      header={header}
      empty_state={
        <EmptyState
          icon="work"
          title="Aucune candidature"
          description="Ajoutez votre première candidature pour démarrer le suivi."
        />
      }
    />
  );
}

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

function formatDate(value: string, format: "court" | "long" | "numeric"): string {
  const date = new Date(`${value.slice(0, 10)}T12:00:00`);
  if (Number.isNaN(date.getTime())) return value.slice(0, 10);
  if (format === "numeric") {
    return new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "2-digit" }).format(date);
  }
  return new Intl.DateTimeFormat("fr-FR", {
    day: format === "long" ? "2-digit" : "numeric",
    month: "short",
  }).format(date);
}

function day(value: string): string {
  return value.slice(8, 10);
}

function month(value: string): string {
  const date = new Date(`${value.slice(0, 10)}T12:00:00`);
  return Number.isNaN(date.getTime())
    ? ""
    : new Intl.DateTimeFormat("fr-FR", { month: "short" }).format(date).replace(".", "");
}

function formatCount(value: number): string {
  return new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 }).format(value);
}
