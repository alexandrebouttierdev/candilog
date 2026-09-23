import type { MouseEvent, ReactNode } from "react";
import type { Company } from "@/features/companies";
import type { Contact } from "@/features/contacts";
import type { Application } from "@/features/applications";
import { Statuses, formatReference } from "@/features/applications";
import { companySizeLabel } from "@/features/referentials";
import { openExternal } from "@/shared/services/external-link";
import { cn } from "@/shared/lib/cn";
import { Avatar, GlyphButton, StatusGlyph } from "@/shared/ui";

/** `AAAA-MM-JJ` → `JJ-MM-AA`. */
function shortYearDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}-${iso.slice(2, 4)}`;
}

/** `AAAA-MM-JJ` → `JJ-MM`. */
function shortDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}`;
}

/** Lien web affichable sans protocole : `vallis-conseil.fr`. */
function bareUrl(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/\/$/, "");
}

interface Action {
  readonly label: string;
  readonly primary?: boolean;
  readonly disabled?: boolean;
  readonly title?: string | undefined;
  readonly onClick: () => void;
}

/**
 * Inspecteur de Relations (`screens/05-companies.png`, `06-network.png`), 300 px : identité,
 * trois actions, champs, candidatures rattachées, historique et notes.
 *
 * L'historique ne montre que des faits enregistrés — envoi des candidatures, ajout de la
 * fiche — jamais une chronologie reconstituée : une date inventée se lirait comme vraie.
 */
