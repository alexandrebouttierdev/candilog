import { useState } from "react";
import type { KeyboardEvent } from "react";
import { useNavigate } from "react-router-dom";
import type { AgendaItem } from "@/shared/types/generated/analytics";
import { Statuses, formatReference } from "@/features/applications";
import { FollowUpFormModal } from "@/features/followups";
import type { FollowUp } from "@/features/followups";
import { PATHS, applicationsPath } from "@/shared/lib/paths";
import { useChrome } from "@/shared/lib/chrome";
import { cn } from "@/shared/lib/cn";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { AppError } from "@/shared/types/app-error";
import { Button, EmptyState, ErrorBanner, Kbd, Skeleton, StatusGlyph } from "@/shared/ui";
import type { GlyphTone } from "@/shared/ui";
import { useTodayViewModel } from "../../viewmodel/useTodayViewModel";
import { daysBetween, dayMonth, longDay, shortWeekday } from "../../model/agenda";
import type { Horizon } from "../../model/agenda";

const HORIZONS: ReadonlyArray<{ key: Horizon; label: string; glyph: GlyphTone }> = [
  { key: "late", label: "En retard", glyph: "c" },
  { key: "today", label: "Aujourd'hui", glyph: "g" },
  { key: "week", label: "Cette semaine", glyph: "n" },
];

function glyphOf(item: AgendaItem): GlyphTone {
  return Statuses.find((status) => status.value === item.status)?.glyph ?? "n";
}

/**
 * Aujourd'hui (`screens/01-today-light.png`) : un bloc de 1 180 px maximum centré dans le
 * panneau — colonne des échéances en trois horizons, colonne Situation de 290 px.
 *
 * Contrat clavier : `↑ ↓` parcourent les échéances, `⏎` fait la relance focalisée (ou
 * ouvre la candidature d'un entretien), `R` la reporte, `N` crée une candidature.
 */
