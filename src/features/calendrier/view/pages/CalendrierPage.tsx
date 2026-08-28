import { useState } from "react";
import { useCalendrierViewModel } from "../../viewmodel/useCalendrierViewModel";
import type { EvenementCalendrier } from "../../model/evenement";
import { GrilleMois } from "../components/GrilleMois";
import { EntretienFormModal } from "@/features/entretiens/view/components/EntretienFormModal";
import { RelanceFormModal } from "@/features/relances/view/components/RelanceFormModal";
import type { Entretien } from "@/features/entretiens/services/entretien.service";
import type { Relance } from "@/features/relances/services/relance.service";
import {
  Button,
  ConfirmDialog,
  ErrorBanner,
  Icon,
  PageHeader,
  Skeleton,
  StatusPill,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";

/** Ce que la page a ouvert : une modale d'entretien, de relance, ou rien. */
type Edition =
  | { genre: "aucune" }
  | { genre: "entretien"; cible: Entretien | null; jour: string | null }
  | { genre: "relance"; cible: Relance | null; jour: string | null };

/** Écran Suivi → Calendrier : entretiens et relances du mois. */
export function CalendrierPage() {
  const vm = useCalendrierViewModel();
  const [edition, setEdition] = useState<Edition>({ genre: "aucune" });
  const [aSupprimer, setASupprimer] = useState<EvenementCalendrier | null>(null);

  const fermer = () => setEdition({ genre: "aucune" });

  /** Clic sur un événement : rouvre la modale de son entité d'origine. */
  const ouvrirEvenement = (evenement: EvenementCalendrier) => {
    if (evenement.genre === "entretien") {
      setEdition({ genre: "entretien", cible: vm.entretienDe(evenement.id), jour: null });
    } else {
      setEdition({ genre: "relance", cible: vm.relanceDe(evenement.id), jour: null });
    }
  };

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="calendar_month"
        title="Calendrier"
        subtitle="Entretiens et relances"
        secondary={
          <Button
            icon="send"
            onClick={() => setEdition({ genre: "relance", cible: null, jour: null })}
          >
            Relance
          </Button>
        }
        primary={
          <Button
            variant="primary"
            icon="add"
            onClick={() => setEdition({ genre: "entretien", cible: null, jour: null })}
          >
            Nouvel entretien
          </Button>
        }
      />

      <div className="flex flex-none items-center gap-3 border-b border-line bg-surface-alt px-6 py-2.5">
        <Button icon="today" onClick={vm.revenirAujourdhui}>
          Aujourd'hui
        </Button>

        <div className="flex items-center gap-0.5">
          <NavigationMois direction="chevron_left" label="Mois précédent" onClick={() => vm.naviguer(-1)} />
          <NavigationMois direction="chevron_right" label="Mois suivant" onClick={() => vm.naviguer(1)} />
        </div>

        <h2 className="text-section text-ink capitalize">{vm.libelle}</h2>

        <div className="flex-1" />

        <StatusPill tone="success" icon="event_available">
          {`${vm.nombreEntretiens} entretien${vm.nombreEntretiens > 1 ? "s" : ""}`}
        </StatusPill>
        <StatusPill tone="warning" icon="send">
          {`${vm.nombreRelances} relance${vm.nombreRelances > 1 ? "s" : ""}`}
        </StatusPill>
      </div>

      <div className="flex min-h-0 flex-1 flex-col p-6">
        {vm.error ? (
          <ErrorBanner
            message={
              vm.error instanceof AppError
                ? vm.error.message
                : "Le calendrier n'a pas pu être chargé."
            }
            onRetry={vm.recharger}
          />
        ) : vm.isLoading ? (
          <GrilleSquelette />
        ) : (
          <GrilleMois
            cases={vm.cases}
            parJour={vm.parJour}
            onJourClick={(jour) => setEdition({ genre: "entretien", cible: null, jour })}
            onEvenementClick={ouvrirEvenement}
          />
        )}
      </div>

      <EntretienFormModal
        open={edition.genre === "entretien"}
        entretien={edition.genre === "entretien" ? edition.cible : null}
        jour={edition.genre === "entretien" ? edition.jour : null}
        busy={vm.isSaving}
        onClose={fermer}
        onSubmit={(valeurs) =>
          vm.enregistrerEntretien({
            id: edition.genre === "entretien" ? (edition.cible?.id ?? null) : null,
            input: valeurs,
          })
        }
      />

      <RelanceFormModal
        open={edition.genre === "relance"}
        relance={edition.genre === "relance" ? edition.cible : null}
        jour={edition.genre === "relance" ? edition.jour : null}
        busy={vm.isSaving}
        onClose={fermer}
        onSubmit={(valeurs) =>
          vm.enregistrerRelance({
            id: edition.genre === "relance" ? (edition.cible?.id ?? null) : null,
            input: valeurs,
          })
        }
      />

      <ConfirmDialog
        open={aSupprimer !== null}
        title={
          aSupprimer?.genre === "entretien" ? "Supprimer cet entretien ?" : "Supprimer cette relance ?"
        }
        description={`« ${aSupprimer?.libelle ?? ""} » sera définitivement retiré du calendrier.`}
        note={
          aSupprimer?.genre === "entretien"
            ? "La candidature conserve son statut « Entretien »."
            : "La candidature n'est pas modifiée."
        }
        busy={vm.isDeleting}
        onCancel={() => setASupprimer(null)}
        onConfirm={() => {
          const cible = aSupprimer;
          setASupprimer(null);
          if (!cible) return;
          if (cible.genre === "entretien") void vm.supprimerEntretien(cible.id);
          else void vm.supprimerRelance(cible.id);
        }}
      />
    </div>
  );
}

function NavigationMois({
  direction,
  label,
  onClick,
}: {
  direction: string;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className="flex size-8 items-center justify-center rounded-button border border-line bg-surface text-ink-muted transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
    >
      <Icon name={direction} size={16} />
    </button>
  );
}

/** Squelette de la grille, aux dimensions des cases réelles. */
function GrilleSquelette() {
  return (
    <div
      role="status"
      aria-label="Chargement du calendrier"
      className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6 overflow-hidden rounded-card border border-line bg-surface"
    >
      {Array.from({ length: 42 }, (_, index) => (
        <div key={index} className="flex flex-col gap-1 border-r border-b border-line p-1.5">
          <Skeleton className="size-6 flex-none rounded-pill" />
          {index % 5 === 0 ? <Skeleton className="h-3.5 w-full" /> : null}
        </div>
      ))}
    </div>
  );
}
