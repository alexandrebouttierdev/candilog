import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import type { ARelancer, Periode } from "@/shared/types/generated/analyses";
import { useAnalysesViewModel } from "../../viewmodel/useAnalysesViewModel";
import { analysesService } from "../../services/analyses.service";
import {
  ActivityChart,
  AnalyticsSkeleton,
  FollowUpList,
  FunnelChart,
  PerformanceList,
} from "../components/AnalyticsUi";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { RelanceFormModal } from "@/features/relances/view/components/RelanceFormModal";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
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

const PERIODES: readonly { value: Periode; label: string }[] = [
  { value: "trenteJours", label: "30 j" },
  { value: "quatreVingtDixJours", label: "90 j" },
  { value: "tout", label: "Tout" },
];

/** Statistiques de candidature, période et export réellement interactifs. */
export function AnalysesPage() {
  const vm = useAnalysesViewModel();
  const notify = useUiStore((state) => state.notify);
  const [aRelancer, setARelancer] = useState<ARelancer | null>(null);
  const [exportEnCours, setExportEnCours] = useState(false);

  const exporter = async () => {
    const chemin = await save({
      title: "Exporter les analyses",
      defaultPath: "analyses-candilog.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (chemin === null) return;
    setExportEnCours(true);
    try {
      await analysesService.exporterCsv(vm.periode, chemin);
      notify({ tone: "success", title: "Analyses exportées" });
    } catch (error) {
      notify({
        tone: "error",
        title: "Export impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    } finally {
      setExportEnCours(false);
    }
  };

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
            ? `${vm.data.indicateurs.candidatures} candidature${
                vm.data.indicateurs.candidatures > 1 ? "s" : ""
              } sur la période`
            : "Suivi des conversions"
        }
        toolbar={
          <SegmentedControl
            label="Période d’analyse"
            value={vm.periode}
            onChange={vm.changerPeriode}
            options={PERIODES}
          />
        }
        primary={
          <Button
            variant="primary"
            icon="download"
            disabled={exportEnCours}
            onClick={() => void exporter()}
          >
            Exporter
          </Button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <AnalyticsSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="px-7 pt-[22px]">
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
          <div className="px-7 pt-[22px] pb-8">
            <div className="mb-4 grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(190px,100%),1fr))]">
              <StatCard
                icon="work"
                tone="accent"
                label="Candidatures"
                value={vm.data.indicateurs.candidatures.toString()}
                delta="envoyées"
              />
              <StatCard
                icon="event_available"
                tone="success"
                label="Entretiens"
                value={vm.data.indicateurs.entretiens.toString()}
                delta={`${vm.data.indicateurs.tauxEntretien} %`}
                deltaTone="success"
              />
              <StatCard
                icon="mark_email_read"
                tone="accent"
                label="Taux de réponse"
                value={`${vm.data.indicateurs.tauxReponse} %`}
                delta={`${vm.data.indicateurs.reponses} réponse${
                  vm.data.indicateurs.reponses > 1 ? "s" : ""
                }`}
              />
              <StatCard
                icon="do_not_disturb_on"
                tone="danger"
                label="Refus reçus"
                value={vm.data.indicateurs.refus.toString()}
                delta={
                  vm.data.indicateurs.candidatures === 0
                    ? "aucune candidature"
                    : `${Math.round(
                        (vm.data.indicateurs.refus / vm.data.indicateurs.candidatures) * 100,
                      )} % du total`
                }
                deltaTone="danger"
              />
            </div>

            <div className="mb-4 flex flex-wrap items-start gap-3.5">
              <Card padded className="flex-[1_1_480px]">
                <CardTitle
                  icon="bar_chart_4_bars"
                  className="mb-[18px]"
                  meta={
                    <SegmentedControl
                      dense
                      label="Période du graphique"
                      value={vm.periode}
                      onChange={vm.changerPeriode}
                      options={PERIODES}
                    />
                  }
                >
                  Candidatures envoyées
                </CardTitle>
                <ActivityChart activite={vm.data.activite} height={150} gap={10} />
              </Card>

              <Card padded className="flex-[1_1_320px]">
                <CardTitle icon="conversion_path" className="mb-4">
                  Entonnoir de conversion
                </CardTitle>
                <FunnelChart etapes={vm.data.entonnoir} />
              </Card>
            </div>

            <div className="flex flex-wrap items-start gap-3.5">
              <Card clipped className="flex-[1_1_420px]">
                <CardHeader
                  icon="notifications_active"
                  iconClassName="text-warning"
                  meta={
                    <StatusPill tone="warning" compact className="font-semibold">
                      {vm.data.aRelancer.length}
                    </StatusPill>
                  }
                >
                  Candidatures à relancer
                </CardHeader>
                <FollowUpList items={vm.data.aRelancer} onRelancer={setARelancer} />
              </Card>

              <Card padded className="flex-[1_1_300px]">
                <CardTitle icon="insights" className="mb-[15px]">
                  Performance
                </CardTitle>
                <PerformanceList
                  performance={vm.data.performance}
                  indicateurs={vm.data.indicateurs}
                />
              </Card>
            </div>
          </div>
        )}
      </div>

      <RelanceFormModal
        open={aRelancer !== null}
        relance={null}
        candidatureId={aRelancer?.id ?? null}
        busy={vm.isSaving}
        onClose={() => setARelancer(null)}
        onSubmit={vm.creerRelance}
      />
    </div>
  );
}
