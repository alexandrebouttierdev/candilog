import { useState } from "react";
import type { ReactNode } from "react";
import type { Analytics, ToFollowUp, Period } from "@/shared/types/generated/analytics";
import { useAnalyticsViewModel } from "../../viewmodel/useAnalyticsViewModel";
import { AnalyticsSkeleton, FollowUpList, PerformanceList } from "../components/AnalyticsUi";
import { ActivityChart } from "../components/charts";
import { insightsOf } from "../../model/insights";
import { channelLabel } from "@/features/applications";
import { FollowUpFormModal } from "@/features/followups";
import { AppError } from "@/shared/types/app-error";
import { useChrome } from "@/shared/lib/chrome";
import { cn } from "@/shared/lib/cn";
import { Button, ErrorBanner, LineIcon, SegmentedControl, StatusGlyph } from "@/shared/ui";
import type { GlyphTone } from "@/shared/ui";

const PERIODES: readonly { value: Period; label: string }[] = [
  { value: "trente_days", label: "30 j" },
  { value: "quatre_vingt_dix_days", label: "90 j" },
  { value: "tout", label: "Tout" },
];

const PERIOD_LABEL: Record<Period, string> = {
  trente_days: "30 derniers jours",
  quatre_vingt_dix_days: "90 derniers jours",
  tout: "tout l'historique",
};

/** Bloc de l'écran : fond groupé, titre capitalisé. */
function Block({ title, aside, children, className }: { title: string; aside?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <section aria-label={title} className={cn("rounded-r9 bg-group p-3.5", className)}>
      <div className="mb-3 flex items-center gap-3">
        <h2 className="caps">{title}</h2>
        {aside ? <span className="ml-auto flex items-center gap-3">{aside}</span> : null}
      </div>
      {children}
    </section>
  );
}

function Kpi({ label, value, unit, note }: { label: string; value: string; unit?: string; note: string }) {
  return (
    <div className="rounded-r9 bg-group px-3.5 py-3">
      <p className="text-sub text-tx-4">{label}</p>
      <p className="mt-1.5 flex items-baseline gap-1.5">
        <span className="serif-title text-[24px] leading-none text-tx">{value}</span>
        {unit ? <span className="text-small text-tx-4">{unit}</span> : null}
      </p>
      <p className="mt-2 text-tiny text-tx-5">{note}</p>
    </div>
  );
}

/** Parcours des candidatures : chaque étape, sa part et ce qui s'est perdu en route. */
function Journey({ data }: { data: Analytics }) {
  const { metrics } = data;
  const steps: Array<{ label: string; count: number; glyph: GlyphTone; bar: string; note: string | null }> = [
    { label: "Envoyées", count: metrics.applications, glyph: "n", bar: "bg-ac", note: null },
    {
      label: "Réponses reçues",
      count: metrics.responses,
      glyph: "a",
      bar: "bg-st-a",
      note: `${metrics.applications - metrics.responses} sans réponse sur la période.`,
    },
    {
      label: "Entretiens",
      count: metrics.interviews,
      glyph: "g",
      bar: "bg-st-g",
      note: metrics.responses > 0 ? `${metrics.interviews} réponse${metrics.interviews > 1 ? "s" : ""} sur ${metrics.responses} ont mené à un entretien.` : null,
    },
    {
      label: "Refus",
      count: metrics.rejected,
      glyph: "c",
      bar: "bg-st-c",
      note: null,
    },
  ];
  return (
    <ol className="flex flex-col gap-3">
      {steps.map((step) => {
        const pct = metrics.applications > 0 ? Math.round((step.count / metrics.applications) * 100) : 0;
        return (
          <li key={step.label}>
            <div className="flex items-baseline gap-2.5">
              <StatusGlyph tone={step.glyph} small />
              <span className="text-ui text-tx-2">{step.label}</span>
              <span className="serif-title ml-auto text-[16px] text-tx">{step.count}</span>
              <span className="w-10 text-right font-mono text-caps text-tx-5">{pct} %</span>
            </div>
            <div aria-hidden className="mt-1.5 h-1 rounded-r2 bg-chip">
              <div className={cn("h-full rounded-r2", step.bar)} style={{ width: `${Math.max(pct, step.count > 0 ? 2 : 0)}%` }} />
            </div>
            {step.note ? <p className="mt-1 text-tiny text-tx-5">{step.note}</p> : null}
          </li>
        );
      })}
    </ol>
  );
}

