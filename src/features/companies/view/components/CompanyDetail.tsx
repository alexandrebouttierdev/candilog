import type { Company } from "../../services/companyService";
import type { Application } from "@/features/applications/services/applicationService";
import { contract_label, status_meta } from "@/features/applications/model/statuses";
import { versDateLongue } from "@/shared/lib/dates";
import {
  Card,
  CardHeader,
  CardLink,
  EmptyState,
  Icon,
  RecordAction,
  RecordHeader,
  RecordStat,
  StatusPill,
  wordInitials,
} from "@/shared/ui";

/**
 * Fiche détaillée d'une entreprise.
 *
 * Disposition des maquettes Relations : bandeau d'identité et chiffres clés, colonne
 * principale (candidatures liées, notes) et colonne latérale d'informations.
 *
 * Les champs non renseignés sont **explicitement marqués** plutôt que masqués — le guide le
 * demande, et une fiche dont les lignes disparaissent au gré du remplissage ne se lit pas
 * d'un coup d'œil.
 */
export function CompanyDetail({
  company,
  applications,
  metrics,
  onEdit,
  onDelete,
  onOuvrirApplication,
  onToutVoir,
}: {
  company: Company;
  applications: readonly Application[];
  metrics: { total: number; interview: number; pending: number };
  onEdit: () => void;
  onDelete: () => void;
  onOuvrirApplication: (application: Application) => void;
  onToutVoir: () => void;
}) {
  return (
    <div className="min-w-0 flex-1 overflow-y-auto bg-page">
      <RecordHeader
        initials={wordInitials(company.name)}
        title={company.name}
        badge={
          company.type ? (
            <StatusPill tone="accent" icon="business_center">
              {company.type}
            </StatusPill>
          ) : null
        }
        subtitle={
          [company.sector, company.city].filter(Boolean).join(" · ") ||
          "Aucune information complémentaire"
        }
        actions={
          <>
            {company.website ? (
              <RecordAction
                icon="open_in_new"
                onClick={() => window.open(company.website ?? "", "_blank", "noopener")}
              >
                Site web
              </RecordAction>
            ) : null}
            <RecordAction icon="edit" onClick={onEdit}>
              Modifier
            </RecordAction>
            <RecordAction icon="delete" onClick={onDelete}>
              Supprimer
            </RecordAction>
          </>
        }
        stats={
          <>
            <RecordStat icon="work" iconClassName="text-accent" label="Candidatures">
              {metrics.total}
            </RecordStat>
            <RecordStat icon="event_available" iconClassName="text-success" label="Entretiens">
              {metrics.interview}
            </RecordStat>
            <RecordStat icon="hourglass_top" label="En attente">
              {metrics.pending}
            </RecordStat>
          </>
        }
      />

      <div className="flex flex-wrap items-start gap-4 px-[26px] pt-5 pb-[30px]">
        <div className="flex min-w-0 flex-[1_1_420px] flex-col gap-4">
          <Card clipped>
            <CardHeader
              compact
              meta={
                applications.length > 0 ? (
                  <CardLink compact onClick={onToutVoir}>
                    Tout voir
                  </CardLink>
                ) : undefined
              }
            >
              Candidatures liées
            </CardHeader>
            {applications.length === 0 ? (
              <EmptyState
                icon="work_off"
                title="Aucune candidature"
                description="Les candidatures envoyées à cette société apparaîtront ici."
              />
            ) : (
              applications.map((application) => {
                const status = status_meta(application.status);
                return (
                  <button
                    key={application.id}
                    type="button"
                    onClick={() => onOuvrirApplication(application)}
                    className="flex w-full items-center gap-[11px] border-b border-line px-[17px] py-[11px] text-left transition-colors duration-150 last:border-b-0 hover:bg-neutral-tint"
                  >
                    <span className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-ink-muted">
                      <Icon name="work" size={16} />
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-body font-mid text-ink">
                        {application.job_title}
                      </span>
                      <span className="mt-px block truncate text-meta text-ink-faint">
                        {contract_label(application.contract_type)} · envoyée le{" "}
                        {versDateLongue(application.sent_date)}
                      </span>
                    </span>
                    <StatusPill tone={status.tone} compact>
                      {status.label}
                    </StatusPill>
                  </button>
                );
              })
            )}
          </Card>

          <Card clipped>
            <CardHeader compact>Notes</CardHeader>
            <div className="px-[17px] py-3.5">
              {company.notes ? (
                <p className="text-body leading-normal whitespace-pre-wrap text-ink">
                  {company.notes}
                </p>
              ) : (
                <p className="text-label leading-normal text-ink-faint">
                  Aucune note. Utilisez « Modifier » pour consigner le contexte, la culture ou
                  les informations utiles.
                </p>
              )}
            </div>
          </Card>
        </div>

        <Card clipped className="max-w-[360px] flex-[1_1_280px]">
          <CardHeader compact>Informations</CardHeader>
          <div className="px-[17px] pt-1 pb-3">
            <Row label="Secteur" value={company.sector} />
            <Row label="Type" value={company.type} />
            <Row label="Ville" value={company.city} />
            <Row label="Adresse" value={company.address} />
            <Row label="Site web" value={company.website} url />
            <Row label="Ajoutée le" value={versDateLongue(company.created_at, true)} />
          </div>
        </Card>
      </div>
    </div>
  );
}

/** Rangée libellé / valeur de la carte « Informations » : 9 px de padding, filet bas. */
function Row({
  label,
  value,
  url = false,
}: {
  label: string;
  value: string | null;
  url?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-3.5 border-b border-line py-[9px] last:border-b-0">
      <span className="flex-none text-note text-ink-faint">{label}</span>
      <span className="min-w-0 flex-1 truncate text-right text-body font-medium text-ink">
        {value ? (
          url ? (
            // `rel` et `target` explicites : l'application est servie depuis un contexte
            // local, un lien externe sans `noreferrer` exposerait son origine.
            <a
              href={value}
              target="_blank"
              rel="noreferrer noopener"
              className="text-accent underline-offset-2 hover:underline"
            >
              {value}
            </a>
          ) : (
            value
          )
        ) : (
          <span className="font-normal text-ink-faint">Non renseigné</span>
        )}
      </span>
    </div>
  );
}