export function RelationInspector({
  company,
  contact,
  applications,
  applicationsTotal,
  floating,
  onClose,
  onMenu,
  onEdit,
  onNewApplication,
  onFollowUp,
  onOpenApplication,
  onOpenCompany,
}: {
  company: Company | null;
  contact: Contact | null;
  applications: readonly Application[];
  applicationsTotal: number;
  floating: boolean;
  onClose: () => void;
  onMenu: (event: MouseEvent<HTMLButtonElement>) => void;
  onEdit: () => void;
  onNewApplication: () => void;
  onFollowUp: (application: Application) => void;
  onOpenApplication: (id: string) => void;
  onOpenCompany: (id: string) => void;
}) {
  if (!company && !contact) return null;

  const name = company ? company.name : `${contact?.first_name ?? ""} ${contact?.name ?? ""}`.trim();
  const sub = company
    ? [company.sector_name, company.city].filter(Boolean).join(" · ")
    : [contact?.job_title ?? contact?.tracking_role, contact?.company_name].filter(Boolean).join(" · ");
  const latest = applications[0] ?? null;

  const actions: Action[] = company
    ? [
        { label: "Nouvelle candidature", primary: true, onClick: onNewApplication },
        {
          label: "Site web",
          disabled: !company.website,
          title: company.website ? undefined : "Aucun site renseigné",
          onClick: () => void openExternal(company.website ?? ""),
        },
        { label: "Note", onClick: onEdit },
      ]
    : [
        {
          label: "Écrire",
          primary: true,
          disabled: !contact?.email,
          title: contact?.email ? undefined : "Aucun courriel renseigné",
          // `mailto:` reste un lien natif : le système ouvre le client de messagerie.
          onClick: () => {
            if (contact?.email) window.location.href = `mailto:${contact.email}`;
          },
        },
        {
          label: "Relancer",
          disabled: latest === null,
          title: latest ? undefined : "Ce contact n'est rattaché à aucune candidature",
          onClick: () => {
            if (latest) onFollowUp(latest);
          },
        },
        { label: "Note", onClick: onEdit },
      ];

  const history: Array<{ date: string; text: string }> = [
    ...applications.map((application) => ({
      date: application.sent_date,
      text: `Candidature envoyée · ${application.job_title}`,
    })),
    {
      date: (company ?? contact)?.created_at.slice(0, 10) ?? "",
      text: company ? "Entreprise ajoutée au suivi" : "Contact ajouté",
    },
  ].sort((a, b) => b.date.localeCompare(a.date));

  const notes = company ? company.notes : (contact?.notes ?? null);

  return (
    <aside
      aria-label={`Fiche ${name}`}
      className={cn(
        "flex w-inspector flex-none flex-col overflow-y-auto border-l border-bd-soft bg-panel px-4 pt-3.5",
        floating && "absolute inset-y-0 right-0 z-40 animate-pop shadow-pop",
      )}
    >
      <div className="flex items-start gap-[11px]">
        <Avatar name={name} kind={company ? "company" : "person"} size={34} />
        <div className="min-w-0 flex-1">
          <h2 className="text-lead leading-[1.3] font-medium text-tx">{name}</h2>
          {sub ? <p className="mt-0.5 text-sub text-tx-5">{sub}</p> : null}
        </div>
        <GlyphButton glyph="⋯" label={`Actions sur ${name}`} onClick={onMenu} />
        {floating ? <GlyphButton glyph="✕" label="Fermer la fiche" onClick={onClose} /> : null}
      </div>

      <div className="mt-[13px] flex flex-wrap gap-[5px]">
        {actions.map((action) => (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            title={action.title}
            onClick={action.onClick}
            className={cn(
              "inline-flex h-[25px] items-center rounded-r7 px-2.5 text-sub whitespace-nowrap disabled:cursor-default disabled:opacity-50",
              action.primary ? "bg-ac font-medium text-white" : "bg-chip text-tx-2 hover:text-tx",
            )}
          >
            {action.label}
          </button>
        ))}
      </div>

      <dl className="mt-4 grid grid-cols-[88px_minmax(0,1fr)] gap-x-1 gap-y-1.5 border-t border-bd-soft pt-3.5 text-small">
        {company ? (
          <>
            <Field label="Secteur" value={company.sector_name} />
            <Field label="Type" value={company.company_type_name} />
            <Field label="Taille" value={company.company_size === "UNKNOWN" ? null : companySizeLabel(company.company_size)} />
            <Field label="Ville" value={company.city} />
            <Field label="Adresse" value={company.address} />
            <Field
              label="Site"
              value={company.website ? bareUrl(company.website) : null}
              mono
              link={company.website ? () => void openExternal(company.website ?? "") : undefined}
            />
            <Field
              label="Dernière candidature"
              value={company.activity.last_sent_date ? shortYearDate(company.activity.last_sent_date) : null}
              mono
            />
          </>
        ) : contact ? (
          <>
            <Field label="Rôle" value={contact.tracking_role} />
            <Field label="Poste" value={contact.job_title} />
            <Field
              label="Entreprise"
              value={contact.company_name}
              link={contact.company_id ? () => onOpenCompany(contact.company_id ?? "") : undefined}
            />
            <Field
              label="Courriel"
              value={contact.email}
              mono
              href={contact.email ? `mailto:${contact.email}` : undefined}
            />
            <Field label="Téléphone" value={contact.phone} mono href={contact.phone ? `tel:${contact.phone}` : undefined} />
            <Field
              label="LinkedIn"
              value={contact.linkedin ? bareUrl(contact.linkedin) : null}
              link={contact.linkedin ? () => void openExternal(contact.linkedin ?? "") : undefined}
            />
          </>
        ) : null}
      </dl>

      <Section title={company ? "Candidatures" : "Rattaché à"} count={applicationsTotal}>
        {applications.length === 0 ? (
          <p className="text-sub leading-[1.5] text-tx-6">
            {company
              ? "Aucune candidature envoyée à cette entreprise pour l'instant."
              : "Ce contact n'est rattaché à aucune candidature."}
          </p>
        ) : (
          <ul>
            {applications.map((application) => {
              const status = Statuses.find((item) => item.value === application.status) ?? Statuses[0]!;
              return (
                <li key={application.id}>
                  <button
                    type="button"
                    onClick={() => onOpenApplication(application.id)}
                    className="-mx-2 flex h-7 w-[calc(100%+16px)] items-center gap-2.5 rounded-r6 px-2 text-left hover:bg-hover"
                  >
                    <StatusGlyph tone={status.glyph} small />
                    <span className="truncate text-small text-tx-2">{application.job_title}</span>
                    <span className="ml-auto flex-none font-mono text-[10px] text-tx-6">
                      {formatReference(application.reference_number)}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </Section>

      <Section title="Historique">
        <ul className="flex flex-col gap-2">
          {history.map((item) => (
            <li key={`${item.date}-${item.text}`} className="flex items-baseline gap-2.5 text-small">
              <span className="w-9 flex-none font-mono text-caps text-tx-6">{item.date ? shortDate(item.date) : ""}</span>
              <span className="leading-[1.45] text-tx-3">{item.text}</span>
            </li>
          ))}
        </ul>
      </Section>

      {notes ? (
        <Section title="Notes">
          <p className="text-small leading-[1.5] whitespace-pre-wrap text-tx-3">{notes}</p>
        </Section>
      ) : null}
      <div className="h-4 flex-none" />
    </aside>
  );
}

function Field({
  label,
  value,
  mono = false,
  link,
  href,
}: {
  label: string;
  value: string | null | undefined;
  mono?: boolean;
  link?: (() => void) | undefined;
  href?: string | undefined;
}) {
  const text = value ?? "—";
  const style = cn("min-w-0 truncate", mono && "font-mono text-caps", value ? "text-tx-2" : "text-tx-6");
  return (
    <>
      <dt className="text-tx-5">{label}</dt>
      <dd className={style}>
        {value && link ? (
          <button type="button" onClick={link} className={cn("max-w-full truncate text-ac-tx hover:underline", mono && "font-mono text-caps")}>
            {text}
          </button>
        ) : value && href ? (
          <a href={href} className="text-ac-tx hover:underline">
            {text}
          </a>
        ) : (
          text
        )}
      </dd>
    </>
  );
}

function Section({ title, count, children }: { title: string; count?: number; children: ReactNode }) {
  return (
    <section className="mt-4 border-t border-bd-soft pt-3">
      <h3 className="caps mb-2 flex items-center">
        {title}
        {count !== undefined ? <span className="ml-auto font-mono text-tx-7">{count}</span> : null}
      </h3>
      {children}
    </section>
  );
}
