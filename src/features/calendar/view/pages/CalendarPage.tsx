import { useMemo, useState } from "react";
import { useCalendarViewModel } from "../../viewmodel/useCalendarViewModel";
import type { CalendarEvent } from "../../model/event";
import {
  dateFromIso,
  decalerDays,
  decalerMonth,
  isoLocal,
  daysDeLaWeek,
  labelDay,
  labelWeek,
} from "../../model/month";
import { GridMonth } from "../components/GridMonth";
import { ViewDay, ViewWeek } from "../components/ViewAgenda";
import { InterviewFormModal } from "@/features/interviews/view/components/InterviewFormModal";
import { FollowUpFormModal } from "@/features/followups/view/components/FollowUpFormModal";
import type { Interview } from "@/features/interviews/services/interviewService";
import type { FollowUp } from "@/features/followups/services/followUpService";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import {
  Button,
  ConfirmDialog,
  ErrorBanner,
  Icon,
  PageHeader,
  Skeleton,
  StatusPill,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";
import { cn } from "@/shared/lib/cn";

type CalendarView = "mois" | "semaine" | "jour";

const VIEWS: readonly { id: CalendarView; label: string }[] = [
  { id: "mois", label: "Mois" },
  { id: "semaine", label: "Semaine" },
  { id: "jour", label: "Jour" },
];

/** Ce que la page a ouvert : une modale d'entretien, de relance, ou rien. */
type Edition =
  | { kind: "aucune" }
  | { kind: "entretien"; target: Interview | null; day: string | null }
  | { kind: "relance"; target: FollowUp | null; day: string | null };

/** Écran Suivi → Calendrier : entretiens et relances du mois. */
export function CalendarPage() {
  const vm = useCalendarViewModel();
  const [edition, setEdition] = useState<Edition>({ kind: "aucune" });
  const [to_delete, setToDelete] = useState<CalendarEvent | null>(null);
  const [view, setView] = useState<CalendarView>("mois");
  const [selected_day, setSelectedDay] = useState(() => isoLocal(new Date()));

  const close = () => setEdition({ kind: "aucune" });

  const goToDay = (iso: string) => {
    setSelectedDay(iso);
    const date = dateFromIso(iso);
    if (date.getFullYear() !== vm.year || date.getMonth() !== vm.month) {
      vm.allerA(date.getFullYear(), date.getMonth());
    }
  };

  const navigate = (step: number) => {
    if (view === "mois") {
      const next = decalerMonth(vm.year, vm.month, step);
      vm.allerA(next.year, next.month);
      const today = new Date();
      if (today.getFullYear() === next.year && today.getMonth() === next.month) {
        setSelectedDay(isoLocal(today));
      } else {
        setSelectedDay(isoLocal(new Date(next.year, next.month, 1)));
      }
      return;
    }
    goToDay(decalerDays(selected_day, view === "semaine" ? step * 7 : step));
  };

  const goToday = () => {
    vm.revenirToday();
    setSelectedDay(isoLocal(new Date()));
  };

  const heading =
    view === "semaine"
      ? labelWeek(selected_day)
      : view === "jour"
        ? labelDay(selected_day)
        : vm.label;

  const week_days = useMemo(() => daysDeLaWeek(selected_day), [selected_day]);
  const day_events = vm.parDay.get(selected_day) ?? [];

  /** Clic sur un événement : rouvre la modale de son entité d'origine. */
  const openEvent = (event: CalendarEvent) => {
    if (event.kind === "entretien") {
      setEdition({ kind: "entretien", target: vm.interviewDe(event.id), day: null });
    } else {
      setEdition({ kind: "relance", target: vm.followUpDe(event.id), day: null });
    }
  };

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Entretiens et relances</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="calendar_month"
        title="Calendrier"
        subtitle="Entretiens et relances"
        secondary={
          <Button
            icon="send"
            onClick={() => setEdition({ kind: "relance", target: null, day: null })}
          >
            Relance
          </Button>
        }
        primary={
          <Button
            variant="primary"
            icon="add"
            onClick={() => setEdition({ kind: "entretien", target: null, day: null })}
          >
            Nouvel entretien
          </Button>
        }
      />

      <div className="flex flex-none items-center gap-3 border-b border-line bg-surface-alt px-6 py-2.5">
        <div className="flex items-center">
          <MonthNav direction="chevron_left" label="Période précédente" onClick={() => navigate(-1)} />
          <MonthNav direction="chevron_right" label="Période suivante" onClick={() => navigate(1)} joined />
        </div>

        <Button icon="today" onClick={goToday}>
          Aujourd'hui
        </Button>

        <h2 className="text-section text-ink capitalize">{heading}</h2>

        <div className="flex-1" />

        <StatusPill tone="success" icon="event_available">
          {`${vm.countInterviews} entretien${vm.countInterviews > 1 ? "s" : ""}`}
        </StatusPill>
        <StatusPill tone="warning" icon="send">
          {`${vm.countFollowUps} relance${vm.countFollowUps > 1 ? "s" : ""}`}
        </StatusPill>

        <div
          role="group"
          aria-label="Vue du calendrier"
          className="flex items-center gap-0.5 rounded-button bg-neutral-tint p-0.5"
        >
          {VIEWS.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-pressed={view === item.id}
              onClick={() => setView(item.id)}
              className={cn(
                "h-7 rounded-[6px] px-2.5 text-meta font-medium transition-[background-color,color] duration-150",
                view === item.id ? "bg-surface text-ink shadow-e1" : "text-ink-muted hover:text-ink",
              )}
            >
              {item.label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex min-h-0 flex-1 flex-col p-6">
        {vm.error ? (
          <ErrorBanner
            message={
              vm.error instanceof AppError
                ? vm.error.message
                : "Le calendrier n'a pas pu être chargé."
            }
            onRetry={vm.recharger}
          />
        ) : vm.isLoading ? (
          <GridSkeleton />
        ) : view === "semaine" ? (
          <ViewWeek
            days={week_days}
            parDay={vm.parDay}
            selection={selected_day}
            onDayClick={(iso) => {
              goToDay(iso);
              setEdition({ kind: "entretien", target: null, day: iso });
            }}
            onEventClick={openEvent}
          />
        ) : view === "jour" ? (
          <ViewDay
            day={selected_day}
            events={day_events}
            onDayClick={(iso) => setEdition({ kind: "entretien", target: null, day: iso })}
            onEventClick={openEvent}
          />
        ) : (
          <GridMonth
            cells={vm.cells}
            parDay={vm.parDay}
            onDayClick={(iso) => setEdition({ kind: "entretien", target: null, day: iso })}
            onEventClick={openEvent}
          />
        )}
      </div>

      <InterviewFormModal
        open={edition.kind === "entretien"}
        interview={edition.kind === "entretien" ? edition.target : null}
        day={edition.kind === "entretien" ? edition.day : null}
        busy={vm.isSaving}
        onClose={close}
        onSubmit={(values) =>
          vm.saveInterview({
            id: edition.kind === "entretien" ? (edition.target?.id ?? null) : null,
            input: values,
          })
        }
      />

      <FollowUpFormModal
        open={edition.kind === "relance"}
        follow_up={edition.kind === "relance" ? edition.target : null}
        day={edition.kind === "relance" ? edition.day : null}
        busy={vm.isSaving}
        onClose={close}
        onSubmit={(values) =>
          vm.saveFollowUp({
            id: edition.kind === "relance" ? (edition.target?.id ?? null) : null,
            input: values,
          })
        }
      />

      <ConfirmDialog
        open={to_delete !== null}
        title={
          to_delete?.kind === "entretien"
            ? "Supprimer cet entretien ?"
            : "Supprimer cette relance ?"
        }
        description={`« ${to_delete?.label ?? ""} » sera définitivement retiré du calendrier.`}
        note={
          to_delete?.kind === "entretien"
            ? "La candidature conserve son statut « Entretien »."
            : "La candidature n'est pas modifiée."
        }
        busy={vm.isDeleting}
        onCancel={() => setToDelete(null)}
        onConfirm={() => {
          const target = to_delete;
          setToDelete(null);
          if (!target) return;
          if (target.kind === "entretien") void vm.deleteInterview(target.id);
          else void vm.deleteFollowUp(target.id);
        }}
      />
    </div>
  );
}

function MonthNav({
  direction,
  label,
  onClick,
  joined = false,
}: {
  direction: string;
  label: string;
  onClick: () => void;
  joined?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        "flex size-[30px] items-center justify-center border border-line bg-surface text-ink-muted transition-colors duration-150 hover:bg-neutral-tint hover:text-ink",
        joined ? "rounded-r-button border-l-0" : "rounded-l-button",
      )}
    >
      <Icon name={direction} size={16} />
    </button>
  );
}

/** Squelette de la grille, aux dimensions des cases réelles. */
function GridSkeleton() {
  return (
    <div
      role="status"
      aria-label="Chargement du calendrier"
      className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6 overflow-hidden rounded-card border border-line bg-surface"
    >
      {Array.from({ length: 42 }, (_, index) => (
        <div key={index} className="flex flex-col gap-1 border-r border-b border-line p-1.5">
          <Skeleton className="size-6 flex-none rounded-pill" />
          {index % 5 === 0 ? <Skeleton className="h-3.5 w-full" /> : null}
        </div>
      ))}
    </div>
  );
}