export function TodayPage() {
  const vm = useTodayViewModel();
  const navigate = useNavigate();
  const [focusId, setFocusId] = useState<string | null>(null);
  const [postpone, setPostpone] = useState<FollowUp | null>(null);

  const ordered = HORIZONS.flatMap((horizon) => vm.groups[horizon.key]);
  const focused = ordered.find((item) => item.id === focusId) ?? null;
  const late = vm.groups.late.length;
  const todays = vm.groups.today.length;
  const week = vm.groups.week.length;
  const nextInterview = [...vm.groups.today, ...vm.groups.week].find((item) => item.kind === "interview");

  const openPostpone = async (item: AgendaItem) => {
    const followUp = await vm.loadFollowUp(item);
    if (followUp) setPostpone(followUp);
  };

  const act = (item: AgendaItem) => {
    if (item.kind === "follow_up") void vm.markDone(item.id);
    else void navigate(applicationsPath({ id: item.application_id }));
  };

  useShortcut("n", () => void navigate(applicationsPath({ create: true })));
  useShortcut("r", () => {
    const target = focused ?? ordered.find((item) => item.kind === "follow_up");
    if (target?.kind === "follow_up") void openPostpone(target);
  });

  useChrome({
    crumb: longDay(),
    aside: dayMonth(vm.today) + ` ${vm.today.slice(0, 4)}`,
    status:
      vm.total === 0
        ? vm.applicationsTotal === 0
          ? "aucune échéance · commencez par une candidature"
          : "aucune échéance · rien à préparer"
        : `${vm.total} échéance${vm.total > 1 ? "s" : ""}${nextInterview ? ` · prochaine ${nextInterview.date.slice(0, 10) === vm.today ? "aujourd'hui" : shortWeekday(nextInterview.date)} à ${nextInterview.date.slice(11, 16)}` : ""}`,
    keys: [
      { label: "Faire", shortcut: "enter" },
      { label: "Reporter", shortcut: "r" },
    ],
  });

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const index = focused ? ordered.indexOf(focused) : -1;
      const next = ordered[Math.min(Math.max(index + (event.key === "ArrowDown" ? 1 : -1), 0), ordered.length - 1)];
      if (next) setFocusId(next.id);
    } else if (event.key === "Enter" && focused) {
      event.preventDefault();
      act(focused);
    }
  };

  const fresh = vm.applicationsTotal === 0 && vm.total === 0;

  return (
    <div className="flex h-full min-h-0 justify-center overflow-y-auto">
      <div className="flex w-full max-w-[1180px] min-w-0">
        <div className="min-w-0 flex-1 px-[22px] pt-[18px] pb-6">
          <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
            <h1 className="serif-title text-[23px] leading-tight text-tx">{longDay()}</h1>
            <span className="font-mono text-tiny text-tx-5">
              {late} en retard · {todays} aujourd'hui · {week} cette semaine
            </span>
          </div>

          {vm.error ? (
            <div className="mt-4">
              <ErrorBanner
                message={vm.error instanceof AppError ? vm.error.message : "Les échéances n'ont pas pu être chargées."}
                onRetry={vm.reload}
              />
            </div>
          ) : vm.isLoading ? (
            <div role="status" aria-label="Chargement des échéances" className="mt-5">
              {[168, 214, 142, 196, 178].map((width, index) => (
                <div key={width} className="flex h-today-lg items-center gap-[11px]">
                  <Skeleton index={index} className="size-[11px] rounded-full" />
                  <Skeleton index={index} className="h-[9px] w-[52px]" />
                  <Skeleton index={index} className="h-[9px]" />
                  <span style={{ width }} aria-hidden />
                  <Skeleton index={index} className="ml-auto h-[22px] w-[82px] rounded-r7" />
                </div>
              ))}
            </div>
          ) : fresh ? (
            <div className="flex min-h-[min(60vh,600px)] items-center justify-center">
              <EmptyState
                icon="today"
                title="Votre suivi commence ici"
                description="Commencez par enregistrer une candidature, ou importez votre CV pour remplir votre profil."
                action={
                  <>
                    <Button variant="primary" size="empty" shortcut="n" onClick={() => void navigate(applicationsPath({ create: true }))}>
                      Nouvelle candidature
                    </Button>
                    <Button variant="secondary" size="empty" onClick={() => void navigate(PATHS.profile)}>
                      Importer un CV
                    </Button>
                  </>
                }
              />
            </div>
          ) : vm.total === 0 ? (
            <div className="flex min-h-[min(60vh,600px)] items-center justify-center">
              <EmptyState
                good
                icon="check"
                title="Rien à faire aujourd'hui"
                description="Aucune relance en retard, aucun entretien prévu cette semaine."
                action={
                  <>
                    <Button variant="primary" size="empty" shortcut="n" onClick={() => void navigate(applicationsPath({ create: true }))}>
                      Nouvelle candidature
                    </Button>
                    <Button variant="secondary" size="empty" onClick={() => void navigate(PATHS.applications)}>
                      Voir le suivi
                    </Button>
                  </>
                }
              />
            </div>
          ) : (
            <div
              role="listbox"
              aria-label="Échéances"
              tabIndex={0}
              aria-activedescendant={focused ? `echeance-${focused.id}` : undefined}
              onKeyDown={onKeyDown}
              className="mt-4 outline-none"
            >
              {HORIZONS.map((horizon) => {
                const items = vm.groups[horizon.key];
                if (items.length === 0) return null;
                return (
                  <section key={horizon.key} aria-label={`${horizon.label}, ${items.length}`}>
                    <div className="-mx-[22px] flex h-group-head items-center gap-[9px] bg-group px-[22px]">
                      <StatusGlyph tone={horizon.glyph} />
                      <span className="text-small font-medium text-tx">{horizon.label}</span>
                      <span className="font-mono text-caps text-tx-5">{items.length}</span>
                    </div>
                    {items.map((item) => (
                      <AgendaRow
                        key={item.id}
                        item={item}
                        horizon={horizon.key}
                        today={vm.today}
                        focused={item.id === focused?.id}
                        onPointer={() => setFocusId(item.id)}
                        onOpen={() => void navigate(applicationsPath({ id: item.application_id }))}
                        onAct={() => act(item)}
                        onWrite={() => void navigate(PATHS.writeLetter)}
                      />
                    ))}
                  </section>
                );
              })}
            </div>
          )}
        </div>

        <Situation vm={vm} onOpen={(id) => void navigate(applicationsPath({ id }))} />
      </div>

      <FollowUpFormModal
        open={postpone !== null}
        follow_up={postpone}
        busy={vm.isRescheduling}
        onClose={() => setPostpone(null)}
        onSubmit={(input) => (postpone ? vm.reschedule(postpone.id, input) : Promise.resolve())}
      />
    </div>
  );
}

