import { useEffect, useState } from "react";
import type { Application, ApplicationStatus } from "@/shared/types/generated/applications";
import { Statuses } from "../../model/statuses";
import { SILENCE_DAYS, dueOf, localIso } from "../../model/presentation";
import { ApplicationCard } from "./ApplicationCard";
import type { DragPreview } from "./ApplicationCard";
import type { Anchor } from "./ApplicationGroupList";
import { ColumnPager, StatusGlyph } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { Page } from "@/shared/types/page";

interface ColumnAlert {
  readonly text: string;
  readonly tone: "danger" | "success";
}

const ALERT_TINT = {
  danger: { box: "bg-tint-c-bg text-tint-c-tx", dot: "bg-st-c" },
  success: { box: "bg-tint-g-bg text-tint-g-tx", dot: "bg-st-g" },
} as const;

/**
 * Bandeau d'une colonne (`screens/19-kanban.png`) : les candidatures en attente restées
 * sans réponse au-delà de 14 jours, et les entretiens du jour. Calculé sur les cartes
 * chargées — c'est ce que l'utilisateur voit sous le bandeau.
 */
function alertOf(status: ApplicationStatus, items: readonly Application[]): ColumnAlert | null {
  if (status === "EN_ATTENTE") {
    const silent = items.filter((item) => dueOf(item)?.kind === "silence").length;
    return silent > 0
      ? { text: `${silent} sans réponse depuis plus de ${SILENCE_DAYS} jours`, tone: "danger" }
      : null;
  }
  if (status === "ENTRETIEN") {
    const day = localIso(new Date());
    const today = items
      .flatMap((item) => (item.next_interview_at?.slice(0, 10) === day ? [item.next_interview_at] : []))
      .map((at) => at.slice(11, 16))
      .sort();
    if (today.length === 0) return null;
    return {
      text:
        today.length === 1
          ? `1 entretien aujourd'hui à ${today[0]}`
          : `${today.length} entretiens aujourd'hui, le premier à ${today[0]}`,
      tone: "success",
    };
  }
  return null;
}

/**
 * Pipeline en colonnes, avec glisser-déposer entre statuts (`screens/19-kanban.png`).
 *
 * Colonnes élastiques de 236 à 340 px sur fond `bg-group`, défilement horizontal quand la
 * fenêtre est étroite ; la colonne survolée pendant une glisse prend un filet pointillé
 * `ac`. Les compteurs d'en-tête sont ceux du filtre (`total` de la page), pas des cartes
 * chargées : une colonne paginée annoncerait sinon « 25 » en contenant tout le pipeline.
 */
