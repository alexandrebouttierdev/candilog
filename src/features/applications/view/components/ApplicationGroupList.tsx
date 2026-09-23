import { useMemo, useRef, useState } from "react";
import type { KeyboardEvent, MouseEvent } from "react";
import type { Application, ApplicationStatus } from "@/shared/types/generated/applications";
import type { Page } from "@/shared/types/page";
import { cn } from "@/shared/lib/cn";
import { Avatar, Skeleton, StatusGlyph, StatusPill, Tag } from "@/shared/ui";
import { Statuses } from "../../model/statuses";
import { channelLabel, dueOf, formatReference, shortDate } from "../../model/presentation";

/** Point d'ancrage d'un menu : clic droit (point) ou bouton (rectangle). */
export type Anchor = { x: number; y: number } | DOMRect;

/**
 * Colonnes de la ligne selon la **largeur de la liste**, pas de la fenêtre
 * (`DESIGN_SYSTEM.md` §9.5.2, règle 4) : ouvrir l'inspecteur peut faire changer de palier.
 * De base : case, glyphe, référence, intitulé, contrat, échéance, date, avatar. À 920 px,
 * la colonne Entreprise remplace l'avatar seul ; à 1400 px s'ajoutent Ville et Canal.
 */
const GRILLE =
  "grid-cols-[12px_11px_47px_minmax(160px,1fr)_64px_92px_40px_19px] " +
  "@min-[920px]:grid-cols-[12px_11px_47px_minmax(160px,1fr)_170px_64px_92px_40px] " +
  "@min-[1400px]:grid-cols-[12px_11px_47px_minmax(160px,1fr)_170px_130px_150px_64px_92px_40px]";

const TON_ECHEANCE = { success: "success", accent: "accent", danger: "danger" } as const;

export interface ApplicationListHandlers {
  readonly onSelect: (application: Application) => void;
  readonly onToggleCheck: (id: string) => void;
  readonly onMenu: (application: Application, anchor: Anchor) => void;
  readonly onStatusMenu: (application: Application, anchor: Anchor) => void;
  readonly onKey: (application: Application, event: KeyboardEvent) => boolean;
  readonly onCreate: (status: ApplicationStatus) => void;
  readonly onShowMore: (status: ApplicationStatus) => void;
}

/**
 * Liste groupée par statut (`screens/02-applications-list.png`), l'écran de référence.
 *
 * Une seule tabulation entre dans la liste (composite ARIA), puis `↑ ↓` parcourent les
 * lignes **visibles** — groupes repliés exclus. Le focus clavier est un liseré gauche
 * `tx-6`, la sélection un liseré `ac` (`DECISIONS.md` B1). Les décomptes de groupe sont
 * ceux du filtre, pas de la page chargée.
 */
