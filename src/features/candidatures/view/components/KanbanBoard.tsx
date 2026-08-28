import { useState } from "react";
import type { Candidature, StatutCandidature } from "../../services/candidature.service";
import type { RepartitionPipeline } from "../../services/candidature.service";
import { STATUTS } from "../../model/statuts";
import { CandidatureCard } from "./CandidatureCard";
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
    <div className="min-h-0 flex-1 overflow-auto px-7 pt-[18px] pb-[26px]">
      <div className="grid min-h-full gap-3.5 [grid-template-columns:repeat(auto-fit,minmax(min(240px,100%),1fr))]">
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
                "flex min-w-0 flex-col rounded-card border bg-surface-alt",
                "transition-[border-color,background-color] duration-150",
                survolee ? "border-accent bg-accent-tint" : "border-line",
              )}
            >
              <header className="flex flex-none items-center gap-2 border-b border-line px-3.5 py-3">
                <span
                  aria-hidden="true"
                  className={cn("size-[7px] flex-none rounded-full", POINT[statut.tone])}
                />
                <h3 className="min-w-0 truncate text-body font-semibold text-ink">
                  {statut.label}
                </h3>
                <span className="tabular flex-none rounded-chip bg-neutral-tint px-1.5 py-0.5 text-meta font-semibold text-ink-faint">
                  {compteurs[statut.valeur]}
                </span>
                <span className="flex-1" />
                <button
                  type="button"
                  aria-label={`Nouvelle candidature au statut ${statut.label}`}
                  onClick={onCreate}
                  className="flex flex-none items-center text-ink-faint transition-colors duration-150 hover:text-ink"
                >
                  <Icon name="add" size={17} />
                </button>
              </header>

              <div className="flex flex-1 flex-col gap-2 p-2.5">
                {colonne.length === 0 ? (
                  <p className="rounded-tile border-[1.5px] border-dashed border-line px-3 py-[22px] text-center text-label leading-normal text-ink-faint">
                    Aucune candidature
                    <br />
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

              {/* Le pied n'apparaît que si la page chargée ne couvre pas la colonne : sans
                  lui, le compteur d'en-tête contredirait le nombre de cartes visibles. */}
              {compteurs[statut.valeur] > colonne.length ? (
                <p className="tabular flex-none border-t border-line px-3 py-2 text-center text-meta text-ink-faint">
                  {colonne.length} sur {compteurs[statut.valeur]} affichées
                </p>
              ) : null}
            </section>
          );
        })}
      </div>
    </div>
  );
}
