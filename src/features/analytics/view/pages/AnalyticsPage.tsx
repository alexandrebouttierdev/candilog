import { useState } from "react";
import type { ToFollowUp, Period } from "@/shared/types/generated/analytics";
import { useAnalyticsViewModel } from "../../viewmodel/useAnalyticsViewModel";
import {
  AnalyticsSkeleton,
  FollowUpList,
  PerformanceList,
} from "../components/AnalyticsUi";
import { ActivityChart, FunnelChart } from "../components/charts";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { FollowUpFormModal } from "@/features/followups";
import { AppError } from "@/shared/types/app-error";
import {
  Button,
  Card,
  CardHeader,
  CardTitle,
  ErrorBanner,
  PageHeader,
  SegmentedControl,
  StatCard,
  StatusPill,
} from "@/shared/ui";

const PERIODES: readonly { value: Period; label: string }[] = [
  { value: "trente_days", label: "30 j" },
  { value: "quatre_vingt_dix_days", label: "90 j" },
  { value: "tout", label: "Tout" },
];

/** Statistiques de candidature, période et export réellement interactifs. */
export function AnalyticsPage() {
  const vm = useAnalyticsViewModel();
  const [to_follow_up, setToFollowUp] = useState<ToFollowUp | null>(null);

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Période glissante · export CSV</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="monitoring"
        title="Analyses"
        subtitle={
          vm.data
            ? `${vm.data.metrics.applications} candidature${
                vm.data.metrics.applications > 1 ? "s" : ""
              } sur la période`
            : "Suivi des conversions"
        }
        toolbar={
          <SegmentedControl
            label="Période d’analyse"
            value={vm.period}
            onChange={vm.changePeriod}
            options={PERIODES}
          />
        }
        primary={
          <Button
            variant="primary"
            icon="download"
            disabled={vm.isExporting}
            onClick={() => void vm.exportCsv()}
          >
            Export
          </Button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <AnalyticsSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="px-[18px] pt-4">
            <ErrorBanner
              message={
                vm.error instanceof AppError
                  ? vm.error.message
                  : "Les analyses n’ont pas pu être chargées."
              }
              onRetry={vm.recharger}
            />
          </div>
        ) : (
          <div className="flex flex-col gap-4 px-[18px] pt-4 pb-[22px]">
            <div className="grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(min(190px,100%),1fr))]">
              <StatCard
                icon="work"
                tone="accent"
                label="Candidatures"
                value={vm.data.metrics.applications.toString()}
                delta="envoyées"
              />
              <StatCard
                icon="event_available"
                tone="success"
                label="Entretiens"
                value={vm.data.metrics.interviews.toString()}
                delta={`${vm.data.metrics.interview_rate} %`}
                deltaTone="success"
              />
              <StatCard
                icon="mark_email_read"
                tone="accent"
                label="Taux de réponse"
                value={`${vm.data.metrics.response_rate} %`}
                delta={`${vm.data.metrics.responses} réponse${
                  vm.data.metrics.responses > 1 ? "s" : ""
                }`}
              />
              <StatCard
                icon="do_not_disturb_on"
                tone="danger"
                label="Refus reçus"
                value={vm.data.metrics.rejected.toString()}
                delta={
                  vm.data.metrics.applications === 0
                    ? "aucune candidature"
                    : `${Math.round(
                        (vm.data.metrics.rejected / vm.data.metrics.applications) * 100,
                      )} % du total`
                }
                deltaTone="danger"
              />
            </div>

            <div className="flex flex-wrap items-start gap-4">
              <Card padded className="flex-[1_1_480px]">
                <CardTitle
                  icon="bar_chart_4_bars"
                  className="mb-[18px]"
                  meta={
                    <SegmentedControl
                      dense
                      label="Période du graphique"
                      value={vm.period}
                      onChange={vm.changePeriod}
                      options={PERIODES}
                    />
                  }
                >
                  Candidatures envoyées
                </CardTitle>
                <ActivityChart
                  activity={vm.data.activity}
                  height={168}
                  shortLabels={vm.period !== "trente_days"}
                />
              </Card>

              <Card padded className="flex-[1_1_320px]">
                <CardTitle icon="conversion_path" className="mb-4">
                  Funnel de conversion
                </CardTitle>
                <FunnelChart steps={vm.data.funnel} />
              </Card>
            </div>

            <div className="flex flex-wrap items-start gap-4">
              <Card clipped className="flex-[1_1_420px]">
                <CardHeader
                  icon="notifications_active"
                  iconClassName="text-warning"
                  meta={
                    <StatusPill tone="warning" compact className="font-semibold">
                      {vm.data.to_follow_up.length}
                    </StatusPill>
                  }
                >
                  Candidatures à relancer
                </CardHeader>
                <FollowUpList items={vm.data.to_follow_up} onFollowUp={setToFollowUp} />
              </Card>

              <Card padded className="flex-[1_1_300px]">
                <CardTitle icon="insights" className="mb-[15px]">
                  Performance
                </CardTitle>
                <PerformanceList
                  performance={vm.data.performance}
                  metrics={vm.data.metrics}
                />
              </Card>
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
