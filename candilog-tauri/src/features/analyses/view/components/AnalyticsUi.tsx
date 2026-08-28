import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import type {
  ARelancer,
  Echeance,
  Etape,
  Indicateurs,
  Performance,
  SemaineActivite,
} from "@/shared/types/generated/analyses";
import type { Candidature } from "@/shared/types/generated/candidatures";
import { contratLabel, statutMeta } from "@/features/candidatures/model/statuts";
import { cn } from "@/shared/lib/cn";
import { Button, EmptyState, Icon, Skeleton } from "@/shared/ui";

type Ton = "accent" | "success" | "warning" | "danger" | "neutral";

const TONS: Record<Ton, { icon: string; text: string; tint: string; bar: string }> = {
  accent: {
    icon: "bg-accent-tint text-accent",
    text: "text-accent",
    tint: "border-accent-border bg-accent-tint/55",
    bar: "bg-accent",
  },
  success: {
    icon: "bg-success-tint text-success",
    text: "text-success",
    tint: "border-success/20 bg-success-tint/50",
    bar: "bg-success",
  },
  warning: {
    icon: "bg-warning-tint text-warning",
    text: "text-warning",
    tint: "border-warning/20 bg-warning-tint/50",
    bar: "bg-warning",
  },
  danger: {
    icon: "bg-danger-tint text-danger",
    text: "text-danger",
    tint: "border-danger/20 bg-danger-tint/50",
    bar: "bg-danger",
  },
  neutral: {
    icon: "bg-neutral-tint text-ink-muted",
    text: "text-ink",
    tint: "border-line bg-surface",
    bar: "bg-ink-faint",
  },
};

export function AnalyticsPanel({
  icon,
  title,
  meta,
  className,
  children,
}: {
  icon: string;
  title: string;
  meta?: ReactNode;
  className?: string;
  children: ReactNode;
}) {
  return (
    <section className={cn("min-w-0 rounded-card border border-line bg-surface shadow-e1", className)}>
      <header className="flex min-h-12 items-center gap-2 border-b border-line px-4">
        <Icon name={icon} size={17} className="text-ink-faint" />
        <h2 className="min-w-0 flex-1 truncate text-section text-ink">{title}</h2>
        {meta ? <div className="flex-none text-meta text-ink-faint">{meta}</div> : null}
      </header>
      {children}
    </section>
  );
}

export function MetricCard({
  icon,
  label,
  value,
  context,
  tone = "neutral",
  primary = false,
  sparkline,
}: {
  icon: string;
  label: string;
  value: string;
  context: string;
  tone?: Ton;
  primary?: boolean;
  sparkline?: readonly number[];
}) {
  const styles = TONS[tone];
  return (
    <article
      className={cn(
        "relative min-w-0 overflow-hidden rounded-card border p-4 shadow-e1",
        primary ? styles.tint : "border-line bg-surface",
      )}
    >
      <div className="flex items-center gap-2">
        <span className={cn("flex size-7 items-center justify-center rounded-button", styles.icon)}>
          <Icon name={icon} size={16} />
        </span>
        <p className="min-w-0 flex-1 truncate text-meta font-medium text-ink-muted">{label}</p>
      </div>
      <div className="mt-2 flex items-end justify-between gap-3">
        <div className="min-w-0">
          <p
            className={cn(
              "tabular font-semibold tracking-[-0.025em]",
              primary ? "text-[34px] leading-none" : "text-kpi",
              primary ? styles.text : "text-ink",
            )}
          >
            {value}
          </p>
          <p className="mt-1 truncate text-meta text-ink-faint">{context}</p>
        </div>
        {sparkline && sparkline.length > 1 ? (
          <Sparkline values={sparkline} label={`Tendance de ${label.toLowerCase()}`} />
        ) : null}
      </div>
    </article>
  );
}

function Sparkline({ values, label }: { values: readonly number[]; label: string }) {
  const maximum = Math.max(...values, 1);
  const pas = 112 / Math.max(values.length - 1, 1);
  const points = values
    .map((valeur, index) => `${4 + index * pas},${28 - (valeur / maximum) * 22}`)
    .join(" ");
  const aire = `4,30 ${points} 116,30`;
  return (
    <svg role="img" aria-label={label} viewBox="0 0 120 32" className="h-8 w-28 flex-none">
      <polygon points={aire} className="fill-accent/10" />
      <polyline
        points={points}
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        className="text-accent"
      />
    </svg>
  );
}

