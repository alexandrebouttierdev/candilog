import type { Candidature } from "../../services/candidature.service";
import { STATUTS, contratLabel, statutMeta } from "../../model/statuts";
import { joursDepuis, versDateAffichee } from "@/shared/lib/dates";
import { Button, DetailDrawer, DrawerRow, DrawerSection, Icon } from "@/shared/ui";
import type { StatutCandidature } from "../../services/candidature.service";
import { cn } from "@/shared/lib/cn";

/**
 * Panneau latéral de détail d'une candidature.
 *
 * Reprend le panneau des maquettes : encadré de statut en teinte accent, groupes
 * libellé/valeur à filet, puis notes. Le statut y est **modifiable directement** : c'est
 * l'équivalent clavier du glisser-déposer du Kanban, et la vue Liste n'offrirait sinon aucun
 * moyen de faire avancer un dossier.
 */
export function CandidatureDetail({
  candidature,
  onClose,
  onEdit,
  onDelete,
  onStatutChange,
}: {
  candidature: Candidature;
  onClose: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onStatutChange: (statut: StatutCandidature) => void;
}) {
  const statut = statutMeta(candidature.statut);

  const POINT: Record<string, string> = {
    neutral: "bg-ink-faint",
    accent: "bg-accent",
    success: "bg-success",
    warning: "bg-warning",
    danger: "bg-danger",
  };

  return (
    <DetailDrawer
      open
      initials={initiales(candidature.entrepriseNom ?? candidature.poste)}
      title={candidature.poste}
      subtitle={[
        candidature.entrepriseNom ?? "Entreprise inconnue",
        contratLabel(candidature.typeContrat),
        candidature.entrepriseVille,
      ]
        .filter(Boolean)
        .join(" · ")}
      onClose={onClose}
      actions={
        <>
          <Button variant="danger" icon="delete" onClick={onDelete}>
            Supprimer
          </Button>
          <span className="flex-1" />
          <Button variant="primary" icon="edit" onClick={onEdit}>
            Modifier
          </Button>
        </>
      }
    >
      <div className="mb-[18px] flex items-center gap-3 rounded-tile border border-accent-border bg-accent-tint px-3.5 py-3">
        <div className="min-w-0 flex-1">
          <p className="text-note font-semibold text-accent">Statut de la candidature</p>
          <p className="mt-0.5 text-meta text-ink-muted">Choisissez l’étape du suivi</p>
        </div>
        <div className="relative flex h-8 flex-none items-center gap-2 rounded-button border border-line bg-surface pr-8 pl-[11px]">
          <span aria-hidden className={cn("size-1.5 rounded-full", POINT[statut.tone])} />
          <span className="text-body font-mid text-ink">{statut.label}</span>
          <Icon
            name="expand_more"
            size={17}
            className="pointer-events-none absolute right-2 text-ink-faint"
          />
          <select
            aria-label="Changer le statut"
            value={candidature.statut}
            onChange={(event) => onStatutChange(event.target.value as StatutCandidature)}
            className="absolute inset-0 cursor-pointer opacity-0"
          >
            {STATUTS.map((option) => (
              <option key={option.valeur} value={option.valeur}>
                {option.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      <DrawerSection icon="work" title="Candidature">
        <DrawerRow label="Contrat">{contratLabel(candidature.typeContrat)}</DrawerRow>
        <DrawerRow label="Envoyée le">
          <span className="tabular">{versDateAffichee(candidature.dateEnvoi)}</span>
        </DrawerRow>
        <DrawerRow label="Ancienneté">{joursDepuis(candidature.dateEnvoi)} jours</DrawerRow>
        <DrawerRow label="Ville" tone={candidature.entrepriseVille ? undefined : "muted"}>
          {candidature.entrepriseVille ?? "Non renseignée"}
        </DrawerRow>
        <DrawerRow label="Offre" tone={candidature.lienOffre ? "accent" : "muted"}>
          {candidature.lienOffre ? (
            // `rel` et `target` explicites : l'application est servie depuis un contexte
            // local, un lien externe sans `noreferrer` exposerait son origine.
            <a
              href={candidature.lienOffre}
              target="_blank"
              rel="noreferrer noopener"
              className="underline-offset-2 hover:underline"
            >
              {candidature.lienOffre}
            </a>
          ) : (
            "Aucun lien"
          )}
        </DrawerRow>
      </DrawerSection>

      <DrawerSection icon="notes" title="Notes">
        {candidature.notes ? (
          <p className="text-body leading-normal whitespace-pre-wrap text-ink">
            {candidature.notes}
          </p>
        ) : (
          <p className="text-label leading-normal text-ink-faint">
            Aucune note. Utilisez « Modifier » pour consigner le contexte de la candidature.
          </p>
        )}
      </DrawerSection>
    </DetailDrawer>
  );
}

/** Initiales de l'entreprise, pour la pastille d'en-tête du panneau. */
function initiales(valeur: string): string {
  return valeur
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}
