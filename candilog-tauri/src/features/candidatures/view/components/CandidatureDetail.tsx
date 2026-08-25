import type { Candidature } from "../../services/candidature.service";
import { STATUTS, contratLabel, statutMeta } from "../../model/statuts";
import { versDateAffichee } from "@/shared/lib/dates";
import { Button, DetailDrawer, Select, StatusPill } from "@/shared/ui";
import type { StatutCandidature } from "../../services/candidature.service";

/**
 * Panneau latéral de détail d'une candidature.
 *
 * Le statut y est **modifiable directement** : c'est l'équivalent clavier du glisser-déposer
 * du Kanban, et la vue Liste n'offrirait sinon aucun moyen de faire avancer un dossier.
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

  return (
    <DetailDrawer
      open
      title={candidature.poste}
      subtitle={candidature.entrepriseNom ?? "Entreprise inconnue"}
      onClose={onClose}
      actions={
        <>
          <Button icon="edit" onClick={onEdit} className="flex-1">
            Modifier
          </Button>
          <Button variant="danger" icon="delete" onClick={onDelete}>
            Supprimer
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-5">
        <section className="flex flex-col gap-2">
          <h3 className="text-eyebrow uppercase text-ink-faint">Statut</h3>
          <div className="flex items-center gap-2">
            <StatusPill tone={statut.tone} icon={statut.icon}>
              {statut.label}
            </StatusPill>
          </div>
          <Select
            aria-label="Changer le statut"
            value={candidature.statut}
            onChange={(event) => onStatutChange(event.target.value as StatutCandidature)}
          >
            {STATUTS.map((option) => (
              <option key={option.valeur} value={option.valeur}>
                {option.label}
              </option>
            ))}
          </Select>
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-eyebrow uppercase text-ink-faint">Candidature</h3>
          <dl className="flex flex-col gap-3 rounded-card border border-line p-3">
            <Ligne label="Contrat" valeur={contratLabel(candidature.typeContrat)} />
            <Ligne label="Envoyée le" valeur={versDateAffichee(candidature.dateEnvoi)} tabulaire />
            <Ligne label="Ville" valeur={candidature.entrepriseVille} />
            <Ligne label="Offre" valeur={candidature.lienOffre} lien />
          </dl>
        </section>

        <section className="flex flex-col gap-2">
          <h3 className="text-eyebrow uppercase text-ink-faint">Notes</h3>
          <div className="rounded-card border border-line p-3">
            {candidature.notes ? (
              <p className="text-body whitespace-pre-wrap text-ink">{candidature.notes}</p>
            ) : (
              <p className="text-meta text-ink-faint">
                Aucune note. Utilisez « Modifier » pour consigner le contexte de la candidature.
              </p>
            )}
          </div>
        </section>
      </div>
    </DetailDrawer>
  );
}

function Ligne({
  label,
  valeur,
  lien = false,
  tabulaire = false,
}: {
  label: string;
  valeur: string | null;
  lien?: boolean;
  tabulaire?: boolean;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <dt className="text-label text-ink-faint">{label}</dt>
      <dd className={`text-body break-words text-ink${tabulaire ? " tabular" : ""}`}>
        {valeur ? (
          lien ? (
            <a
              href={valeur}
              target="_blank"
              rel="noreferrer noopener"
              className="text-accent underline-offset-2 hover:underline"
            >
              {valeur}
            </a>
          ) : (
            valeur
          )
        ) : (
          <span className="text-ink-faint">Non renseigné</span>
        )}
      </dd>
    </div>
  );
}
