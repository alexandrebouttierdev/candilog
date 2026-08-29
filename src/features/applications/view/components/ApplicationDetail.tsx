import type { Application } from "../../services/applicationService";
import { Statuses, contract_label, status_meta } from "../../model/statuses";
import { daysFrom, versDateAffichee } from "@/shared/lib/dates";
import {
  Button,
  Icon,
  IconButton,
  Inspector,
  InspectorRow,
  InspectorSectionLabel,
} from "@/shared/ui";
import type { ApplicationStatus } from "../../services/applicationService";
import { cn } from "@/shared/lib/cn";

/** Panneau latéral de détail d'une candidature. */
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
    <Inspector
      open
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
          <Button variant="secondary" icon="edit" onClick={onEdit}>
            Modifier
          </Button>
          <IconButton icon="delete" label="Supprimer" onClick={onDelete} className="text-danger" />
        </>
      }
      headerExtra={
        <div className="flex items-center gap-2">
          <span aria-hidden className={cn("size-1.5 rounded-full", POINT[status.tone])} />
          <div className="relative flex h-[29px] flex-1 items-center gap-2 rounded-control border border-control bg-surface pr-7 pl-2.5">
            <span className="text-body text-ink">{status.label}</span>
            <Icon
              name="expand_more"
              size={15}
              className="pointer-events-none absolute right-1.5 text-ink-faint"
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
      }
    >
      <InspectorSectionLabel>Candidature</InspectorSectionLabel>
      <InspectorRow label="Contrat">{contract_label(application.contract_type)}</InspectorRow>
      <InspectorRow label="Envoyée le">
        <span className="tabular">{versDateAffichee(application.sent_date)}</span>
      </InspectorRow>
      <InspectorRow label="Ancienneté">{daysFrom(application.sent_date)} jours</InspectorRow>
      <InspectorRow label="Ville" tone={application.company_city ? undefined : "muted"}>
        {application.company_city ?? "Non renseignée"}
      </InspectorRow>
      <InspectorRow label="Offre" tone={application.job_url ? "accent" : "muted"}>
        {application.job_url ? (
          <a
            href={application.job_url}
            target="_blank"
            rel="noreferrer noopener"
            className="underline-offset-2 hover:underline"
          >
            Ouvrir l'offre
          </a>
        ) : (
          "Aucun lien"
        )}
      </InspectorRow>

      <div className="mt-4 border-t border-line-soft pt-3">
        <InspectorSectionLabel>Notes</InspectorSectionLabel>
        {application.notes ? (
          <p className="text-body leading-normal whitespace-pre-wrap text-ink-strong">
            {application.notes}
          </p>
        ) : (
          <p className="text-note text-ink-faint">
            Aucune note. Utilisez « Modifier » pour consigner le contexte.
          </p>
        )}
      </div>
    </Inspector>
  );
}
