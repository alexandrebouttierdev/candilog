import { useNavigate } from "react-router-dom";
import { useDashboardViewModel } from "../../viewmodel/useDashboardViewModel";
import type { Dashboard } from "@/shared/types/generated/analytics";
import {
  NextEvent,
  RecentRows,
  TodayActivity,
  TodayEmpty,
  TodayPipeline,
  TodaySkeleton,
  TodayStats,
  TodoRows,
  UpcomingEmpty,
  UpcomingRows,
  isTodayEmpty,
  splitUpcoming,
} from "../components/TodayUi";
import { ContextBarAccessory } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import { Button, ErrorBanner, InspectorSectionLabel } from "@/shared/ui";

/** Centre d'activité : prochain rendez-vous, tâches, suivi. */
export function DashboardPage() {
  const vm = useDashboardViewModel();
  const navigate = useNavigate();

  const date = new Intl.DateTimeFormat("fr-FR", {
    weekday: "long",
    day: "numeric",
    month: "long",
  }).format(new Date());

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <span className="hidden text-note text-ink-faint min-[1280px]:inline">
          {date.charAt(0).toUpperCase() + date.slice(1)}
        </span>
        <Button icon="calendar_month" onClick={() => void navigate("/tracking/calendar")}>
          Calendrier
        </Button>
        <Button
          variant="primary"
          icon="add"
          onClick={() => void navigate("/tracking/applications?nouvelle=1")}
        >
          Nouvelle
        </Button>
      </ContextBarAccessory>

      {vm.isLoading ? (
        <TodaySkeleton />
      ) : vm.error || !vm.data ? (
        <div className="px-[18px] pt-4">
          <ErrorBanner
            message={
              vm.error instanceof AppError
                ? vm.error.message
                : "L'écran d'accueil n'a pas pu être chargé."
            }
            onRetry={vm.recharger}
          />
        </div>
      ) : (
        <TodayWorkspace data={vm.data} />
      )}
    </div>
  );
}

function TodayWorkspace({ data }: { data: Dashboard }) {
  const navigate = useNavigate();
  const { next, rest } = splitUpcoming(data.upcoming_items);

  if (isTodayEmpty(data)) {
    return (
      <TodayEmpty
        onCreate={() => void navigate("/tracking/applications?nouvelle=1")}
        onOpenApplications={() => void navigate("/tracking/applications")}
        onOpenCalendar={() => void navigate("/tracking/calendar")}
      />
    );
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-[18px] pt-4 pb-[22px]">
      <TodayStats
        applications={data.metrics.applications}
        responses={data.metrics.responses}
        interviews={data.performance.upcoming_interviews}
        overdue={data.performance.overdue_follow_ups}
      />

      <div className="grid min-h-0 gap-x-8 gap-y-6 min-[1280px]:grid-cols-[minmax(0,1.05fr)_minmax(0,1fr)]">
        <section className="min-w-0">
          <InspectorSectionLabel>Prochainement</InspectorSectionLabel>
          {next === null ? (
            <UpcomingEmpty onOpenCalendar={() => void navigate("/tracking/calendar")} />
          ) : (
            <>
              <NextEvent
                item={next}
                href={next.kind === "entretien" ? "/tracking/calendar" : "/tracking/applications"}
              />
              {rest.length > 0 ? (
                <UpcomingRows
                  items={rest}
                  hrefFor={(item) =>
                    item.kind === "entretien" ? "/tracking/calendar" : "/tracking/applications"
                  }
                />
              ) : null}
            </>
          )}

          <TodoRows
            overdue={data.performance.overdue_follow_ups}
            items={data.upcoming_items}
            onOpenApplications={() => void navigate("/tracking/applications")}
            onOpenCalendar={() => void navigate("/tracking/calendar")}
          />
        </section>

        <section className="min-w-0 min-[1280px]:border-l min-[1280px]:border-line-soft min-[1280px]:pl-8">
          <div className="flex items-center justify-between">
            <InspectorSectionLabel>Candidatures récentes</InspectorSectionLabel>
            <button
              type="button"
              onClick={() => void navigate("/tracking/applications")}
              className="mb-[7px] text-label font-medium text-accent-text hover:text-accent-hover"
            >
              Tout voir
            </button>
          </div>
          <RecentRows
            applications={data.recent}
            onOpen={(id) =>
              void navigate(`/tracking/applications?fiche=${encodeURIComponent(id)}`)
            }
          />
          <TodayActivity activity={data.activity} />
          <TodayPipeline pipeline={data.pipeline} />
        </section>
      </div>
    </div>
  );
}