function ChannelRates({ data }: { data: Analytics }) {
  if (data.channels.length === 0) {
    return <p className="text-small text-tx-5">Aucune candidature sur la période.</p>;
  }
  const best = Math.max(...data.channels.map((rate) => (rate.applications > 0 ? rate.responses / rate.applications : 0)));
  return (
    <ul className="flex flex-col gap-2.5">
      {data.channels.map((rate) => {
        const pct = rate.applications > 0 ? Math.round((rate.responses / rate.applications) * 100) : 0;
        const top = rate.applications > 0 && rate.responses / rate.applications === best && best > 0;
        return (
          <li key={rate.channel} className="grid grid-cols-[minmax(0,130px)_1fr_40px_36px] items-center gap-2.5">
            <span className="truncate text-small text-tx-2">{channelLabel(rate.channel)}</span>
            <span aria-hidden className="h-1 rounded-r2 bg-chip">
              <span className={cn("block h-full rounded-r2", top ? "bg-st-g" : "bg-ac")} style={{ width: `${pct}%` }} />
            </span>
            <span className={cn("text-right font-mono text-caps", top ? "text-tint-g-tx" : "text-tx-3")}>{pct} %</span>
            <span className="text-right font-mono text-caps text-tx-6">
              {rate.responses}/{rate.applications}
            </span>
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Analyse (`screens/10-analytics.png`) : quatre indicateurs, le parcours des candidatures,
 * le rythme d'envoi, le taux de réponse par canal et les constats qui s'en dégagent ; puis
 * les candidatures à relancer et la performance. Tout est calculé par SQLite sur la
 * période choisie ; aucune tendance n'est inventée faute de période de comparaison.
 */
export function AnalyticsPage() {
  const vm = useAnalyticsViewModel();
  const [to_follow_up, setToFollowUp] = useState<ToFollowUp | null>(null);
  const data = vm.data ?? null;

  useChrome({
    status: data
      ? `${data.metrics.applications} candidature${data.metrics.applications > 1 ? "s" : ""} analysée${data.metrics.applications > 1 ? "s" : ""} · ${PERIOD_LABEL[vm.period]}`
      : "",
    keys: [],
  });

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-toolbar flex-none items-center gap-2 border-b border-bd-soft px-3.5">
        <SegmentedControl dense label="Période d’analyse" value={vm.period} onChange={vm.changePeriod} options={PERIODES} />
        <span className="ml-auto">
          <Button size="compact" disabled={vm.isExporting} onClick={() => void vm.exportCsv()}>
            <LineIcon name="export-csv" size={13} />
            Exporter en CSV
          </Button>
        </span>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <AnalyticsSkeleton />
        ) : vm.error || !data ? (
          <div className="px-3.5 pt-3.5">
            <ErrorBanner
              message={vm.error instanceof AppError ? vm.error.message : "Les analyses n’ont pas pu être chargées."}
              onRetry={vm.reload}
            />
          </div>
        ) : (
          <div className="flex flex-col gap-2.5 p-3.5">
            <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fit,minmax(min(180px,100%),1fr))]">
              <Kpi label="Candidatures envoyées" value={String(data.metrics.applications)} note={PERIOD_LABEL[vm.period]} />
              <Kpi
                label="Taux de réponse"
                value={String(data.metrics.response_rate)}
                unit="%"
                note={`${data.metrics.responses} réponse${data.metrics.responses > 1 ? "s" : ""}`}
              />
              <Kpi
                label="Entretiens obtenus"
                value={String(data.metrics.interviews)}
                note={`${data.metrics.interview_rate} % des candidatures`}
              />
              <Kpi
                label="Délai moyen de réponse"
                value={data.performance.average_response_days === null ? "—" : String(data.performance.average_response_days)}
                {...(data.performance.average_response_days === null ? {} : { unit: "jours" })}
                note="entre l’envoi et la réponse"
              />
            </div>

            <div className="grid gap-2.5 min-[980px]:grid-cols-2">
              <Block title={`Parcours de vos ${data.metrics.applications} candidatures`}>
                <Journey data={data} />
              </Block>
              <Block title="Rythme d’envoi">
                <ActivityChart activity={data.activity} height={168} shortLabels={vm.period !== "trente_days"} />
              </Block>
            </div>

            <div className="grid gap-2.5 min-[980px]:grid-cols-2">
              <Block title="Taux de réponse par canal">
                <ChannelRates data={data} />
              </Block>
              <Block title="Ce que disent ces chiffres">
                {insightsOf(data).length === 0 ? (
                  <p className="text-small text-tx-5">Pas encore assez de candidatures sur la période pour en tirer un constat.</p>
                ) : (
                  <ul className="flex flex-col gap-2.5">
                    {insightsOf(data).map((insight) => (
                      <li key={insight.text} className="flex gap-2.5 text-small leading-[1.5] text-tx-2">
                        <span className="mt-[5px]">
                          <StatusGlyph tone={insight.tone} small />
                        </span>
                        {insight.text}
                      </li>
                    ))}
                  </ul>
                )}
              </Block>
            </div>

            <div className="grid gap-2.5 min-[980px]:grid-cols-[3fr_2fr]">
              <Block title="Candidatures à relancer" aside={<span className="font-mono text-caps text-st-a">{data.to_follow_up.length}</span>}>
                <FollowUpList items={data.to_follow_up} onFollowUp={setToFollowUp} />
              </Block>
              <Block title="Performance">
                <PerformanceList performance={data.performance} metrics={data.metrics} />
              </Block>
            </div>
          </div>
        )}
      </div>

      <FollowUpFormModal
        open={to_follow_up !== null}
        follow_up={null}
        application_id={to_follow_up?.id ?? null}
        busy={vm.isSaving}
        onClose={() => setToFollowUp(null)}
        onSubmit={vm.createFollowUp}
      />
    </div>
  );
}
