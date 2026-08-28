import type { Candidature } from "../../services/candidature.service";
import { contratLabel } from "../../model/statuts";
import { joursDepuis, versDateAffichee } from "@/shared/lib/dates";
import { Icon, Tag } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/**
 * Carte d'une candidature dans le Kanban.
 *
 * Géométrie des maquettes : rayon 10 px, padding 12 px / 13 px, pastille d'initiales de
 * 26 px, intitulé 12,5 px/600 sur 1,35 d'interligne, puis une ligne d'attributs — contrat,
 * ville, ancienneté. Le survol passe le filet en accent.
 *
 * Composant **métier** : il reste dans sa feature, pas dans `shared/ui` (MIGRATION.md §35).
 */
export function CandidatureCard({
  candidature,
  selected = false,
  draggable = false,
  onSelect,
  onDragStart,
  onDragEnd,
}: {
  candidature: Candidature;
  selected?: boolean;
  draggable?: boolean;
  onSelect?: () => void;
  onDragStart?: () => void;
  onDragEnd?: () => void;
}) {
  const jours = joursDepuis(candidature.dateEnvoi);

  return (
    <article
      draggable={draggable}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      // Le clavier doit pouvoir ce que la souris peut : la carte est atteignable en
      // tabulation et s'ouvre sur Entrée, le glisser-déposer restant un raccourci.
      tabIndex={onSelect ? 0 : undefined}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" && onSelect) onSelect();
      }}
      className={cn(
        "min-w-0 rounded-tile border bg-surface px-[13px] py-3 shadow-e1",
        "transition-[border-color,background-color] duration-150",
        draggable && "cursor-grab active:cursor-grabbing",
        selected ? "border-accent bg-accent-tint" : "border-line hover:border-accent-border",
      )}
    >
      <div className="mb-[9px] flex items-start gap-[9px]">
        <span
          aria-hidden="true"
          className="flex size-[26px] flex-none items-center justify-center rounded-control bg-neutral-tint text-eyebrow font-strong tracking-normal text-ink-muted"
        >
          {initiales(candidature.entrepriseNom ?? candidature.poste)}
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-body leading-[1.35] font-semibold text-ink">{candidature.poste}</p>
          <p className="mt-0.5 truncate text-meta text-ink-faint">
            {candidature.entrepriseNom ?? "Entreprise inconnue"}
          </p>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-1.5">
        <Tag>{contratLabel(candidature.typeContrat)}</Tag>
        {candidature.entrepriseVille ? (
          <span className="truncate text-eyebrow font-normal tracking-normal text-ink-faint">
            {candidature.entrepriseVille}
          </span>
        ) : null}
        <span className="flex-1" />
        <span
          className={cn(
            "inline-flex flex-none items-center gap-1 text-eyebrow font-normal tracking-normal",
            jours >= 15 ? "text-warning" : "text-ink-faint",
          )}
          title={`Envoyée le ${versDateAffichee(candidature.dateEnvoi)}`}
        >
          <Icon name={jours >= 15 ? "schedule" : "event"} size={13} />
          {jours} j
        </span>
      </div>
    </article>
  );
}

/** Initiales de l'entreprise, pour la pastille de la carte. */
function initiales(valeur: string): string {
  return valeur
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}
