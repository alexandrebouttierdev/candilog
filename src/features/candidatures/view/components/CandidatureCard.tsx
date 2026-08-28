import type { Candidature } from "../../services/candidature.service";
import { contratLabel } from "../../model/statuts";
import { versDateAffichee } from "@/shared/lib/dates";
import { Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/**
 * Carte d'une candidature, partagée par le Kanban et par le panneau de détail.
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
        "flex flex-col gap-1.5 rounded-card border bg-surface p-3 shadow-e1",
        "transition-[border-color,background-color] duration-150",
        draggable && "cursor-grab active:cursor-grabbing",
        selected ? "border-accent bg-accent-tint" : "border-line hover:border-line-strong",
      )}
    >
      <p className="truncate text-body font-medium text-ink">{candidature.poste}</p>
      <p className="truncate text-meta text-ink-muted">
        {candidature.entrepriseNom ?? "Entreprise inconnue"}
        {candidature.entrepriseVille ? ` · ${candidature.entrepriseVille}` : ""}
      </p>
      <div className="mt-0.5 flex items-center gap-2 text-meta text-ink-faint">
        <span className="rounded-pill bg-neutral-tint px-1.5 py-px">
          {contratLabel(candidature.typeContrat)}
        </span>
        <span className="tabular flex items-center gap-1">
          <Icon name="event" size={12} />
          {versDateAffichee(candidature.dateEnvoi)}
        </span>
      </div>
    </article>
  );
}
