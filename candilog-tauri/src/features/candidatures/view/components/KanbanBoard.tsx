import { useState } from "react";
import type { Candidature, StatutCandidature } from "../../services/candidature.service";
import type { RepartitionPipeline } from "../../services/candidature.service";
import { STATUTS } from "../../model/statuts";
import { CandidatureCard } from "./CandidatureCard";
import { Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { Tone } from "@/shared/ui";

/**
 * Classes de la pastille d'en-tête, par tonalité.
 *
 * Une table statique et non une interpolation `bg-${tone}-tint` : Tailwind analyse les
 * sources à la compilation et ne génère que les classes qu'il y trouve littéralement — une
 * classe construite à l'exécution n'existerait tout simplement pas dans la feuille de style.
 */
const PASTILLE: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Pipeline en colonnes, avec glisser-déposer entre statuts.
 *
 * Les compteurs d'en-tête viennent de `repartition`, calculée par `SQLite` sur tout le
 * filtre — et non de la longueur des colonnes affichées, qui ne compterait que la page
 * chargée. Une colonne annoncerait sinon « 3 » en contenant tout le pipeline.
 */
export function KanbanBoard({
  candidatures,
  repartition,
  selectedId,
  onSelect,
  onStatutChange,
  onCreate,
}: {
  candidatures: readonly Candidature[];
  repartition: RepartitionPipeline;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onStatutChange: (id: string, statut: StatutCandidature) => void;
  onCreate: () => void;
}) {
  const [glissee, setGlissee] = useState<Candidature | null>(null);
  const [cible, setCible] = useState<StatutCandidature | null>(null);

  const compteurs: Record<StatutCandidature, number> = {
    EN_ATTENTE: repartition.enAttente,
    RELANCEE: repartition.relancee,
    ENTRETIEN: repartition.entretien,
    REFUS: repartition.refus,
  };

  return (
    <div className="flex h-full gap-3.5 overflow-x-auto p-6">
      {STATUTS.map((statut) => {
        const colonne = candidatures.filter((item) => item.statut === statut.valeur);
        const survolee = cible === statut.valeur && glissee?.statut !== statut.valeur;

        return (
          <section
            key={statut.valeur}
            onDragOver={(event) => {
              // `preventDefault` est ce qui autorise le dépôt : sans lui, le navigateur
              // refuse la cible et le curseur affiche « interdit ».
              event.preventDefault();
              setCible(statut.valeur);
            }}
            onDragLeave={() => setCible((valeur) => (valeur === statut.valeur ? null : valeur))}
            onDrop={() => {
              if (glissee && glissee.statut !== statut.valeur) {
                onStatutChange(glissee.id, statut.valeur);
              }
              setGlissee(null);
              setCible(null);
            }}
            className={cn(
              "flex w-[280px] flex-none flex-col rounded-card border bg-surface-alt",
              "transition-[border-color,background-color] duration-150",
              survolee ? "border-accent bg-accent-tint" : "border-line",
            )}
          >
            <header className="flex flex-none items-center gap-2 border-b border-line px-3 py-2.5">
              <span
                aria-hidden="true"
                className={cn(
                  "flex size-5 flex-none items-center justify-center rounded-pill",
                  PASTILLE[statut.tone],
                )}
              >
                <Icon name={statut.icon} size={12} />
              </span>
              <h3 className="min-w-0 flex-1 truncate text-section text-ink">{statut.label}</h3>
              <span className="tabular rounded-pill bg-neutral-tint px-1.5 py-px text-meta text-ink-muted">
                {compteurs[statut.valeur]}
              </span>
              <button
                type="button"
                aria-label={`Nouvelle candidature au statut ${statut.label}`}
                onClick={onCreate}
                className="flex size-6 items-center justify-center rounded-button text-ink-faint transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
              >
                <Icon name="add" size={15} />
              </button>
            </header>

            <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-2">
              {colonne.length === 0 ? (
                <p className="rounded-card border border-dashed border-line px-3 py-6 text-center text-meta text-ink-faint">
                  Glissez une carte ici
                </p>
              ) : (
                colonne.map((candidature) => (
                  <CandidatureCard
                    key={candidature.id}
                    candidature={candidature}
                    draggable
                    selected={candidature.id === selectedId}
                    onSelect={() => onSelect(candidature.id)}
                    onDragStart={() => setGlissee(candidature)}
                    onDragEnd={() => {
                      setGlissee(null);
                      setCible(null);
                    }}
                  />
                ))
              )}
            </div>

            {compteurs[statut.valeur] > colonne.length ? (
              <footer className="tabular flex-none border-t border-line px-3 py-2 text-meta text-ink-faint">
                {colonne.length} sur {compteurs[statut.valeur]} affichées
              </footer>
            ) : null}
          </section>
        );
      })}
    </div>
  );
}