export function KanbanBoard({
  columns,
  selected_id,
  checkedIds,
  filtered,
  onSelect,
  onToggleSelect,
  onMenu,
  onStatusChange,
  onCreate,
  onPageChange,
}: {
  columns: Record<ApplicationStatus, Page<Application>>;
  selected_id: string | null;
  checkedIds: ReadonlySet<string>;
  /** Un filtre ou une recherche est actif : une colonne vide le dit autrement. */
  filtered: boolean;
  onSelect: (id: string) => void;
  onToggleSelect: (id: string) => void;
  onMenu: (application: Application, anchor: Anchor) => void;
  onStatusChange: (id: string, status: ApplicationStatus) => void;
  onCreate: (status: ApplicationStatus) => void;
  onPageChange: (status: ApplicationStatus, page: number) => void;
}) {
  const [dragged, setDragged] = useState<Application | null>(null);
  const [preview, setPreview] = useState<DragPreview | null>(null);
  const [dropTarget, setDropTarget] = useState<ApplicationStatus | null>(null);

  const isDragging = dragged !== null;
  useEffect(() => {
    if (!isDragging) return;
    const followPointer = (event: DragEvent) => {
      setPreview((current) =>
        current ? { ...current, x: event.clientX, y: event.clientY } : current,
      );
    };
    document.addEventListener("dragover", followPointer);
    return () => document.removeEventListener("dragover", followPointer);
  }, [isDragging]);

  const applications = Statuses.flatMap((status) => columns[status.value].items);
  const anyChecked = checkedIds.size > 0;

  return (
    <div className="flex min-h-0 flex-1 items-stretch gap-[9px] overflow-x-auto overflow-y-hidden px-3.5 pt-[11px] pb-3.5">
      {Statuses.map((status) => {
        const column = columns[status.value];
        const isDropTarget = dropTarget === status.value && dragged?.status !== status.value;
        const alert = alertOf(status.value, column.items);

        return (
          <section
            key={status.value}
            aria-label={`${status.label}, ${column.total}`}
            onDragOver={(event) => {
              // `preventDefault` est ce qui autorise le dépôt : sans lui, le navigateur
              // refuse la cible et le curseur affiche « interdit ».
              event.preventDefault();
              event.dataTransfer.dropEffect = "move";
              setDropTarget(status.value);
            }}
            onDragLeave={(event) => {
              const next = event.relatedTarget;
              if (next instanceof Node && event.currentTarget.contains(next)) return;
              setDropTarget((value) => (value === status.value ? null : value));
            }}
            onDrop={(event) => {
              event.preventDefault();
              const id = event.dataTransfer.getData("text/plain");
              const moved = applications.find((item) => item.id === id);
              if (moved && moved.status !== status.value) {
                onStatusChange(moved.id, status.value);
              }
              setDragged(null);
              setPreview(null);
              setDropTarget(null);
            }}
            className={cn(
              "flex w-[236px] max-w-[340px] min-w-[236px] flex-1 flex-col rounded-r9 transition-colors duration-100",
              isDropTarget
                ? "bg-hover outline-[1.4px] -outline-offset-1 outline-ac outline-dashed"
                : "bg-group",
            )}
          >
            <header className="flex h-8 flex-none items-center gap-[9px] px-2.5">
              <StatusGlyph tone={status.glyph} />
              <h3 className="truncate text-small font-medium text-tx">{status.label}</h3>
              <span className="font-mono text-caps text-tx-5">{column.total}</span>
              <button
                type="button"
                aria-label={`Nouvelle candidature au statut ${status.label}`}
                title="Nouvelle candidature"
                onClick={() => onCreate(status.value)}
                className="ml-auto flex size-[19px] flex-none items-center justify-center rounded-r5 text-small text-tx-6 hover:bg-elev hover:text-tx-3"
              >
                +
              </button>
            </header>

            {alert ? (
              <div
                className={cn(
                  "mx-2 mb-[7px] flex items-center gap-2 rounded-r7 px-[9px] py-1.5",
                  ALERT_TINT[alert.tone].box,
                )}
              >
                <span aria-hidden className={cn("size-[7px] flex-none rounded-full", ALERT_TINT[alert.tone].dot)} />
                <span className="min-w-0 text-tiny leading-[1.4]">{alert.text}</span>
              </div>
            ) : null}

            <div className="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto px-2 pb-2">
              {column.items.length === 0 ? (
                <p className="flex min-h-[88px] items-center justify-center rounded-r8 border-[1.4px] border-dashed border-bd-menu px-3.5 text-center text-sub text-tx-5">
                  {filtered
                    ? "Aucune candidature ne passe les filtres."
                    : status.value === "REFUS"
                      ? "Aucun refus pour l'instant."
                      : `Déposez une candidature ici pour la passer à « ${status.label.toLowerCase()} ».`}
                </p>
              ) : (
                column.items.map((application) => (
                  <ApplicationCard
                    key={application.id}
                    application={application}
                    draggable
                    dragging={dragged?.id === application.id}
                    selected={application.id === selected_id}
                    checked={checkedIds.has(application.id)}
                    showCheck={anyChecked}
                    onSelect={() => onSelect(application.id)}
                    onToggleSelect={() => onToggleSelect(application.id)}
                    onMenu={(event) => {
                      event.preventDefault();
                      onMenu(application, { x: event.clientX, y: event.clientY });
                    }}
                    onDragStart={(next) => {
                      setDragged(application);
                      setPreview(next);
                    }}
                    onDragEnd={() => {
                      setDragged(null);
                      setPreview(null);
                      setDropTarget(null);
                    }}
                  />
                ))
              )}
            </div>

            {column.total > column.page_size ? (
              <ColumnPager
                page={column.page}
                page_size={column.page_size}
                total={column.total}
                label={status.label}
                onPageChange={(page) => onPageChange(status.value, page)}
              />
            ) : null}
          </section>
        );
      })}
      {dragged && preview ? (
        <div
          aria-hidden="true"
          className="pointer-events-none fixed z-50 opacity-90 shadow-sheet"
          style={{
            left: preview.x - preview.grabX,
            top: preview.y - preview.grabY,
            width: preview.width > 0 ? preview.width : undefined,
          }}
        >
          <ApplicationCard application={dragged} />
        </div>
      ) : null}
    </div>
  );
}
