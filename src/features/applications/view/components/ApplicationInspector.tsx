import type { ReactNode } from "react";
import type { Application } from "@/shared/types/generated/applications";
import { companySizeLabel, weeklyDurationLabel } from "@/features/referentials";
import { daysFrom, toDisplayDate } from "@/shared/lib/dates";
import { openExternal } from "@/shared/services/external-link";
import { cn } from "@/shared/lib/cn";
import { GlyphButton, Kbd, StatusGlyph } from "@/shared/ui";
import { Statuses, status_meta } from "../../model/statuses";
import { channelLabel, formatReference, shortDate } from "../../model/presentation";
import { useStatusHistory } from "../../viewmodel/useApplicationDetails";
import type { Anchor } from "./ApplicationGroupList";

/** Date ISO → `JJ-MM-AA`, la notation des champs de l'inspecteur. */
function shortYearDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}-${iso.slice(2, 4)}`;
}

/**
 * Inspecteur d'une candidature (`screens/03-application-detail.png`) : 300 px, identité,
 * champs, puis Documents, Activité et Historique des statuts (`COMPONENTS.md` §13).
 *
 * Toutes les données de la fiche v1 y restent : valeurs héritées de l'entreprise signalées
 * comme telles, taille, adresse, domaine, régime horaire, lien de l'offre, notes. Une
 * section vide dit pourquoi elle est vide.
 */
export function ApplicationInspector({
  application,
  floating = false,
  onClose,
  onEdit,
  onMenu,
  onStatusMenu,
  onScheduleFollowUp,
}: {
  application: Application;
  /** Sous 1060 px : panneau superposé à droite (`DECISIONS.md` D6). */
  floating?: boolean;
  onClose: () => void;
  onEdit: () => void;
  onMenu: (anchor: Anchor) => void;
  onStatusMenu: (anchor: Anchor) => void;
  onScheduleFollowUp: () => void;
}) {
  const status = status_meta(application.status);
  const history = useStatusHistory(application.id);
  const reference = formatReference(application.reference_number);
  const contract = application.contract_type_name ?? application.contract_type_code;
  const hours = application.weekly_hours
    ? `${String(application.weekly_hours).replace(".", ",")} h`
    : null;

  const activity: Array<{ date: string; text: string }> = [];
  if (application.next_interview_at) {
    const time = application.next_interview_at.slice(11, 16);
    activity.push({
      date: shortDate(application.next_interview_at.slice(0, 10)),
      text: `Entretien prévu${time ? ` à ${time}` : ""}`,
    });
  }
  if (application.next_follow_up_date) {
    activity.push({ date: shortDate(application.next_follow_up_date), text: "Relance programmée" });
  }
  activity.push({ date: shortDate(application.sent_date), text: "Candidature envoyée" });

  return (
    <aside
      aria-label={`Fiche ${reference}`}
      className={cn(
        "flex w-inspector flex-none flex-col overflow-y-auto border-l border-bd-soft bg-panel px-4 pt-3.5",
        floating && "absolute inset-y-0 right-0 z-40 animate-pop shadow-pop",
      )}
    >
      <div className="flex items-center gap-1.5">
        <span className="font-mono text-caps text-tx-5">{reference}</span>
        <span className="ml-auto flex gap-1">
          <GlyphButton glyph="✎" label="Modifier la fiche" onClick={onEdit} />
          <GlyphButton
            glyph="⋯"
            label={`Actions sur ${reference}`}
            onClick={(event) => onMenu(event.currentTarget.getBoundingClientRect())}
          />
          <GlyphButton glyph="✕" label="Fermer la fiche" onClick={onClose} />
        </span>
      </div>
      <h2 className="mt-2 text-lead font-medium text-tx">{application.job_title}</h2>
      <p className="mt-0.5 text-sub text-tx-5">
        {[application.company_name ?? "Entreprise inconnue", application.effective_city]
          .filter(Boolean)
          .join(" · ")}
      </p>

      <dl className="mt-3.5 grid grid-cols-[82px_minmax(0,1fr)] items-center gap-x-2 gap-y-1.5 text-small">
        <Field label="Statut">
          <button
            type="button"
            onClick={(event) => onStatusMenu(event.currentTarget.getBoundingClientRect())}
            aria-label={`Statut ${status.label}, changer`}
            className="flex h-6 w-full items-center gap-2 rounded-r6 bg-chip px-2 text-tx-2 hover:bg-elev"
          >
            <StatusGlyph tone={status.glyph} />
            {status.label}
            <Kbd shortcut="s" tone="ghost" decorative className="ml-auto" />
          </button>
        </Field>
        <Field label="Contrat">{[contract, hours].filter(Boolean).join(" · ")}</Field>
        <Field label="Trouvée via">{channelLabel(application.channel)}</Field>
        <Field label="Domaine" muted={!application.professional_domain_name}>
          {application.professional_domain_name ?? "Non renseigné"}
        </Field>
        <Field label="Horaires" muted={application.weekly_work_schedule === "UNSPECIFIED"}>
          {weeklyDurationLabel(application.weekly_work_schedule, application.weekly_hours)}
        </Field>
        <Field label="Envoyée">
          <span className="font-mono text-caps">
            {shortYearDate(application.sent_date)} · {daysFrom(application.sent_date)} j
          </span>
        </Field>
        <Field label="Relance">
          <button
            type="button"
            onClick={onScheduleFollowUp}
            className="flex w-full items-center text-left text-ac-tx hover:underline"
          >
            {application.next_follow_up_date ? (
              <span className="font-mono text-caps">{shortYearDate(application.next_follow_up_date)}</span>
            ) : (
              "Programmer"
            )}
            <Kbd shortcut="r" tone="ghost" decorative className="ml-auto" />
          </button>
        </Field>
        <Inherited label="Ville" own={application.city} value={application.effective_city} />
        <Inherited
          label="Type"
          own={application.company_type_id}
          value={application.effective_company_type_name}
        />
        <Inherited label="Adresse" own={application.address} value={application.effective_address} />
        <Field label="Taille">{companySizeLabel(application.company_size)}</Field>
        <Field label="Offre" muted={!application.job_url}>
          {application.job_url ? (
            <button
              type="button"
              onClick={() => void openExternal(application.job_url ?? "")}
              className="text-ac-tx underline-offset-2 hover:underline"
            >
              Ouvrir l'offre
            </button>
          ) : (
            "Aucun lien"
          )}
        </Field>
      </dl>

      {application.notes ? (
        <Section title="Notes">
          <p className="text-small leading-[1.5] whitespace-pre-wrap text-tx-3">{application.notes}</p>
        </Section>
      ) : null}

      <Section title="Documents">
        <p className="text-small text-tx-5">
          Les CV et lettres générés pour cette candidature apparaîtront ici.
        </p>
      </Section>

      <Section title="Activité">
        <ul className="flex flex-col gap-1.5">
          {activity.map((item) => (
            <li key={`${item.date}-${item.text}`} className="flex items-baseline gap-2.5 text-small">
              <span className="w-9 flex-none font-mono text-caps text-tx-5">{item.date}</span>
              <span className="text-tx-3">{item.text}</span>
            </li>
          ))}
        </ul>
      </Section>

      <Section title="Historique des statuts">
        {history.data && history.data.length > 0 ? (
          <ul className="flex flex-col gap-1.5">
            {history.data.map((change) => {
              const meta = Statuses.find((item) => item.value === change.status) ?? Statuses[0]!;
              return (
                <li key={`${change.changed_at}-${change.status}`} className="flex items-center gap-2 text-small">
                  <StatusGlyph tone={meta.glyph} small />
                  <span className="text-tx-3">{meta.label}</span>
                  <span className="ml-auto font-mono text-caps text-tx-5">
                    {toDisplayDate(change.changed_at.slice(0, 10)).slice(0, 5)}
                  </span>
                </li>
              );
            })}
          </ul>
        ) : (
          <p className="text-small text-tx-5">
            {history.isPending ? "Chargement…" : "Aucun changement de statut enregistré."}
          </p>
        )}
      </Section>
      <div className="h-4 flex-none" />
    </aside>
  );
}

function Field({ label, muted = false, children }: { label: string; muted?: boolean; children: ReactNode }) {
  return (
    <>
      <dt className="text-tx-5">{label}</dt>
      <dd className={cn("min-w-0 truncate", muted ? "text-tx-6" : "text-tx-2")}>{children}</dd>
    </>
  );
}

/**
 * Valeur pouvant venir de l'entreprise : l'origine est dite (« — héritée »), sans quoi une
 * adresse affichée laisserait croire qu'elle a été saisie pour cette candidature, alors
 * qu'elle suivra l'entreprise si celle-ci change.
 */
function Inherited({ label, own, value }: { label: string; own: string | null; value: string | null }) {
  if (value === null) {
    return (
      <Field label={label} muted>
        Non renseignée
      </Field>
    );
  }
  return (
    <Field label={label}>
      {value}
      {own === null ? <span className="text-tx-5"> — héritée</span> : null}
    </Field>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mt-4 border-t border-bd-soft pt-3">
      <h3 className="caps mb-2">{title}</h3>
      {children}
    </section>
  );
}