export function ApplicationGroupList({
  groups,
  collapsed,
  onToggleGroup,
  selectedId,
  checkedIds,
  loading,
  handlers,
}: {
  groups: Record<ApplicationStatus, Page<Application>>;
  collapsed: ReadonlySet<ApplicationStatus>;
  onToggleGroup: (status: ApplicationStatus) => void;
  selectedId: string | null;
  checkedIds: ReadonlySet<string>;
  loading: boolean;
  handlers: ApplicationListHandlers;
}) {
  const [focusId, setFocusId] = useState<string | null>(null);
  const container = useRef<HTMLDivElement>(null);

  const visibles = useMemo(
    () => Statuses.flatMap((status) => (collapsed.has(status.value) ? [] : groups[status.value].items)),
    [collapsed, groups],
  );
  const focused = visibles.find((item) => item.id === focusId) ?? null;

  const moveFocus = (delta: number) => {
    if (visibles.length === 0) return;
    const current = focused ?? visibles.find((item) => item.id === selectedId) ?? null;
    const index = current ? visibles.indexOf(current) : -1;
    const next = visibles[Math.min(Math.max(index + delta, 0), visibles.length - 1)]!;
    setFocusId(next.id);
    container.current
      ?.querySelector(`[data-row="${next.id}"]`)
      ?.scrollIntoView({ block: "nearest" });
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      moveFocus(event.key === "ArrowDown" ? 1 : -1);
      return;
    }
    const target = focused ?? visibles.find((item) => item.id === selectedId);
    if (!target) return;
    if (event.key === " ") {
      event.preventDefault();
      handlers.onToggleCheck(target.id);
      return;
    }
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      event.preventDefault();
      const rect = container.current
        ?.querySelector(`[data-row="${target.id}"]`)
        ?.getBoundingClientRect();
      if (rect) handlers.onMenu(target, { x: rect.left + 80, y: rect.bottom });
      return;
    }
    if (handlers.onKey(target, event)) event.preventDefault();
  };

  if (loading) {
    return (
      <div role="status" aria-label="Chargement des candidatures" className="pt-1">
        {[168, 214, 142, 196, 178, 232, 156].map((width, index) => (
          <div key={width} className="flex h-row-app items-center gap-[11px] px-3.5">
            <Skeleton index={index} className="size-3" />
            <Skeleton index={index} className="size-[11px] rounded-full" />
            <Skeleton index={index} className="h-[9px] w-10" />
            <Skeleton index={index} className="h-[9px]" />
            <span style={{ width }} aria-hidden />
          </div>
        ))}
      </div>
    );
  }

  return (
    <div
      ref={container}
      role="listbox"
      aria-label="Candidatures"
      aria-multiselectable="false"
      tabIndex={0}
      aria-activedescendant={focused ? `candidature-${focused.id}` : undefined}
      onKeyDown={onKeyDown}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) setFocusId(null);
      }}
      className="@container min-h-0 flex-1 overflow-y-auto outline-none"
    >
      {Statuses.map((status) => {
        const page = groups[status.value];
        const replie = collapsed.has(status.value);
        const restantes = page.total - page.items.length;
        return (
          <section key={status.value} aria-label={`${status.label}, ${page.total}`}>
            <div className="sticky top-0 z-[1] flex h-group-head items-center gap-[9px] bg-group px-3.5">
              <button
                type="button"
                tabIndex={-1}
                aria-expanded={!replie}
                aria-label={`${replie ? "Déplier" : "Replier"} ${status.label}`}
                onClick={() => onToggleGroup(status.value)}
                className="flex w-3 justify-center text-[8px] text-tx-5"
              >
                {replie ? "▸" : "▾"}
              </button>
              <StatusGlyph tone={status.glyph} />
              <span className="text-small font-medium text-tx">{status.label}</span>
              <span className="font-mono text-caps text-tx-5">{page.total}</span>
              <button
                type="button"
                tabIndex={-1}
                aria-label={`Ajouter une candidature ${status.label.toLowerCase()}`}
                title="Ajouter une candidature"
                onClick={() => handlers.onCreate(status.value)}
                className="ml-auto flex size-[18px] items-center justify-center rounded-r5 text-small text-tx-5 hover:bg-elev hover:text-tx-2"
              >
                +
              </button>
            </div>
            {replie
              ? null
              : page.items.map((application) => (
                  <Row
                    key={application.id}
                    application={application}
                    selected={application.id === selectedId}
                    focused={application.id === focused?.id}
                    checked={checkedIds.has(application.id)}
                    handlers={handlers}
                    onPointer={() => setFocusId(application.id)}
                  />
                ))}
            {!replie && restantes > 0 ? (
              <button
                type="button"
                tabIndex={-1}
                onClick={() => handlers.onShowMore(status.value)}
                className="flex h-row-app w-full items-center px-3.5 text-left text-small text-ac-tx hover:bg-hover"
              >
                Afficher {Math.min(restantes, 50)} de plus
                <span className="ml-2 font-mono text-caps text-tx-5">{restantes} restantes</span>
              </button>
            ) : null}
          </section>
        );
      })}
    </div>
  );
}