export function ActivityChart({ activite }: { activite: readonly SemaineActivite[] }) {
  if (activite.every((semaine) => semaine.nombre === 0)) {
    return (
      <EmptyState
        icon="bar_chart"
        title="Pas encore d’activité"
        description="Les candidatures envoyées apparaîtront ici semaine après semaine."
      />
    );
  }
  const maximum = Math.max(...activite.map((semaine) => semaine.nombre), 1);
  const pasLibelle = activite.length > 16 ? Math.ceil(activite.length / 8) : 1;
  return (
    <div className="p-4">
      <div
        role="img"
        aria-label="Candidatures envoyées par semaine"
        className="flex h-36 items-end gap-1.5 border-b border-line px-1"
      >
        {activite.map((semaine) => {
          const hauteur = semaine.nombre === 0 ? 3 : Math.max(8, (semaine.nombre / maximum) * 112);
          return (
            <div key={semaine.debut} className="group flex h-full min-w-0 flex-1 items-end justify-center">
              <div
                title={`${formatDate(semaine.debut, "long")} : ${semaine.nombre}`}
                style={{ height: `${hauteur}px` }}
                className={cn(
                  "relative w-full max-w-8 rounded-t-[5px]",
                  semaine.nombre === 0 ? "bg-neutral-tint" : "bg-accent",
                )}
              >
                <span className="tabular absolute -top-5 left-1/2 -translate-x-1/2 text-[10px] text-ink-faint opacity-0 transition-opacity duration-150 group-hover:opacity-100">
                  {semaine.nombre}
                </span>
              </div>
            </div>
          );
        })}
      </div>
      <div className="mt-2 flex gap-1.5 px-1" aria-hidden="true">
        {activite.map((semaine, index) => (
          <span key={semaine.debut} className="min-w-0 flex-1 truncate text-center text-[10px] text-ink-faint">
            {index % pasLibelle === 0 || index === activite.length - 1
              ? formatDate(semaine.debut, "court")
              : ""}
          </span>
        ))}
      </div>
      <ol className="sr-only">
        {activite.map((semaine) => (
          <li key={semaine.debut}>
            Semaine du {formatDate(semaine.debut, "long")} : {semaine.nombre} candidature
            {semaine.nombre > 1 ? "s" : ""}
          </li>
        ))}
      </ol>
    </div>
  );
}

