import { useNavigate } from "react-router-dom";
import { useTableauDeBordViewModel } from "../../viewmodel/useTableauDeBordViewModel";
import {
  ActivityChart,
  AnalyticsPanel,
  AnalyticsSkeleton,
  MetricCard,
  PipelineBar,
  RecentApplications,
  UpcomingList,
} from "../components/AnalyticsUi";
import { AppError } from "@/shared/types/app-error";
import { Button, ErrorBanner, PageHeader } from "@/shared/ui";

/** Accueil : état de la recherche d'emploi et prochaines actions. */
export function DashboardPage() {
  const vm = useTableauDeBordViewModel();
  const navigate = useNavigate();
  const date = new Intl.DateTimeFormat("fr-FR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  }).format(new Date());

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="space_dashboard"
        title="Tableau de bord"
        subtitle={date.charAt(0).toUpperCase() + date.slice(1)}
        secondary={
          <Button icon="calendar_month" onClick={() => void navigate("/suivi/calendrier")}>
            Calendrier
          </Button>
        }
        primary={
          <Button
            variant="primary"
            icon="add"
            onClick={() => void navigate("/suivi/candidatures?nouvelle=1")}
          >
            Nouvelle candidature
          </Button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <AnalyticsSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="p-6">
            <ErrorBanner
              message={
                vm.error instanceof AppError
                  ? vm.error.message
                  : "Le tableau de bord n’a pas pu être chargé."
              }
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
                label="Candidatures actives"
                value={Math.max(0, vm.data.indicateurs.candidatures - vm.data.indicateurs.refus).toString()}
                context="envoyées sur les 30 derniers jours"
                sparkline={vm.data.activite.map((semaine) => semaine.nombre)}
              />
              <MetricCard
                tone="success"
                icon="event_available"
                label="Entretiens à venir"
                value={vm.data.performance.entretiensAVenir.toString()}
                context="tous horizons confondus"
              />
              <MetricCard
                tone="accent"
                icon="mark_email_read"
                label="Taux de réponse"
                value={`${vm.data.indicateurs.tauxReponse} %`}
                context={`${vm.data.indicateurs.reponses} réponse${vm.data.indicateurs.reponses > 1 ? "s" : ""} reçue${vm.data.indicateurs.reponses > 1 ? "s" : ""}`}
              />
              <MetricCard
                tone="warning"
                icon="notifications_active"
                label="Relances à traiter"
                value={vm.data.performance.relancesEnRetard.toString()}
                context="dont la date est dépassée"
              />
            </div>

            <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
              <AnalyticsPanel
                icon="event_upcoming"
                title="Prochains événements"
                meta={`${vm.data.echeances.length} à venir`}
              >
                <UpcomingList echeances={vm.data.echeances} />
              </AnalyticsPanel>
              <AnalyticsPanel icon="bar_chart_4_bars" title="Activité récente" meta="8 dernières semaines">
                <ActivityChart activite={vm.data.activite} />
              </AnalyticsPanel>
            </div>

            <AnalyticsPanel
              icon="conversion_path"
              title="Pipeline"
              meta={`Taux d’entretien ${vm.data.indicateurs.tauxEntretien} %`}
            >
              <PipelineBar etapes={vm.data.pipeline} />
            </AnalyticsPanel>

            <AnalyticsPanel
              icon="work_history"
              title="Candidatures récentes"
              meta={
                <button
                  type="button"
                  onClick={() => void navigate("/suivi/candidatures")}
                  className="inline-flex items-center gap-1 font-medium text-accent hover:underline"
                >
                  Tout voir
                </button>
              }
            >
              <RecentApplications candidatures={vm.data.recentes} />
            </AnalyticsPanel>
          </div>
        )}
      </div>
    </div>
  );
}
