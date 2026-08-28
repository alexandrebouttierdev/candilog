import { useNavigate } from "react-router-dom";
import { useTableauDeBordViewModel } from "../../viewmodel/useTableauDeBordViewModel";
import {
  ActivityChart,
  AnalyticsSkeleton,
  PipelineBar,
  RecentApplications,
  UpcomingList,
} from "../components/AnalyticsUi";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import {
  Button,
  Card,
  CardHeader,
  CardLink,
  CardMeta,
  CardTitle,
  ErrorBanner,
  PageHeader,
  StatCard,
} from "@/shared/ui";

/** Grilles auto-ajustées des maquettes : les cartes se replient sous une largeur plancher. */
const GRILLE_KPI = "grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(200px,100%),1fr))]";
const GRILLE_CARTES =
  "grid gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(320px,100%),1fr))]";

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
      <ContextBarAccessory>
        <ContextNote>Vue d’ensemble de la recherche</ContextNote>
      </ContextBarAccessory>
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
          <div className="px-7 pt-[22px]">
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
          <div className="px-7 pt-[22px] pb-8">
            <div className={`${GRILLE_KPI} mb-4`}>
              <StatCard
                icon="work"
                tone="accent"
                label="Candidatures actives"
                value={vm.data.indicateurs.enAttente.toString()}
                delta={`${vm.data.indicateurs.candidatures} sur 30 j`}
                deltaIcon="trending_up"
                deltaTone="success"
              />
              <StatCard
                icon="event_available"
                tone="success"
                label="Entretiens à venir"
                value={vm.data.performance.entretiensAVenir.toString()}
                delta="tous horizons"
                deltaIcon="schedule"
              />
              <StatCard
                icon="mark_email_read"
                tone="accent"
                label="Taux de réponse"
                value={`${vm.data.indicateurs.tauxReponse} %`}
                delta={`${vm.data.indicateurs.reponses} réponse${
                  vm.data.indicateurs.reponses > 1 ? "s" : ""
                }`}
                deltaIcon="mark_chat_read"
                deltaTone="success"
              />
              <StatCard
                icon="notifications_active"
                tone="warning"
                label="Relances à traiter"
                value={vm.data.performance.relancesEnRetard.toString()}
                delta="en retard"
                deltaIcon="priority_high"
                deltaTone="warning"
              />
            </div>

            <div className={`${GRILLE_CARTES} mb-4`}>
              <Card padded>
                <CardTitle
                  icon="event_upcoming"
                  meta={<CardMeta>{vm.data.echeances.length} à venir</CardMeta>}
                  className="mb-3"
                >
                  Prochains événements
                </CardTitle>
                <UpcomingList echeances={vm.data.echeances} />
              </Card>

              <Card padded>
                <CardTitle
                  icon="bar_chart_4_bars"
                  meta={<CardMeta>8 dernières semaines</CardMeta>}
                  className="mb-4"
                >
                  Activité récente
                </CardTitle>
                <ActivityChart activite={vm.data.activite} />
              </Card>
            </div>

            <Card padded className="mb-4">
              <CardTitle
                icon="conversion_path"
                meta={<CardMeta>Taux d’entretien {vm.data.indicateurs.tauxEntretien} %</CardMeta>}
                className="mb-[13px]"
              >
                Pipeline
              </CardTitle>
              <PipelineBar etapes={vm.data.pipeline} />
            </Card>

            <RecentApplications
              candidatures={vm.data.recentes}
              header={
                <CardHeader
                  icon="work_history"
                  meta={
                    <CardLink onClick={() => void navigate("/suivi/candidatures")}>
                      Tout voir
                    </CardLink>
                  }
                >
                  Candidatures récentes
                </CardHeader>
              }
              onOuvrir={(candidature) =>
                void navigate(`/suivi/candidatures?fiche=${candidature.id}`)
              }
            />
          </div>
        )}
      </div>
    </div>
  );
}
