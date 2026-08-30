import type { Application } from "../../services/applicationService";
import { Statuses, status_meta } from "../../model/statuses";
import {
  applicationTypeLabel,
  companySizeLabel,
  weeklyDurationLabel,
} from "@/features/referentials";
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
  const contrat = application.contract_type_name ?? application.contract_type_code;

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
      subtitle={[application.company_name ?? "Entreprise inconnue", contrat, application.effective_city]
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
      <InspectorRow label="Type">{applicationTypeLabel(application.application_type)}</InspectorRow>
      <InspectorRow label="Contrat">{contrat}</InspectorRow>
      <InspectorRow label="Durée hebdomadaire">
        {weeklyDurationLabel(application.weekly_work_schedule, application.weekly_hours)}
      </InspectorRow>
      <InspectorRow
        label="Domaine professionnel"
        tone={application.professional_domain_name ? undefined : "muted"}
      >
        {application.professional_domain_name ?? "Non renseigné"}
      </InspectorRow>
      <InspectorRow label="Envoyée le">
        <span className="tabular">{versDateAffichee(application.sent_date)}</span>
      </InspectorRow>
      <InspectorRow label="Ancienneté">{daysFrom(application.sent_date)} jours</InspectorRow>
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
        <InspectorSectionLabel>Entreprise</InspectorSectionLabel>
        <InspectorRow
          label="Entreprise"
          tone={application.company_name ? undefined : "muted"}
        >
          {application.company_name ?? "Inconnue"}
        </InspectorRow>
        <InspectorRow label="Taille">{companySizeLabel(application.company_size)}</InspectorRow>
        <HeritableRow
          label="Type d'entreprise"
          override={application.company_type_id}
          value={application.effective_company_type_name}
        />
        <HeritableRow
          label="Ville"
          override={application.city}
          value={application.effective_city}
        />
        <HeritableRow
          label="Adresse"
          override={application.address}
          value={application.effective_address}
        />
      </div>

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

/**
 * Rangée d'une valeur pouvant être héritée de l'entreprise.
 *
 * L'origine est signalée explicitement : une adresse affichée sans mention laisserait
 * croire qu'elle a été saisie pour cette candidature, alors qu'elle suivra l'entreprise si
 * celle-ci change.
 */
function HeritableRow({
  label,
  override,
  value,
}: {
  label: string;
  /** Surcharge propre à la candidature ; `null` signifie « héritée ». */
  override: string | null;
  /** Valeur effective, surcharge ou héritage confondus. */
  value: string | null;
}) {
  if (value === null) {
    return (
      <InspectorRow label={label} tone="muted">
        Non renseignée
      </InspectorRow>
    );
  }
  return (
    <InspectorRow label={label}>
      <span className="flex flex-col items-end">
        <span>{value}</span>
        <span className="text-meta text-ink-faint">
          {override === null ? "Valeur de l'entreprise" : "Spécifique à cette candidature"}
        </span>
      </span>
    </InspectorRow>
  );
}
