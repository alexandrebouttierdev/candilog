import { useNavigate } from "react-router-dom";
import { useDashboardViewModel } from "../../viewmodel/useDashboardViewModel";
import type { Dashboard } from "@/shared/types/generated/analytics";
import {
  NextEvent,
  RecentRows,
  TodayActivity,
  TodayCard,
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
import { AppError } from "@/shared/types/app-error";
import { Button, ErrorBanner } from "@/shared/ui";
import { PATHS, applicationsPath } from "@/shared/lib/paths";

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
      <div className="flex flex-none items-center justify-between gap-3 border-b border-line px-[18px] py-3">
        <span className="text-note text-ink-faint">
          {date.charAt(0).toUpperCase() + date.slice(1)}
        </span>
        <div className="flex items-center gap-2">
          <Button icon="calendar_month" onClick={() => void navigate(PATHS.calendar)}>
            Calendrier
          </Button>
          <Button
            variant="primary"
            icon="add"
            onClick={() => void navigate(applicationsPath({ create: true }))}
          >
            Nouvelle
          </Button>
        </div>
      </div>

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
            onRetry={vm.reload}
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
        onCreate={() => void navigate(applicationsPath({ create: true }))}
        onOpenApplications={() => void navigate(PATHS.applications)}
        onOpenCalendar={() => void navigate(PATHS.calendar)}
      />
    );
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-[18px] pt-4 pb-[22px]">
      <div className="flex flex-col gap-4">
        <TodayStats
          applications={data.metrics.applications}
          responses={data.metrics.responses}
          interviews={data.metrics.interviews}
        />

        <div className="grid min-h-0 gap-4 min-[1280px]:grid-cols-[minmax(0,1.05fr)_minmax(0,1fr)] min-[1280px]:items-start">
          <div className="flex min-w-0 flex-col gap-4">
            <TodayCard label="Prochainement">
              {next === null ? (
                <UpcomingEmpty onOpenCalendar={() => void navigate(PATHS.calendar)} />
              ) : (
                <>
                  <NextEvent
                    item={next}
                    href={next.kind === "entretien" ? PATHS.calendar : PATHS.applications}
                  />
                  {rest.length > 0 ? (
                    <UpcomingRows
                      items={rest}
                      hrefFor={(item) =>
                        item.kind === "entretien" ? PATHS.calendar : PATHS.applications
                      }
                    />
                  ) : null}
                </>
              )}
            </TodayCard>

            <TodoRows
              overdue={data.performance.overdue_follow_ups}
              items={data.upcoming_items}
              onOpenApplications={() => void navigate(PATHS.applications)}
              onOpenCalendar={() => void navigate(PATHS.calendar)}
            />
          </div>

          <div className="flex min-w-0 flex-col gap-4">
            <TodayCard
              label="Candidatures récentes"
              action={
                <button
                  type="button"
                  onClick={() => void navigate(PATHS.applications)}
                  className="text-label font-medium text-accent-text hover:text-accent-hover"
                >
                  Tout voir
                </button>
              }
            >
              <RecentRows
                applications={data.recent}
                onOpen={(id) =>
                  void navigate(applicationsPath({ id }))
                }
                onCreate={() => void navigate(applicationsPath({ create: true }))}
              />
            </TodayCard>
            <TodayActivity activity={data.activity} />
            <TodayPipeline pipeline={data.pipeline} />
          </div>
        </div>
      </div>
    </div>
  );
}