function AgendaRow({
  item,
  horizon,
  today,
  focused,
  onPointer,
  onOpen,
  onAct,
  onWrite,
}: {
  item: AgendaItem;
  horizon: Horizon;
  today: string;
  focused: boolean;
  onPointer: () => void;
  onOpen: () => void;
  onAct: () => void;
  onWrite: () => void;
}) {
  const company = item.company_name ?? "Entreprise inconnue";
  const interview = item.kind === "interview";
  const time = interview ? item.date.slice(11, 16) : "";
  const common = cn(
    "-mx-[22px] flex cursor-default items-center gap-[11px] px-[22px] hover:bg-hover",
    focused && "row-focus",
  );

  if (horizon === "week") {
    return (
      <div id={`echeance-${item.id}`} role="option" aria-selected={focused} onPointerDown={onPointer} onClick={onOpen} className={cn(common, "h-today-sm")}>
        <StatusGlyph tone={glyphOf(item)} />
        <span className="flex-none font-mono text-tiny text-tx-5">{formatReference(item.reference_number)}</span>
        <span className="truncate text-row text-tx-2">
          {interview ? `Entretien — ${item.job_title}` : `Relance — ${company}`}
        </span>
        <span className="ml-auto flex flex-none items-center gap-2">
          <span className="truncate text-sub text-tx-4">{interview ? company : item.job_title}</span>
          <span className="w-12 text-right font-mono text-caps text-tx-5">
            {shortWeekday(item.date)}
          </span>
        </span>
      </div>
    );
  }

  const retard = daysBetween(item.date.slice(0, 10), today);
  return (
    <div id={`echeance-${item.id}`} role="option" aria-selected={focused} onPointerDown={onPointer} onClick={onOpen} className={cn(common, "h-today-lg")}>
      <StatusGlyph tone={glyphOf(item)} />
      <span className="flex-none font-mono text-tiny text-tx-5">{formatReference(item.reference_number)}</span>
      <span className="min-w-0">
        <span className="block truncate text-entry text-tx">
          {interview ? `Entretien — ${item.job_title}` : horizon === "late" ? "Relance à envoyer" : `Relance — ${company}`}
        </span>
        <span className="block truncate text-sub text-tx-4">
          {interview
            ? [company, item.detail.toLowerCase(), item.location].filter(Boolean).join(" · ")
            : `${company} · ${item.job_title}`}
        </span>
      </span>
      <span className="ml-auto flex flex-none items-center gap-2">
        <span
          className={cn(
            "inline-flex h-5 items-center rounded-r5 px-[7px] font-mono text-caps",
            horizon === "late" ? "bg-tint-c-bg text-tint-c-tx" : interview ? "bg-tint-g-bg text-tint-g-tx" : "bg-chip text-tx-3",
          )}
        >
          {horizon === "late" ? `${retard} j de retard` : interview ? time : "à envoyer"}
        </span>
        {interview ? null : (
          <>
            {horizon === "late" ? (
              <Button
                variant="secondary"
                size="bar"
                onClick={(event) => {
                  event.stopPropagation();
                  onWrite();
                }}
              >
                Rédiger
              </Button>
            ) : null}
            <Button
              variant={horizon === "late" ? "primary" : "secondary"}
              size="bar"
              onClick={(event) => {
                event.stopPropagation();
                onAct();
              }}
            >
              Faire
              <Kbd shortcut="enter" tone={horizon === "late" ? "on-accent" : "ghost"} decorative />
            </Button>
          </>
        )}
      </span>
    </div>
  );
}

