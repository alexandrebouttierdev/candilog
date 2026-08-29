import { useEffect, useState } from "react";
import type { Application, ApplicationStatus } from "../../services/applicationService";
import type { PipelineBreakdown } from "../../services/applicationService";
import { Statuses } from "../../model/statuses";
import { ApplicationCard } from "./ApplicationCard";
import type { ApercuGlisse } from "./ApplicationCard";
import { Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { Tone } from "@/shared/ui";

/**
 * Couleur de la pastille d'en-tête de colonne, par tonalité.
 *
 * Une table statique et non une interpolation `bg-${tone}` : Tailwind analyse les sources à
 * la compilation et ne génère que les classes qu'il y trouve littéralement — une classe
 * construite à l'exécution n'existerait tout simplement pas dans la feuille de style.
 */
const POINT: Record<Tone, string> = {
  neutral: "bg-ink-faint",
  accent: "bg-accent",
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-danger",
};

/**
 * Pipeline en colonnes, avec glisser-déposer entre statuts.
 *
 * Géométrie des maquettes : grille auto-ajustée de colonnes d'au moins 240 px sur fond
 * `surface-alt`, en-tête de 12 px / 14 px portant un point de couleur, le libellé et le
 * compteur, cartes espacées de 8 px dans une gouttière de 10 px.
 *
 * Les compteurs d'en-tête viennent de `repartition`, calculée par `SQLite` sur tout le
 * filtre — et non de la longueur des colonnes affichées, qui ne compterait que la page
 * chargée. Une colonne annoncerait sinon « 3 » en contenant tout le pipeline.
 */
export function KanbanBoard({
  applications,
  breakdown,
  selected_id,
  checkedIds,
  onSelect,
  onToggleSelect,
  onStatusChange,
  onCreate,
}: {
  applications: readonly Application[];
  breakdown: PipelineBreakdown;
  selected_id: string | null;
  checkedIds: ReadonlySet<string>;
  onSelect: (id: string) => void;
  onToggleSelect: (id: string) => void;
  onStatusChange: (id: string, status: ApplicationStatus) => void;
  onCreate: () => void;
}) {
  const [glissee, setGlissee] = useState<Application | null>(null);
  const [apercu, setApercu] = useState<ApercuGlisse | null>(null);
  const [cible, setCible] = useState<ApplicationStatus | null>(null);

  const enGlisse = glissee !== null;
  useEffect(() => {
    if (!enGlisse) return;
    const suivre = (event: DragEvent) => {
      setApercu((actuel) =>
        actuel ? { ...actuel, x: event.clientX, y: event.clientY } : actuel,
      );
    };
    document.addEventListener("dragover", suivre);
    return () => document.removeEventListener("dragover", suivre);
  }, [enGlisse]);

  const compteurs: Record<ApplicationStatus, number> = {
    EN_ATTENTE: breakdown.pending,
    RELANCEE: breakdown.followed_up,
    ENTRETIEN: breakdown.interview,
    REFUS: breakdown.rejected,
  };

  return (
    <div className="min-h-0 flex-1 overflow-auto px-7 pt-[18px] pb-[26px]">
      <div className="grid min-h-full gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(240px,100%),1fr))]">
        {Statuses.map((status) => {
          const column = applications.filter((item) => item.status === status.value);
          const survolee = cible === status.value && glissee?.status !== status.value;

          return (
            <section
              key={status.value}
              onDragOver={(event) => {
                // `preventDefault` est ce qui autorise le dépôt : sans lui, le navigateur
                // refuse la cible et le curseur affiche « interdit ».
                event.preventDefault();
                event.dataTransfer.dropEffect = "move";
                setCible(status.value);
              }}
              onDragLeave={(event) => {
                const suivant = event.relatedTarget;
                if (suivant instanceof Node && event.currentTarget.contains(suivant)) return;
                setCible((value) => (value === status.value ? null : value));
              }}
              onDrop={(event) => {
                event.preventDefault();
                const id = event.dataTransfer.getData("text/plain");
                const dragged = applications.find((item) => item.id === id);
                if (dragged && dragged.status !== status.value) {
                  onStatusChange(dragged.id, status.value);
                }
                setGlissee(null);
                setApercu(null);
                setCible(null);
              }}
              className={cn(
                "flex min-w-0 flex-col rounded-card border bg-surface-alt",
                "transition-[border-color,background-color] duration-150",
                survolee ? "border-accent bg-accent-tint" : "border-line",
              )}
            >
              <header className="flex flex-none items-center gap-2 border-b border-line px-3.5 py-3">
                <span
                  aria-hidden="true"
                  className={cn("size-[7px] flex-none rounded-full", POINT[status.tone])}
                />
                <h3 className="min-w-0 truncate text-body font-semibold text-ink">
                  {status.label}
                </h3>
                <span className="tabular flex-none rounded-chip bg-neutral-tint px-1.5 py-0.5 text-meta font-semibold text-ink-faint">
                  {compteurs[status.value]}
                </span>
                <span className="flex-1" />
                <button
                  type="button"
                  aria-label={`Nouvelle candidature au statut ${status.label}`}
                  onClick={onCreate}
                  className="flex flex-none items-center text-ink-faint transition-colors duration-150 hover:text-ink"
                >
                  <Icon name="add" size={17} />
                </button>
              </header>

              <div className="flex flex-1 flex-col gap-2 p-2.5">
                {column.length === 0 ? (
                  <p className="rounded-tile border-[1.5px] border-dashed border-line px-3 py-[22px] text-center text-label leading-normal text-ink-faint">
                    Aucune candidature
                    <br />
                    Glissez une carte ici
                  </p>
                ) : (
                  column.map((application) => (
                    <ApplicationCard
                      key={application.id}
                      application={application}
                      draggable
                      dragging={glissee?.id === application.id}
                      selected={application.id === selected_id}
                      checked={checkedIds.has(application.id)}
                      onSelect={() => onSelect(application.id)}
                      onToggleSelect={() => onToggleSelect(application.id)}
                      onDragStart={(suivant) => {
                        setGlissee(application);
                        setApercu(suivant);
                      }}
                      onDragEnd={() => {
                        setGlissee(null);
                        setApercu(null);
                        setCible(null);
                      }}
                    />
                  ))
                )}
              </div>

              {/* Le pied n'apparaît que si la page chargée ne couvre pas la colonne : sans
                  lui, le compteur d'en-tête contredirait le nombre de cartes visibles. */}
              {compteurs[status.value] > column.length ? (
                <p className="tabular flex-none border-t border-line px-3 py-2 text-center text-meta text-ink-faint">
                  {column.length} sur {compteurs[status.value]} affichEs
                </p>
              ) : null}
            </section>
          );
        })}
      </div>
      {glissee && apercu ? (
        <div
          aria-hidden="true"
          className="pointer-events-none fixed z-50 opacity-90 shadow-e2"
          style={{
            left: apercu.x - apercu.grabX,
            top: apercu.y - apercu.grabY,
            width: apercu.width > 0 ? apercu.width : undefined,
          }}
        >
          <ApplicationCard application={glissee} />
        </div>
      ) : null}
    </div>
  );
}