function Row({
  application,
  selected,
  focused,
  checked,
  handlers,
  onPointer,
}: {
  application: Application;
  selected: boolean;
  focused: boolean;
  checked: boolean;
  handlers: ApplicationListHandlers;
  onPointer: () => void;
}) {
  const status = Statuses.find((item) => item.value === application.status) ?? Statuses[0]!;
  const due = dueOf(application);
  const company = application.company_name ?? "Entreprise inconnue";
  const contract = application.contract_type_name ?? application.contract_type_code;

  const openMenu = (event: MouseEvent) => {
    event.preventDefault();
    handlers.onMenu(application, { x: event.clientX, y: event.clientY });
  };

  return (
    <div
      id={`candidature-${application.id}`}
      data-row={application.id}
      role="option"
      aria-selected={selected}
      aria-label={`${formatReference(application.reference_number)}, ${application.job_title}, ${company}, ${status.label}`}
      onClick={() => handlers.onSelect(application)}
      onContextMenu={openMenu}
      onPointerDown={onPointer}
      className={cn(
        "group grid h-row-app cursor-default items-center gap-x-[11px] px-3.5 text-row",
        GRILLE,
        selected ? "row-selected font-medium" : "text-tx-2 hover:bg-hover",
        focused && "row-focus",
      )}
    >
      <input
        type="checkbox"
        tabIndex={-1}
        checked={checked}
        aria-label={`Cocher ${formatReference(application.reference_number)}`}
        onClick={(event) => event.stopPropagation()}
        onChange={() => handlers.onToggleCheck(application.id)}
      />
      <button
        type="button"
        tabIndex={-1}
        title={`${status.label} — changer le statut`}
        aria-label={`Statut ${status.label}, changer`}
        onClick={(event) => {
          event.stopPropagation();
          handlers.onStatusMenu(application, event.currentTarget.getBoundingClientRect());
        }}
        className="flex size-[11px] items-center justify-center"
      >
        <StatusGlyph tone={status.glyph} />
      </button>
      <span className="font-mono text-tiny text-tx-5">{formatReference(application.reference_number)}</span>
      <span className="flex min-w-0 items-center gap-1.5">
        <span className="truncate">{application.job_title}</span>
        <button
          type="button"
          tabIndex={-1}
          aria-label={`Actions sur ${formatReference(application.reference_number)}`}
          onClick={(event) => {
            event.stopPropagation();
            handlers.onMenu(application, event.currentTarget.getBoundingClientRect());
          }}
          className="ml-auto hidden h-5 flex-none items-center rounded-r5 bg-chip px-1.5 text-small text-tx-4 group-hover:flex"
        >
          ⋯
        </button>
      </span>
      <span className="hidden min-w-0 items-center gap-2 @min-[920px]:flex">
        <Avatar name={company} kind="company" />
        <span className="truncate text-small font-normal text-tx-3">{company}</span>
      </span>
      <span className="hidden truncate text-small font-normal text-tx-4 @min-[1400px]:block">
        {application.effective_city ?? <span className="text-tx-7">—</span>}
      </span>
      <span className="hidden truncate text-small font-normal text-tx-4 @min-[1400px]:block">
        {channelLabel(application.channel)}
      </span>
      <span className="flex min-w-0 justify-end">
        <Tag className="max-w-full truncate">{contract}</Tag>
      </span>
      <span className="flex min-w-0 justify-end">
        {due ? (
          <span title={due.title} className="max-w-full">
            <StatusPill compact tone={TON_ECHEANCE[due.tone]} className="font-mono text-caps">
              {due.label}
            </StatusPill>
          </span>
        ) : (
          <span aria-hidden className="text-small text-tx-7">
            —
          </span>
        )}
      </span>
      <span className="text-right font-mono text-caps font-normal text-tx-5">
        {shortDate(application.sent_date)}
      </span>
      <span className="@min-[920px]:hidden">
        <Avatar name={company} kind="company" />
      </span>
    </div>
  );
}