export function FunnelChart({ etapes }: { etapes: readonly Etape[] }) {
  if (etapes.every((etape) => etape.nombre === 0)) {
    return (
      <EmptyState
        icon="conversion_path"
        title="Entonnoir vide"
        description="Il se construira dès la première candidature."
      />
    );
  }
  const tons: Ton[] = ["accent", "accent", "success", "danger"];
  return (
    <div className="space-y-3.5 p-4">
      {etapes.map((etape, index) => (
        <div key={etape.label}>
          <div className="mb-1.5 flex items-baseline justify-between gap-3">
            <span className="text-meta font-medium text-ink-muted">{etape.label}</span>
            <span className="tabular text-meta font-semibold text-ink">
              {etape.nombre} <span className="font-normal text-ink-faint">· {etape.pourcentage} %</span>
            </span>
          </div>
          <div className="h-2 overflow-hidden rounded-full bg-neutral-tint">
            <div
              style={{ width: `${etape.pourcentage}%` }}
              className={cn("h-full rounded-full", TONS[tons[index] ?? "neutral"].bar)}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

export function PipelineBar({ etapes }: { etapes: readonly Etape[] }) {
  const total = etapes.reduce((somme, etape) => somme + etape.nombre, 0);
  const tons: Ton[] = ["neutral", "warning", "success", "danger"];
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
    <div className="p-4">
      <div className="flex h-2.5 gap-0.5 overflow-hidden rounded-[5px] bg-neutral-tint">
        {etapes.map((etape, index) => (
          <span
            key={etape.label}
            style={{ flexGrow: etape.nombre, flexBasis: 0 }}
            className={TONS[tons[index] ?? "neutral"].bar}
          />
        ))}
      </div>
      <div className="mt-3 flex flex-wrap gap-x-6 gap-y-2">
        {etapes.map((etape, index) => (
          <div key={etape.label} className="flex items-center gap-1.5 text-meta">
            <span className={cn("size-1.5 rounded-full", TONS[tons[index] ?? "neutral"].bar)} />
            <span className="text-ink-muted">{etape.label}</span>
            <span className="tabular font-semibold text-ink">{etape.nombre}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function UpcomingList({ echeances }: { echeances: readonly Echeance[] }) {
  if (echeances.length === 0) {
    return (
      <EmptyState
        icon="event_available"
        title="Rien à venir"
        description="Aucun entretien ni relance n’est programmé."
      />
    );
  }
  return (
    <ul className="divide-y divide-line">
      {echeances.slice(0, 4).map((echeance) => {
        const entretien = echeance.genre === "entretien";
        return (
          <li key={`${echeance.genre}-${echeance.id}`}>
            <Link
              to="/suivi/calendrier"
              className="flex min-h-14 items-center gap-3 px-4 py-2.5 transition-colors duration-150 hover:bg-neutral-tint"
            >
              <span
                className={cn(
                  "flex size-9 flex-none flex-col items-center justify-center rounded-card leading-none",
                  entretien ? TONS.success.icon : TONS.warning.icon,
                )}
              >
                <span className="tabular text-body font-semibold">{jour(echeance.date)}</span>
                <span className="mt-0.5 text-[8px] font-semibold">{mois(echeance.date)}</span>
              </span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-body font-medium text-ink">
                  {entretien ? "Entretien" : "Relance"} — {echeance.entrepriseNom ?? "Entreprise"}
                </span>
                <span className="mt-0.5 block truncate text-meta text-ink-faint">
                  {echeance.poste ?? "Candidature"} · {echeance.detail}
                </span>
              </span>
              <span className={cn("flex items-center gap-1 text-meta font-medium", entretien ? "text-success" : "text-warning")}>
                <Icon name={entretien ? "videocam" : "send"} size={14} />
                {entretien ? "Entretien" : "Relance"}
              </span>
            </Link>
          </li>
        );
      })}
    </ul>
  );
}

export function RecentApplications({ candidatures }: { candidatures: readonly Candidature[] }) {
  if (candidatures.length === 0) {
    return (
      <EmptyState
        icon="work"
        title="Aucune candidature"
        description="Ajoutez votre première candidature pour démarrer le suivi."
      />
    );
  }
  return (
    <div className="overflow-x-auto [scrollbar-gutter:stable]">
      <table className="w-full min-w-[680px] border-collapse">
        <thead className="bg-neutral-tint text-left text-meta font-medium text-ink-faint">
          <tr>
            <th className="px-4 py-2">Poste</th>
            <th className="px-4 py-2">Entreprise</th>
            <th className="px-4 py-2">Statut</th>
            <th className="px-4 py-2 text-right">Mise à jour</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-line">
          {candidatures.map((candidature) => {
            const statut = statutMeta(candidature.statut);
            return (
              <tr key={candidature.id} className="h-row transition-colors duration-150 hover:bg-neutral-tint">
                <td className="px-4">
                  <Link to="/suivi/candidatures" className="flex items-center gap-2.5">
                    <span className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-meta font-semibold text-ink-muted">
                      {initiales(candidature.entrepriseNom ?? candidature.poste)}
                    </span>
                    <span className="min-w-0">
                      <span className="block truncate text-body font-medium text-ink">{candidature.poste}</span>
                      <span className="block text-meta text-ink-faint">
                        {contratLabel(candidature.typeContrat)}
                        {candidature.entrepriseVille ? ` · ${candidature.entrepriseVille}` : ""}
                      </span>
                    </span>
                  </Link>
                </td>
                <td className="px-4 text-body text-ink-muted">{candidature.entrepriseNom ?? "—"}</td>
                <td className="px-4">
                  <span className="inline-flex items-center gap-1.5 text-meta text-ink-muted">
                    <span className={cn("size-1.5 rounded-full", TONS[statut.tone].bar)} />
                    {statut.label}
                  </span>
                </td>
                <td className="tabular px-4 text-right text-meta text-ink-faint">
                  {formatDate(candidature.updatedAt, "long")}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export function FollowUpList({
  items,
  onRelancer,
}: {
  items: readonly ARelancer[];
  onRelancer: (item: ARelancer) => void;
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
    <ul className="divide-y divide-line">
      {items.map((item) => (
        <li key={item.id} className="flex min-h-14 items-center gap-3 px-4 py-2.5">
          <span className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-meta font-semibold text-ink-muted">
            {initiales(item.entrepriseNom ?? item.poste)}
          </span>
          <div className="min-w-0 flex-1">
            <p className="truncate text-body font-medium text-ink">{item.poste}</p>
            <p className="truncate text-meta text-ink-faint">
              {item.entrepriseNom ?? "Entreprise"} · envoyée le {formatDate(item.dateEnvoi, "long")}
            </p>
          </div>
          <span className={cn("tabular rounded-pill px-2 py-1 text-meta font-medium", item.jours >= 15 ? TONS.danger.icon : TONS.warning.icon)}>
            {item.jours} j
          </span>
          <Button icon="send" onClick={() => onRelancer(item)}>
            Relancer
          </Button>
        </li>
      ))}
    </ul>
  );
}

export function PerformanceList({
  performance,
  indicateurs,
}: {
  performance: Performance;
  indicateurs: Indicateurs;
}) {
  const lignes = [
    ["Délai moyen de réponse", performance.delaiMoyenReponse === null ? "—" : `${performance.delaiMoyenReponse} j`],
    ["Candidatures / semaine", formatNombre(performance.candidaturesParSemaine)],
    ["Taux d’entretien", `${indicateurs.tauxEntretien} %`],
    ["Relances en retard", performance.relancesEnRetard.toString()],
  ] as const;
  return (
    <dl className="divide-y divide-line px-4">
      {lignes.map(([label, valeur], index) => (
        <div key={label} className="flex min-h-11 items-center justify-between gap-4">
          <dt className="text-meta text-ink-muted">{label}</dt>
          <dd className={cn("tabular text-body font-semibold", index === 2 ? "text-success" : index === 3 && performance.relancesEnRetard > 0 ? "text-warning" : "text-ink")}>
            {valeur}
          </dd>
        </div>
      ))}
    </dl>
  );
}

export function AnalyticsSkeleton() {
  return (
    <div role="status" aria-label="Chargement du tableau de bord" className="space-y-4 p-6">
      <div className="grid grid-cols-4 gap-3">
        {Array.from({ length: 4 }, (_, index) => (
          <div key={index} className="h-28 rounded-card border border-line bg-surface p-4">
            <Skeleton className="h-3 w-24" />
            <Skeleton className="mt-4 h-8 w-16" />
            <Skeleton className="mt-2 h-2.5 w-28" />
          </div>
        ))}
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Skeleton className="h-64 rounded-card" />
        <Skeleton className="h-64 rounded-card" />
      </div>
      <Skeleton className="h-48 rounded-card" />
    </div>
  );
}

function initiales(valeur: string): string {
  return valeur
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}

function formatDate(valeur: string, format: "court" | "long"): string {
  const date = new Date(`${valeur.slice(0, 10)}T12:00:00`);
  if (Number.isNaN(date.getTime())) return valeur.slice(0, 10);
  return new Intl.DateTimeFormat("fr-FR", {
    day: format === "long" ? "2-digit" : undefined,
    month: "short",
  }).format(date);
}

function jour(valeur: string): string {
  return valeur.slice(8, 10);
}

function mois(valeur: string): string {
  const date = new Date(`${valeur.slice(0, 10)}T12:00:00`);
  return Number.isNaN(date.getTime())
    ? ""
    : new Intl.DateTimeFormat("fr-FR", { month: "short" }).format(date).replace(".", "");
}

function formatNombre(valeur: number): string {
  return new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 }).format(valeur);
}
