import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import type { ARelancer, Periode } from "@/shared/types/generated/analyses";
import { useAnalysesViewModel } from "../../viewmodel/useAnalysesViewModel";
import { analysesService } from "../../services/analyses.service";
import {
  ActivityChart,
  AnalyticsPanel,
  AnalyticsSkeleton,
  FollowUpList,
  FunnelChart,
  MetricCard,
  PerformanceList,
} from "../components/AnalyticsUi";
import { RelanceFormModal } from "@/features/relances/view/components/RelanceFormModal";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { Button, ErrorBanner, PageHeader } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

const PERIODES: readonly { valeur: Periode; label: string }[] = [
  { valeur: "trenteJours", label: "30 j" },
  { valeur: "quatreVingtDixJours", label: "90 j" },
  { valeur: "tout", label: "Tout" },
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
      <PageHeader
        icon="monitoring"
        title="Analyses et performance"
        subtitle={vm.data ? `${vm.data.indicateurs.candidatures} candidatures sur la période` : "Suivi des conversions"}
        secondary={
          <div aria-label="Période d’analyse" className="flex items-center gap-0.5 rounded-button bg-neutral-tint p-0.5">
            {PERIODES.map((periode) => (
              <button
                key={periode.valeur}
                type="button"
                aria-pressed={vm.periode === periode.valeur}
                onClick={() => vm.changerPeriode(periode.valeur)}
                className={cn(
                  "h-7 rounded-[6px] px-2.5 text-meta font-medium transition-[background-color,color] duration-150",
                  vm.periode === periode.valeur
                    ? "bg-surface text-ink shadow-e1"
                    : "text-ink-muted hover:text-ink",
                )}
              >
                {periode.label}
              </button>
            ))}
          </div>
        }
        primary={
          <Button variant="primary" icon="download" disabled={exportEnCours} onClick={() => void exporter()}>
            Exporter
          </Button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <AnalyticsSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="p-6">
            <ErrorBanner
              message={vm.error instanceof AppError ? vm.error.message : "Les analyses n’ont pas pu être chargées."}
              onRetry={vm.recharger}
            />
          </div>
        ) : (
          <div className="space-y-4 p-5 min-[1200px]:p-6">
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-[1.2fr_1fr_1fr_1fr]">
              <MetricCard
                primary
                tone="accent"
                icon="work"
                label="Candidatures"
                value={vm.data.indicateurs.candidatures.toString()}
                context="envoyées sur la période"
                sparkline={vm.data.activite.map((semaine) => semaine.nombre)}
              />
              <MetricCard
                tone="success"
                icon="event_available"
                label="Entretiens"
                value={vm.data.indicateurs.entretiens.toString()}
                context={`${vm.data.indicateurs.tauxEntretien} % des candidatures`}
              />
              <MetricCard
                tone="accent"
                icon="mark_email_read"
                label="Taux de réponse"
                value={`${vm.data.indicateurs.tauxReponse} %`}
                context={`${vm.data.indicateurs.reponses} réponse${vm.data.indicateurs.reponses > 1 ? "s" : ""}`}
              />
              <MetricCard
                tone="danger"
                icon="do_not_disturb_on"
                label="Refus reçus"
                value={vm.data.indicateurs.refus.toString()}
                context={vm.data.indicateurs.candidatures === 0 ? "aucune candidature" : `${Math.round((vm.data.indicateurs.refus / vm.data.indicateurs.candidatures) * 100)} % de la période`}
              />
            </div>

            <div className="grid grid-cols-1 gap-4 xl:grid-cols-[1.35fr_1fr]">
              <AnalyticsPanel icon="bar_chart_4_bars" title="Candidatures envoyées" meta={PERIODES.find((item) => item.valeur === vm.periode)?.label}>
                <ActivityChart activite={vm.data.activite} />
              </AnalyticsPanel>
              <AnalyticsPanel icon="conversion_path" title="Entonnoir de conversion">
                <FunnelChart etapes={vm.data.entonnoir} />
              </AnalyticsPanel>
            </div>

            <div className="grid grid-cols-1 gap-4 xl:grid-cols-[1.3fr_0.7fr]">
              <AnalyticsPanel icon="notifications_active" title="Candidatures à relancer" meta={`${vm.data.aRelancer.length} à traiter`}>
                <FollowUpList items={vm.data.aRelancer} onRelancer={setARelancer} />
              </AnalyticsPanel>
              <AnalyticsPanel icon="insights" title="Performance">
                <PerformanceList performance={vm.data.performance} indicateurs={vm.data.indicateurs} />
              </AnalyticsPanel>
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
