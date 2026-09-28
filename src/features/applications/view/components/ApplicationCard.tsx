import type { DragEvent, MouseEvent } from "react";
import type { Application } from "@/shared/types/generated/applications";
import { toDisplayDate } from "@/shared/lib/dates";
import { Avatar } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { dueOf, formatReference, shortDate } from "../../model/presentation";

/** Géométrie du fantôme de glisse, calée sur la carte d'origine. */
export interface DragPreview {
  readonly width: number;
  readonly height: number;
  readonly grabX: number;
  readonly grabY: number;
  readonly x: number;
  readonly y: number;
}

/** Teinte de la pastille d'échéance, par nature (entretien, relance, silence). */
const DUE_TINT = {
  success: "bg-tint-g-bg text-tint-g-tx",
  accent: "bg-tint-ac-bg text-tint-ac-tx",
  danger: "bg-tint-c-bg text-tint-c-tx",
} as const;

/**
 * Carte d'une candidature dans le Kanban (`screens/19-kanban.png`).
 *
 * Trois lignes : référence et échéance en pastille ; intitulé ; entreprise, contrat et
 * date d'envoi. La sélection passe la carte en `bg-sel` avec un liseré `ac` ; la case à
 * cocher n'apparaît qu'au survol ou dès qu'une carte est cochée, pour garder la sélection
 * multiple sans charger toutes les cartes.
 *
 * Composant **métier** : il reste dans sa feature, pas dans `shared/ui` (docs/CODE_RULES.md §4).
 */
export function ApplicationCard({
  application,
  selected = false,
  checked = false,
  showCheck = false,
  draggable = false,
  dragging = false,
  onSelect,
  onToggleSelect,
  onMenu,
  onDragStart,
  onDragEnd,
}: {
  application: Application;
  selected?: boolean;
  checked?: boolean;
  /** Montre la case même sans survol (une autre carte est déjà cochée). */
  showCheck?: boolean;
  draggable?: boolean;
  dragging?: boolean;
  onSelect?: () => void;
  onToggleSelect?: () => void;
  onMenu?: (event: MouseEvent) => void;
  onDragStart?: (preview: DragPreview) => void;
  onDragEnd?: () => void;
}) {
  const due = dueOf(application);
  const company = application.company_name ?? "Entreprise inconnue";
  const reference = formatReference(application.reference_number);

  return (
    <article
      draggable={draggable}
      aria-label={`${reference}, ${application.job_title}, ${company}`}
      onDragStart={(event) => {
        // Un input dans un parent draggable annule le geste sur WebKit ; on le
        // laisse à la case, et on pose l'id dans dataTransfer pour le drop.
        if (event.target instanceof Element && event.target.closest("input")) {
          event.preventDefault();
          return;
        }
        event.dataTransfer.setData("text/plain", application.id);
        event.dataTransfer.effectAllowed = "move";
        hideNativeDragGhost(event);
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
      onContextMenu={onMenu}
      // Le clavier doit pouvoir ce que la souris peut : la carte est atteignable en
      // tabulation et s'ouvre sur Entrée, le glisser-déposer restant un raccourci.
      tabIndex={onSelect ? 0 : undefined}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" && onSelect) onSelect();
      }}
      className={cn(
        "group min-w-0 rounded-r8 border px-2.5 pt-[9px] pb-2.5 outline-none",
        "transition-[border-color,background-color] duration-100 focus-visible:border-tx-6",
        draggable && "cursor-grab active:cursor-grabbing",
        dragging && "opacity-40",
        selected
          ? "border-ac bg-sel text-tx shadow-[inset_2px_0_0_var(--ac)]"
          : "border-bd-soft bg-elev text-tx-2 hover:border-bd-menu",
      )}
    >
      <div className="flex h-[18px] items-center gap-2">
        {onToggleSelect ? (
          <input
            type="checkbox"
            checked={checked}
            draggable={false}
            aria-label={`Cocher ${reference}`}
            onClick={(event) => event.stopPropagation()}
            onPointerDown={(event) => event.stopPropagation()}
            onKeyDown={(event) => event.stopPropagation()}
            onChange={onToggleSelect}
            className={cn(
              "flex-none",
              checked || showCheck
                ? ""
                : "hidden group-focus-within:block group-hover:block",
            )}
          />
        ) : null}
        <span className="font-mono text-caps text-tx-5">{reference}</span>
        {due ? (
          <span
            title={due.title}
            className={cn(
              "ml-auto inline-flex h-[18px] flex-none items-center rounded-r5 px-1.5 font-mono text-[10px]",
              DUE_TINT[due.tone],
            )}
          >
            {due.label}
          </span>
        ) : null}
      </div>
      <p
        className={cn(
          "mt-[5px] text-ui leading-[1.35] text-pretty",
          selected ? "font-medium" : "font-normal",
        )}
      >
        {application.job_title}
      </p>
      <div className="mt-[9px] flex items-center gap-2">
        <Avatar name={company} kind="company" size={18} />
        <span className="min-w-0 truncate text-[11px] text-tx-4">{company}</span>
        <span className="ml-auto inline-flex h-[17px] max-w-[45%] flex-none items-center truncate rounded-r4 bg-chip px-1.5 text-[10.5px] text-tx-4">
          {application.contract_type_name ?? application.contract_type_code}
        </span>
        <span
          className="flex-none font-mono text-[10px] text-tx-6"
          title={`Envoyée le ${toDisplayDate(application.sent_date)}`}
        >
          {shortDate(application.sent_date)}
        </span>
      </div>
    </article>
  );
}

/**
 * WebKit photographie la couche (souvent tout le tableau) et l'affiche sous le
 * curseur : un canvas 1×1 remplace ce cliché. Le Kanban pose ensuite une copie
 * à la taille réelle de la carte.
 */
function hideNativeDragGhost(event: DragEvent<HTMLElement>): void {
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
