import type { Application } from "../../services/applicationService";
import { Statuses, contract_label, status_meta } from "../../model/statuses";
import { daysFrom, versDateAffichee } from "@/shared/lib/dates";
import { Button, DetailDrawer, DrawerRow, DrawerSection, Icon } from "@/shared/ui";
import type { ApplicationStatus } from "../../services/applicationService";
import { cn } from "@/shared/lib/cn";

/**
 * Panneau latéral de détail d'une candidature.
 *
 * Reprend le panneau des maquettes : encadré de statut en teinte accent, groupes
 * libellé/valeur à filet, puis notes. Le statut y est **modifiable directement** : c'est
 * l'équivalent clavier du glisser-déposer du Kanban, et la vue List n'offrirait sinon aucun
 * moyen de faire avancer un dossier.
 */
export function ApplicationDetail({
  application,
  onClose,
  onEdit,
  onDelete,
  onStatusChange,
}: {
  application: Application;
  onClose: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onStatusChange: (status: ApplicationStatus) => void;
}) {
  const status = status_meta(application.status);

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
      initials={initials(application.company_name ?? application.job_title)}
      title={application.job_title}
      subtitle={[
        application.company_name ?? "Entreprise inconnue",
        contract_label(application.contract_type),
        application.company_city,
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
          <span aria-hidden className={cn("size-1.5 rounded-full", POINT[status.tone])} />
          <span className="text-body font-mid text-ink">{status.label}</span>
          <Icon
            name="expand_more"
            size={17}
            className="pointer-events-none absolute right-2 text-ink-faint"
          />
          <select
            aria-label="Changer le statut"
            value={application.status}
            onChange={(event) => onStatusChange(event.target.value as ApplicationStatus)}
            className="absolute inset-0 cursor-pointer opacity-0"
          >
            {Statuses.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>
      </div>

      <DrawerSection icon="work" title="Candidature">
        <DrawerRow label="Contrat">{contract_label(application.contract_type)}</DrawerRow>
        <DrawerRow label="Envoyée le">
          <span className="tabular">{versDateAffichee(application.sent_date)}</span>
        </DrawerRow>
        <DrawerRow label="Ancienneté">{daysFrom(application.sent_date)} jours</DrawerRow>
        <DrawerRow label="Ville" tone={application.company_city ? undefined : "muted"}>
          {application.company_city ?? "Non renseignée"}
        </DrawerRow>
        <DrawerRow label="Offre" tone={application.job_url ? "accent" : "muted"}>
          {application.job_url ? (
            // `rel` et `target` explicites : l'application est servie depuis un contexte
            // local, un lien externe sans `noreferrer` exposerait son origine.
            <a
              href={application.job_url}
              target="_blank"
              rel="noreferrer noopener"
              className="underline-offset-2 hover:underline"
            >
              {application.job_url}
            </a>
          ) : (
            "Aucun lien"
          )}
        </DrawerRow>
      </DrawerSection>

      <DrawerSection icon="notes" title="Notes">
        {application.notes ? (
          <p className="text-body leading-normal whitespace-pre-wrap text-ink">
            {application.notes}
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

/** Initials de l'entreprise, pour la pastille d'en-tête du panneau. */
function initials(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}
