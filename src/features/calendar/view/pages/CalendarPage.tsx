import { useMemo, useState } from "react";
import { useCalendarViewModel } from "../../viewmodel/useCalendarViewModel";
import type { CalendarEvent } from "../../model/event";
import {
  dateFromIso,
  shiftDays,
  shiftMonth,
  isoLocal,
  daysDeLaWeek,
  labelDay,
  labelWeek,
} from "../../model/month";
import { GridMonth } from "../components/GridMonth";
import { ViewDay, ViewWeek } from "../components/ViewAgenda";
import { InterviewFormModal } from "@/features/interviews";
import { FollowUpFormModal, type FollowUp } from "@/features/followups";
import type { Interview } from "@/features/interviews";
import { Button, ConfirmDialog, ErrorBanner, SegmentedControl, Skeleton, StatusGlyph } from "@/shared/ui";
import { useChrome } from "@/shared/lib/chrome";
import { AppError } from "@/shared/types/app-error";

type CalendarView = "mois" | "semaine" | "jour";

const VIEWS: readonly { value: CalendarView; label: string }[] = [
  { value: "mois", label: "Mois" },
  { value: "semaine", label: "Semaine" },
  { value: "jour", label: "Jour" },
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
      const next = shiftMonth(vm.year, vm.month, step);
      vm.allerA(next.year, next.month);
      const today = new Date();
      if (today.getFullYear() === next.year && today.getMonth() === next.month) {
        setSelectedDay(isoLocal(today));
      } else {
        setSelectedDay(isoLocal(new Date(next.year, next.month, 1)));
      }
      return;
    }
    goToDay(shiftDays(selected_day, view === "semaine" ? step * 7 : step));
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
    if (event.kind === "interview") {
      setEdition({ kind: "entretien", target: vm.interviewDe(event.id), day: null });
    } else {
      setEdition({ kind: "relance", target: vm.followUpDe(event.id), day: null });
    }
  };

  const upcomingFollowUps = [...vm.parDay.values()]
    .flat()
    .filter((event) => event.kind === "follow_up" && !event.done && event.day >= isoLocal(new Date())).length;
  const monthEvents = vm.countInterviews + vm.countFollowUps;
  useChrome({
    status: `${monthEvents} échéance${monthEvents > 1 ? "s" : ""} en ${vm.label.split(" ")[0]?.toLowerCase() ?? ""} · ${upcomingFollowUps} relance${upcomingFollowUps > 1 ? "s" : ""} à venir`,
    keys: [],
  });

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-toolbar flex-none items-center gap-2 border-b border-bd-soft px-3.5">
        <button
          type="button"
          aria-label="Période précédente"
          onClick={() => navigate(-1)}
          className="flex size-[22px] items-center justify-center rounded-r6 text-small text-tx-4 hover:bg-hover hover:text-tx"
        >
          ‹
        </button>
        <h1 className="serif-title min-w-[150px] text-center text-[16px] text-tx capitalize">{heading}</h1>
        <button
          type="button"
          aria-label="Période suivante"
          onClick={() => navigate(1)}
          className="flex size-[22px] items-center justify-center rounded-r6 text-small text-tx-4 hover:bg-hover hover:text-tx"
        >
          ›
        </button>
        <Button variant="ghost" size="compact" onClick={goToday}>
          Aujourd&apos;hui
        </Button>
        <span className="ml-2 hidden items-center gap-3 text-tiny text-tx-4 min-[1100px]:flex">
          <span className="flex items-center gap-1.5">
            <StatusGlyph tone="a" small />
            {`Relance à envoyer · ${vm.countFollowUps}`}
          </span>
          <span className="flex items-center gap-1.5">
            <StatusGlyph tone="g" small />
            {`Entretien · ${vm.countInterviews}`}
          </span>
        </span>
        <span className="ml-auto flex flex-none items-center gap-1.5">
          <SegmentedControl dense label="Vue du calendrier" value={view} options={VIEWS} onChange={setView} />
          <Button size="compact" onClick={() => setEdition({ kind: "relance", target: null, day: null })}>
            Programmer une relance
          </Button>
          <Button
            variant="primary"
            size="compact"
            onClick={() => setEdition({ kind: "entretien", target: null, day: null })}
          >
            Nouvel entretien
          </Button>
        </span>
      </div>

      <div className="flex min-h-0 flex-1 flex-col">
        {vm.error ? (
          <ErrorBanner
            message={
              vm.error instanceof AppError
                ? vm.error.message
                : "Le calendrier n'a pas pu être chargé."
            }
            onRetry={vm.reload}
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
          to_delete?.kind === "interview"
            ? "Supprimer cet entretien ?"
            : "Supprimer cette relance ?"
        }
        description={`« ${to_delete?.label ?? ""} » sera définitivement retiré du calendrier.`}
        note={
          to_delete?.kind === "interview"
            ? "La candidature conserve son statut « Entretien »."
            : "La candidature n'est pas modifiée."
        }
        busy={vm.isDeleting}
        onCancel={() => setToDelete(null)}
        onConfirm={() => {
          const target = to_delete;
          setToDelete(null);
          if (!target) return;
          if (target.kind === "interview") void vm.deleteInterview(target.id);
          else void vm.deleteFollowUp(target.id);
        }}
      />
    </div>
  );
}

/** Squelette de la grille, aux dimensions des cases réelles. */
function GridSkeleton() {
  return (
    <div
      role="status"
      aria-label="Chargement du calendrier"
      className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6 overflow-hidden"
    >
      {Array.from({ length: 42 }, (_, index) => (
        <div key={index} className="flex flex-col gap-1 border-r border-b border-bd-soft p-1.5">
          <Skeleton className="size-6 flex-none rounded-pill" />
          {index % 5 === 0 ? <Skeleton className="h-3.5 w-full" /> : null}
        </div>
      ))}
    </div>
  );
}
