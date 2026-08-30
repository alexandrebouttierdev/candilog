import type { DragEvent } from "react";
import type { Application } from "../../services/applicationService";
import { daysFrom, versDateAffichee } from "@/shared/lib/dates";
import { Icon, Tag } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/** Géométrie du fantôme de glisse, calée sur la carte d'origine. */
export interface ApercuGlisse {
  readonly width: number;
  readonly height: number;
  readonly grabX: number;
  readonly grabY: number;
  readonly x: number;
  readonly y: number;
}

/**
 * Carte d'une candidature dans le Kanban.
 *
 * Géométrie des maquettes : rayon 10 px, padding 12 px / 13 px, pastille d'initiales de
 * 26 px, intitulé 12,5 px/600 sur 1,35 d'interligne, puis une ligne d'attributs — contrat,
 * ville, ancienneté. Le survol passe le filet en accent.
 *
 * Composant **métier** : il reste dans sa feature, pas dans `shared/ui` (docs/CODE_RULES.md §4).
 */
export function ApplicationCard({
  application,
  selected = false,
  checked = false,
  draggable = false,
  dragging = false,
  onSelect,
  onToggleSelect,
  onDragStart,
  onDragEnd,
}: {
  application: Application;
  selected?: boolean;
  checked?: boolean;
  draggable?: boolean;
  dragging?: boolean;
  onSelect?: () => void;
  onToggleSelect?: () => void;
  onDragStart?: (apercu: ApercuGlisse) => void;
  onDragEnd?: () => void;
}) {
  const days = daysFrom(application.sent_date);

  return (
    <article
      draggable={draggable}
      onDragStart={(event) => {
        // Un input dans un parent draggable annule le geste sur WebKit ; on le
        // laisse à la case, et on pose l'id dans dataTransfer pour le drop.
        if (event.target instanceof Element && event.target.closest("input")) {
          event.preventDefault();
          return;
        }
        event.dataTransfer.setData("text/plain", application.id);
        event.dataTransfer.effectAllowed = "move";
        masquerFantomeNatif(event);
        const rect = event.currentTarget.getBoundingClientRect();
        onDragStart?.({
          width: rect.width,
          height: rect.height,
          grabX: event.clientX - rect.left,
          grabY: event.clientY - rect.top,
          x: event.clientX,
          y: event.clientY,
        });
      }}
      onDragEnd={onDragEnd}
      // Le clavier doit pouvoir ce que la souris peut : la carte est atteignable en
      // tabulation et s'ouvre sur Entrée, le glisser-déposer restant un raccourci.
      tabIndex={onSelect ? 0 : undefined}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" && onSelect) onSelect();
      }}
      className={cn(
        "min-w-0 rounded-tile border bg-surface px-3 py-2.5",
        "transition-[border-color,background-color,box-shadow] duration-hover",
        draggable && "cursor-grab active:cursor-grabbing",
        dragging && "opacity-40",
        selected
          ? "row-selected border-accent-border"
          : "border-line hover:border-control-strong",
      )}
    >
      <div className="mb-[9px] flex items-start gap-[9px]">
        {onToggleSelect ? (
          <input
            type="checkbox"
            checked={checked}
            draggable={false}
            aria-label={`Sélectionner ${application.job_title}`}
            onClick={(event) => event.stopPropagation()}
            onPointerDown={(event) => event.stopPropagation()}
            onKeyDown={(event) => event.stopPropagation()}
            onChange={onToggleSelect}
            className="mt-1.5 flex-none"
          />
        ) : null}
        <span
          aria-hidden="true"
          className="flex size-[26px] flex-none items-center justify-center rounded-control bg-neutral-tint text-eyebrow font-strong tracking-normal text-ink-muted"
        >
          {initials(application.company_name ?? application.job_title)}
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-body leading-[1.35] font-semibold text-ink">{application.job_title}</p>
          <p className="mt-0.5 truncate text-meta text-ink-faint">
            {application.company_name ?? "Entreprise inconnue"}
          </p>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-1.5">
        <Tag>{application.contract_type_name ?? application.contract_type_code}</Tag>
        {application.effective_city ? (
          <span className="truncate text-eyebrow font-normal tracking-normal text-ink-faint">
            {application.effective_city}
          </span>
        ) : null}
        <span className="flex-1" />
        <span
          className={cn(
            "inline-flex flex-none items-center gap-1 text-eyebrow font-normal tracking-normal",
            days >= 15 ? "text-warning" : "text-ink-faint",
          )}
          title={`Envoyée le ${versDateAffichee(application.sent_date)}`}
        >
          <Icon name={days >= 15 ? "schedule" : "event"} size={13} />
          {days} j
        </span>
      </div>
    </article>
  );
}

/** Initials de l'entreprise, pour la pastille de la carte. */
function initials(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}

/**
 * WebKit photographie la couche (souvent tout le tableau) et l'affiche sous le
 * curseur : un canvas 1×1 remplace ce cliché. Le Kanban pose ensuite une copie
 * à la taille réelle de la carte.
 */
function masquerFantomeNatif(event: DragEvent<HTMLElement>): void {
  const canvas = document.createElement("canvas");
  canvas.width = 1;
  canvas.height = 1;
  canvas.style.position = "fixed";
  canvas.style.left = "-1px";
  canvas.style.top = "-1px";
  document.body.appendChild(canvas);
  event.dataTransfer.setDragImage(canvas, 0, 0);
  requestAnimationFrame(() => canvas.remove());
}