const SEGMENTS = [
  { key: "pending", label: "En attente", color: "bg-st-n" },
  { key: "followed_up", label: "Relancée", color: "bg-st-a" },
  { key: "interview", label: "Entretien", color: "bg-st-g" },
  { key: "rejected", label: "Refusée", color: "bg-st-c" },
] as const;

function Situation({ vm, onOpen }: { vm: ReturnType<typeof useTodayViewModel>; onOpen: (id: string) => void }) {
  const navigate = useNavigate();
  const b = vm.breakdown;
  return (
    <aside aria-label="Situation" className="hidden w-[290px] flex-none border-l border-bd-soft px-[18px] pt-5 wide:block">
      <h2 className="caps mb-3">Situation</h2>
      {b ? (
        <>
          <div className="mb-2.5 flex gap-0.5" aria-hidden>
            {SEGMENTS.map((segment) =>
              b[segment.key] > 0 ? (
                <span key={segment.key} className={cn("h-[5px] rounded-r2", segment.color)} style={{ flex: b[segment.key] }} />
              ) : null,
            )}
          </div>
          {SEGMENTS.map((segment) => (
            <button
              key={segment.key}
              type="button"
              onClick={() => void navigate(PATHS.applications)}
              className="-mx-2 flex h-[25px] w-[calc(100%+16px)] items-center gap-[9px] rounded-r6 px-2 text-left hover:bg-hover"
            >
              <span aria-hidden className={cn("size-2 flex-none rounded-[2px]", segment.color)} />
              <span className="text-ui text-tx-3">{segment.label}</span>
              <span className="ml-auto font-mono text-tiny text-tx-4">{b[segment.key]}</span>
            </button>
          ))}
        </>
      ) : (
        <Skeleton className="h-24" />
      )}

      <div className="mt-[18px] border-t border-bd-soft pt-[15px]">
        <h2 className="caps mb-3">30 derniers jours</h2>
        {vm.metrics ? (
          <div className="flex gap-[22px]">
            {[
              { value: vm.metrics.applications, label: vm.metrics.applications > 1 ? "envoyées" : "envoyée" },
              { value: vm.metrics.responses, label: vm.metrics.responses > 1 ? "réponses" : "réponse" },
              { value: vm.metrics.interviews, label: vm.metrics.interviews > 1 ? "entretiens" : "entretien" },
            ].map((stat) => (
              <div key={stat.label}>
                <div className="serif-title text-screen leading-none text-tx">{stat.value}</div>
                <div className="mt-1 text-sub text-tx-4">{stat.label}</div>
              </div>
            ))}
          </div>
        ) : (
          <Skeleton className="h-10" />
        )}
      </div>

      <div className="mt-[18px] border-t border-bd-soft pt-[15px]">
        <h2 className="caps mb-2.5 flex items-center">
          Sans réponse
          <span className="ml-auto font-mono text-st-c">{vm.silent.length}</span>
        </h2>
        {vm.silent.length === 0 ? (
          <p className="text-small text-tx-5">Toutes vos candidatures récentes ont eu une réponse ou sont trop récentes pour relancer.</p>
        ) : (
          vm.silent.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => onOpen(item.id)}
              className="-mx-2 flex h-[27px] w-[calc(100%+16px)] items-center gap-[9px] rounded-r6 px-2 text-left hover:bg-hover"
            >
              <span className="flex-none font-mono text-caps text-tx-6">{formatReference(item.reference_number)}</span>
              <span className="truncate text-small text-tx-3">{item.company_name ?? item.job_title}</span>
              <span className="ml-auto flex-none font-mono text-caps text-st-c">{item.days} j</span>
            </button>
          ))
        )}
      </div>
    </aside>
  );
}
