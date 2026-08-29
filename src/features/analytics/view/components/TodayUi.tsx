import { Link } from "react-router-dom";
import type { ActivityWeek, Dashboard, UpcomingItem } from "@/shared/types/generated/analytics";
import type { Application } from "@/shared/types/generated/applications";
import { status_meta } from "@/features/applications/model/statuses";
import { dayOf, timeFromTimestamp } from "@/shared/lib/dates";
import { cn } from "@/shared/lib/cn";
import { ActivityChart } from "./AnalyticsUi";
import { Button, InspectorSectionLabel, Skeleton, StatusPill } from "@/shared/ui";

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

function isoDay(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function todayIso(now = new Date()): string {
  return isoDay(now);
}

function tomorrowIso(now = new Date()): string {
  const next = new Date(now);
  next.setDate(next.getDate() + 1);
  return isoDay(next);
}

function eventDay(value: string): string {
  return dayOf(value);
}

/** Heure `HH:MM` extraite du timestamp, sans passer par `Intl` (évite « 14 h 30 »). */
export function formatEventTime(value: string): string | null {
  if (!value.includes("T")) return null;
  const sliced = timeFromTimestamp(value);
  return /^\d{2}:\d{2}$/.test(sliced) ? sliced : null;
}

/** Libellé court pour une gouttière d'heure : `14:30`, `Auj.`, `Dem.`, `02/09`. */
export function formatWhenShort(value: string, now = new Date()): string {
  const time = formatEventTime(value);
  if (time) return time;
  const day = eventDay(value);
  if (day === todayIso(now)) return "Auj.";
  if (day === tomorrowIso(now)) return "Dem.";
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(day);
  return match ? `${match[3]}/${match[2]}` : day;
}

function isInterview(item: UpcomingItem): boolean {
  return item.kind === "entretien";
}

function initials(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}

export function isTodayEmpty(data: Dashboard): boolean {
  return (
    data.upcoming_items.length === 0 &&
    data.recent.length === 0 &&
    data.performance.overdue_follow_ups === 0 &&
    data.activity.every((week) => week.count === 0)
  );
}

/** Compteurs du mois : kicker, chiffres mono, filets. Pas de cartes. */
export function TodayStats({
  applications,
  responses,
  interviews,
  overdue,
}: {
  applications: number;
  responses: number;
  interviews: number;
  overdue: number;
}) {
  const items = [
    { label: "Candidatures", value: applications },
    { label: "Réponses", value: responses },
    { label: "Entretiens", value: interviews },
  ];

  return (
    <div className="mb-6 border-b border-line-soft pb-4">
      <p className="mb-3 text-eyebrow uppercase text-ink-label">30 derniers jours</p>
      <dl className="flex flex-wrap items-start gap-y-3">
        {items.map((item, index) => (
          <div
            key={item.label}
            className={cn(
              "flex min-w-[7.5rem] flex-col pr-7",
              index > 0 && "border-l border-field pl-7",
            )}
          >
            <dt className="order-2 mt-1.5 text-eyebrow uppercase text-ink-label">{item.label}</dt>
            <dd className="order-1 font-mono tabular text-heading tracking-tight text-ink">
              {item.value}
            </dd>
          </div>
        ))}
        {overdue > 0 ? (
          <div className="ml-auto flex min-w-[7.5rem] flex-col border-l border-field pl-7">
            <dt className="order-2 mt-1.5 text-eyebrow uppercase text-warning">À relancer</dt>
            <dd className="order-1 font-mono tabular text-heading tracking-tight text-warning">
              {overdue}
            </dd>
          </div>
        ) : null}
      </dl>
    </div>
  );
}

/** Briefing du prochain entretien : heure en display, barre d'accent, identité. */
export function NextEvent({
  item,
  href,
}: {
  item: UpcomingItem;
  href: string;
}) {
  const interview = isInterview(item);
  const time = formatEventTime(item.date);
  const day = eventDay(item.date);
  const now = new Date();
  const relative =
    day === todayIso(now) ? "aujourd'hui" : day === tomorrowIso(now) ? "demain" : null;

  return (
    <Link
      to={href}
      className={cn(
        "mb-1 flex items-start gap-4 rounded-none py-3 pr-3 pl-3.5 -mx-1",
        "transition-colors duration-hover hover:bg-surface-hover",
        interview ? "row-selected" : "bg-warning-tint shadow-[inset_2px_0_0_var(--color-warning)]",
      )}
    >
      <div className="flex-none">
        <p className="tabular text-display whitespace-nowrap text-ink">{formatWhenShort(item.date)}</p>
        {time && relative ? (
          <p className="mt-2 text-meta text-ink-faint">{relative}</p>
        ) : null}
      </div>
      <div className="min-w-0 flex-1 pt-0.5">
        <p className="truncate text-eyebrow uppercase text-ink-label">
          {interview ? "Entretien" : "Relance"}
          {item.detail ? ` · ${item.detail}` : ""}
        </p>
        <p className="mt-1.5 truncate text-title leading-snug text-ink">
          {item.company_name ?? "Entreprise"}
        </p>
        <p className="mt-1 truncate text-note leading-snug text-ink-faint">
          {item.job_title ?? "Candidature"}
        </p>
      </div>
    </Link>
  );
}

function IdentityLine({ name, detail }: { name: string; detail: string }) {
  return (
    <span className="block min-w-0 truncate leading-snug">
      <span className="text-body font-medium text-ink">{name}</span>
      <span className="text-ink-disabled"> · </span>
      <span className="text-note text-ink-faint">{detail}</span>
    </span>
  );
}

/** Liste d'échéances, lignes 40 px, heure à gauche. */
export function UpcomingRows({
  items,
  hrefFor,
}: {
  items: readonly UpcomingItem[];
  hrefFor: (item: UpcomingItem) => string;
}) {
  if (items.length === 0) return null;

  return (
    <ul className="mt-1 flex flex-col">
      {items.map((item) => {
        const interview = isInterview(item);
        return (
          <li key={`${item.kind}-${item.id}`} className="border-t border-field">
            <Link
              to={hrefFor(item)}
              className="grid h-10 grid-cols-[3.25rem_minmax(0,1fr)_auto] items-center gap-3 transition-colors duration-hover hover:bg-surface-hover"
            >
              <span className="tabular overflow-hidden font-mono text-meta whitespace-nowrap text-ink-disabled">
                {formatWhenShort(item.date)}
              </span>
              <IdentityLine
                name={item.company_name ?? "Entreprise"}
                detail={item.job_title ?? "Candidature"}
              />
              <span className={cn("flex-none text-meta", interview ? "text-accent-text" : "text-warning")}>
                {interview ? "Entretien" : "Relance"}
              </span>
            </Link>
          </li>
        );
      })}
    </ul>
  );
}

export function UpcomingEmpty() {
  return (
    <p className="py-3 text-note leading-relaxed text-ink-faint">
      Aucun entretien ni relance n'est programmé.
    </p>
  );
}

export function TodoRows({
  overdue,
  nextInterview,
  onOpenApplications,
  onOpenCalendar,
}: {
  overdue: number;
  nextInterview: UpcomingItem | null;
  onOpenApplications: () => void;
  onOpenCalendar: () => void;
}) {
  const rows: { key: string; label: string; onClick: () => void; warn?: boolean }[] = [];

  if (overdue > 0) {
    rows.push({
      key: "relances",
      label: `Relancer ${overdue} candidature${overdue > 1 ? "s" : ""} en retard`,
      onClick: onOpenApplications,
      warn: true,
    });
  }

  if (nextInterview && isInterview(nextInterview) && eventDay(nextInterview.date) === todayIso()) {
    rows.push({
      key: "preparer",
      label: `Préparer l'entretien chez ${nextInterview.company_name ?? "cette entreprise"}`,
      onClick: onOpenCalendar,
    });
  }

  if (rows.length === 0) return null;

  return (
    <section className="mt-6">
      <InspectorSectionLabel>À faire</InspectorSectionLabel>
      <ul className="flex flex-col">
        {rows.map((row) => (
          <li key={row.key} className="border-t border-field first:border-t-0">
            <button
              type="button"
              onClick={row.onClick}
              className="flex h-10 w-full items-center gap-2.5 text-left transition-colors duration-hover hover:bg-surface-hover"
            >
              <span
                className={cn(
                  "flex size-[13px] flex-none rounded-[3px] border",
                  row.warn ? "border-warning" : "border-control",
                )}
              />
              <span
                className={cn(
                  "min-w-0 truncate text-body leading-snug",
                  row.warn ? "text-warning" : "text-ink-strong",
                )}
              >
                {row.label}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function RecentRows({
  applications,
  onOpen,
}: {
  applications: readonly Application[];
  onOpen: (id: string) => void;
}) {
  if (applications.length === 0) {
    return (
      <p className="py-3 text-note leading-relaxed text-ink-faint">Aucune candidature récente.</p>
    );
  }

  return (
    <ul className="flex flex-col">
      {applications.map((app) => {
        const status = status_meta(app.status);
        const name = app.company_name ?? "Entreprise";
        return (
          <li key={app.id} className="border-t border-field first:border-t-0">
            <button
              type="button"
              onClick={() => onOpen(app.id)}
              className="flex h-10 w-full min-w-0 items-center gap-2.5 text-left transition-colors duration-hover hover:bg-surface-hover"
            >
              <span className="flex size-7 flex-none items-center justify-center rounded-button bg-fill text-meta font-semibold text-ink-muted">
                {initials(name)}
              </span>
              <span className="min-w-0 flex-1">
                <IdentityLine name={name} detail={app.job_title} />
              </span>
              <StatusPill tone={status.tone} compact className="flex-none">
                {status.label}
              </StatusPill>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

export function TodayActivity({ activity }: { activity: readonly ActivityWeek[] }) {
  if (activity.every((week) => week.count === 0)) return null;

  return (
    <section className="mt-6">
      <InspectorSectionLabel>Activité</InspectorSectionLabel>
      <ActivityChart activity={activity} height={56} gap={5} showCounts={false} shortLabels />
    </section>
  );
}

/** Bureau vide : même geste que le briefing, sans horloge factice. */
export function TodayEmpty({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="flex min-h-0 flex-1 items-center px-[18px] pb-16">
      <div className="max-w-[22rem] border-l-2 border-accent pl-4">
        <p className="text-eyebrow uppercase text-ink-label">Aujourd'hui</p>
        <p className="mt-2 text-title text-ink">Rien de prévu</p>
        <p className="mt-1.5 text-note leading-relaxed text-ink-faint">
          Le prochain entretien s'affichera ici, avec l'heure en grand. Commence par une
          candidature.
        </p>
        <div className="mt-4">
          <Button variant="primary" icon="add" onClick={onCreate}>
            Nouvelle candidature
          </Button>
        </div>
      </div>
    </div>
  );
}

export function TodaySkeleton() {
  return (
    <div className="px-[18px] pt-4 pb-[22px]" role="status" aria-label="Chargement de l'écran">
      <div className="mb-6 border-b border-line-soft pb-4">
        <Skeleton className="mb-3 h-2.5 w-24" />
        <div className="flex gap-7">
          {Array.from({ length: 3 }, (_, index) => (
            <div key={index} className={cn(index > 0 && "border-l border-field pl-7")}>
              <Skeleton className="h-5 w-8" />
              <Skeleton className="mt-1.5 h-2.5 w-16" />
            </div>
          ))}
        </div>
      </div>
      <div className="grid gap-8 min-[1280px]:grid-cols-2">
        <div>
          <Skeleton className="mb-3 h-2.5 w-24" />
          <Skeleton className="h-[72px] w-full" />
          <Skeleton className="mt-3 h-10 w-full" />
          <Skeleton className="mt-1 h-10 w-full" />
        </div>
        <div>
          <Skeleton className="mb-3 h-2.5 w-32" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="mt-1 h-10 w-full" />
          <Skeleton className="mt-1 h-10 w-full" />
        </div>
      </div>
    </div>
  );
}

/** L'entretien le plus proche devient le briefing ; le reste reste en liste. */
export function splitUpcoming(items: readonly UpcomingItem[]): {
  next: UpcomingItem | null;
  rest: UpcomingItem[];
} {
  const next = items.find((item) => item.kind === "entretien") ?? items[0] ?? null;
  if (next === null) return { next: null, rest: [] };
  return { next, rest: items.filter((item) => item !== next) };
}
