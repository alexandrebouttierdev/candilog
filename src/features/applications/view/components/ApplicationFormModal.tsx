import { useEffect, useState } from "react";
import { Controller, useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  applicationFormSchema,
  type ApplicationFormInput,
  type ApplicationFormValues,
} from "../../model/schemas/application-form.schema";
import { toDisplayDate } from "@/shared/lib/dates";
import { cn } from "@/shared/lib/cn";
import { Statuses } from "../../model/statuses";
import { Channels, formatReference } from "../../model/presentation";
import type { Application, NewApplication } from "@/shared/types/generated/applications";
import { CompanyPicker, useCompany } from "@/features/companies";
import { WeeklyWorkSchedules, useReferentials } from "@/features/referentials";
import {
  ConfirmDialog,
  DateInput,
  FormField,
  ModalHost,
  SegmentedControl,
  Select,
  StatusGlyph,
  TextArea,
  TextInput,
} from "@/shared/ui";

/** Contrats proposés en premier, dans l'ordre du design ; les autres restent au choix. */
const CONTRATS_COURANTS = ["CDI", "CDD", "MIS"] as const;
const AUTRE_CONTRAT = "autre";

/** Date du jour au format saisi, valeur par défaut du champ « Envoyée le ». */
function today(): string {
  return toDisplayDate(new Date().toISOString().slice(0, 10));
}

function vide(status: ApplicationFormInput["status"] = "EN_ATTENTE"): ApplicationFormInput {
  return {
    job_title: "",
    company_id: "",
    contact_id: "",
    channel: "OFFER",
    contract_type_code: "CDI",
    weekly_work_schedule: "UNSPECIFIED",
    weekly_hours: "",
    professional_domain_id: "",
    city: "",
    address: "",
    company_type_id: "",
    status,
    sent_date: today(),
    job_url: "",
    notes: "",
  };
}

/**
 * Préremplit le formulaire depuis une candidature existante.
 *
 * Les surcharges reprennent la valeur **propre** à la candidature, jamais la valeur
 * effective : préremplir avec la ville héritée la figerait dès le premier enregistrement,
 * et un changement d'entreprise laisserait derrière lui la ville de la précédente.
 */
function from(application: Application): ApplicationFormInput {
  return {
    job_title: application.job_title,
    company_id: application.company_id,
    contact_id: application.contact_id ?? "",
    channel: application.channel,
    contract_type_code: application.contract_type_code,
    weekly_work_schedule: application.weekly_work_schedule,
    weekly_hours: application.weekly_hours === null ? "" : String(application.weekly_hours),
    professional_domain_id: application.professional_domain_id ?? "",
    city: application.city ?? "",
    address: application.address ?? "",
    company_type_id: application.company_type_id ?? "",
    status: application.status,
    sent_date: toDisplayDate(application.sent_date),
    job_url: application.job_url ?? "",
    notes: application.notes ?? "",
  };
}

/** Des précisions facultatives sont-elles déjà renseignées ? La section s'ouvre alors seule. */
function hasDetails(values: ApplicationFormInput): boolean {
  return Boolean(
    values.professional_domain_id ||
      values.company_type_id ||
      values.address ||
      values.weekly_hours ||
      values.weekly_work_schedule !== "UNSPECIFIED",
  );
}

/**
 * Formulaire `candidature` (`reference_design/DECISIONS.md` D4) : la création et la
 * modification partagent la même grille ; seul le contrat annoncé change.
 *
 * Grille de la maquette (intitulé, entreprise, ville, contrat, canal, date, statut, lien,
 * notes), plus une section repliée **Plus de détails** qui garde toutes les précisions de
 * Candilog — domaine du poste, type d'entreprise et adresse propres, régime et volume
 * horaires. Le bouton principal reste inactif tant que l'intitulé ou l'entreprise manquent ;
 * fermer une fiche modifiée passe par « Fermer sans enregistrer ? ».
 */
export function ApplicationFormModal({
  open,
  application,
  defaultStatus = null,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  application: Application | null;
  /** Statut proposé à la création, typiquement celui du groupe ou de la colonne. */
  defaultStatus?: Application["status"] | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (values: NewApplication) => Promise<unknown>;
}) {
  const referentials = useReferentials();
  const [details, setDetails] = useState(false);
  const [discard, setDiscard] = useState(false);

  // À chaque ouverture, « Plus de détails » s'ouvre seul si la fiche en porte déjà, et un
  // éventuel dialogue d'abandon précédent est oublié. Ajusté pendant le rendu plutôt que
  // dans un effet : l'état dérive des props, sans rendu intermédiaire incohérent.
  const openedFor = open ? `${application?.id ?? "nouvelle"}:${defaultStatus ?? ""}` : null;
  const [lastOpenedFor, setLastOpenedFor] = useState<string | null>(null);
  if (openedFor !== lastOpenedFor) {
    setLastOpenedFor(openedFor);
    if (openedFor !== null) {
      setDetails(application ? hasDetails(from(application)) : false);
      setDiscard(false);
    }
  }

  const form = useForm<ApplicationFormInput, unknown, ApplicationFormValues>({
    resolver: zodResolver(applicationFormSchema),
    defaultValues: vide(),
  });

  useEffect(() => {
    if (!open) return;
    form.reset(application ? from(application) : vide(defaultStatus ?? "EN_ATTENTE"));
  }, [open, application, defaultStatus, form]);

  const channel = useWatch({ control: form.control, name: "channel" });
  const companyId = useWatch({ control: form.control, name: "company_id" });
  const jobTitle = useWatch({ control: form.control, name: "job_title" });
  const contract = useWatch({ control: form.control, name: "contract_type_code" });
  const jobUrl = useWatch({ control: form.control, name: "job_url" });
  const company = useCompany(companyId || null).data ?? null;

  const missingTitle = !jobTitle?.trim();
  const missingCompany = !companyId;
  const missingLink = channel === "OFFER" && !jobUrl?.trim();
  const incomplete = missingTitle || missingCompany;

  // L'échec d'enregistrement est déjà annoncé par la mutation appelante. Il est intercepté
  // ici pour que la modale reste ouverte sur la saisie en cours, et pour ne pas laisser
  // filer un rejet non traité.
  const save = form.handleSubmit(async (values) => {
    try {
      await onSubmit(values);
    } catch {
      return;
    }
    onClose();
  });

  const requestClose = () => {
    if (form.formState.isDirty) setDiscard(true);
    else onClose();
  };

  const errors = form.formState.errors;
  const contratCourant = (CONTRATS_COURANTS as readonly string[]).includes(contract ?? "");

  return (
    <>
      <ModalHost
        open={open}
        title={application ? `Modifier ${formatReference(application.reference_number)}` : "Nouvelle candidature"}
        subtitle={
          application
            ? "Les champs reprennent la fiche actuelle. Seuls vos changements sont enregistrés."
            : "Le minimum suffit : intitulé et entreprise — et le lien, pour une offre publiée."
        }
        footer_note={application ? "⏎ enregistrer · échap abandonner" : "⏎ créer · échap abandonner"}
        submitLabel={application ? "Enregistrer les modifications" : "Créer la candidature"}
        submitShortcut="enter"
        submitDisabled={incomplete}
        busy={busy}
        onClose={requestClose}
        onSubmit={() => void save()}
        width="600px"
      >
        <form
          onSubmit={(event) => void save(event)}
          className="grid grid-cols-2 gap-x-3.5 gap-y-3"
          noValidate
        >
          <FormField
            label="Intitulé du poste"
            required
            missing={missingTitle}
            error={errors.job_title?.message}
            className="col-span-2"
          >
            {(props) => (
              <TextInput
                {...props}
                {...form.register("job_title")}
                placeholder="Intitulé exact de l'annonce"
                invalid={Boolean(errors.job_title)}
              />
            )}
          </FormField>

          <FormField
            label="Entreprise"
            required
            missing={missingCompany}
            error={errors.company_id?.message}
          >
            {(props) => (
              <Controller
                control={form.control}
                name="company_id"
                render={({ field }) => (
                  <CompanyPicker
                    id={props.id}
                    describedBy={props["aria-describedby"]}
                    invalid={props["aria-invalid"]}
                    value={field.value || null}
                    onChange={(id) => field.onChange(id ?? "")}
                  />
                )}
              />
            )}
          </FormField>

          <FormField label="Ville" hint={company?.city ? "héritée si vide" : undefined}>
            {(props) => (
              <TextInput
                {...props}
                {...form.register("city")}
                placeholder={company?.city ?? "Ville (département)"}
              />
            )}
          </FormField>

          <FormField label="Contrat" error={errors.contract_type_code?.message}>
            {(props) => (
              <div className="flex flex-col gap-1.5" id={props.id}>
                <SegmentedControl
                  label="Contrat"
                  value={contratCourant ? (contract ?? "CDI") : AUTRE_CONTRAT}
                  options={[
                    { value: "CDI", label: "CDI" },
                    { value: "CDD", label: "CDD" },
                    { value: "MIS", label: "Intérim" },
                    { value: AUTRE_CONTRAT, label: "Autre" },
                  ]}
                  onChange={(value) =>
                    form.setValue(
                      "contract_type_code",
                      value === AUTRE_CONTRAT
                        ? (referentials.data.contract_types.find(
                            (item) => !(CONTRATS_COURANTS as readonly string[]).includes(item.code),
                          )?.code ?? "CDI")
                        : value,
                      { shouldDirty: true },
                    )
                  }
                />
                {contratCourant ? null : (
                  <Select aria-label="Autre contrat" {...form.register("contract_type_code")}>
                    {referentials.data.contract_types
                      .filter((item) => !(CONTRATS_COURANTS as readonly string[]).includes(item.code))
                      .map((item) => (
                        <option key={item.code} value={item.code}>
                          {item.name}
                        </option>
                      ))}
                  </Select>
                )}
              </div>
            )}
          </FormField>

          <FormField label="Trouvée via">
            {(props) => (
              <div id={props.id}>
                <SegmentedControl
                  label="Trouvée via"
                  value={channel ?? "OFFER"}
                  options={Channels.map((item) => ({ value: item.value, label: item.label }))}
                  onChange={(value) => form.setValue("channel", value, { shouldDirty: true })}
                />
              </div>
            )}
          </FormField>

          <FormField label="Envoyée le" hint="JJ-MM-AAAA" required error={errors.sent_date?.message}>
            {(props) => (
              <DateInput {...props} {...form.register("sent_date")} invalid={Boolean(errors.sent_date)} />
            )}
          </FormField>

          <FormField label="Statut">
            {(props) => (
              <Controller
                control={form.control}
                name="status"
                render={({ field }) => (
                  <div role="radiogroup" aria-labelledby={props.id} className="flex flex-wrap gap-1.5">
                    <span id={props.id} className="sr-only">
                      Statut
                    </span>
                    {Statuses.map((status) => (
                      <button
                        key={status.value}
                        type="button"
                        role="radio"
                        aria-checked={field.value === status.value}
                        onClick={() => field.onChange(status.value)}
                        className={cn(
                          "inline-flex h-6 items-center gap-1.5 rounded-r6 border px-2 text-small transition-color",
                          field.value === status.value
                            ? "border-ac bg-sel text-tx"
                            : "border-transparent bg-chip text-tx-3 hover:text-tx",
                        )}
                      >
                        <StatusGlyph tone={status.glyph} small />
                        {status.label}
                      </button>
                    ))}
                  </div>
                )}
              />
            )}
          </FormField>

          {channel === "SPONTANEOUS" ? null : (
            <FormField
              label="Lien de l'offre"
              required={channel === "OFFER"}
              missing={missingLink}
              hint="pour retrouver l'annonce plus tard"
              error={errors.job_url?.message}
              className="col-span-2"
            >
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("job_url")}
                  placeholder="https://…"
                  invalid={Boolean(errors.job_url)}
                />
              )}
            </FormField>
          )}

          <FormField label="Notes" className="col-span-2">
            {(props) => (
              <TextArea
                {...props}
                {...form.register("notes")}
                rows={3}
                placeholder="Ce que vous voulez retenir : nom du contact, détail de l'annonce, impression…"
              />
            )}
          </FormField>

          <div className="col-span-2">
            <button
              type="button"
              aria-expanded={details}
              onClick={() => setDetails((value) => !value)}
              className="flex items-center gap-1.5 text-small text-tx-4 hover:text-tx-2"
            >
              <span aria-hidden className="text-mono-sm text-tx-5">
                {details ? "▾" : "▸"}
              </span>
              Plus de détails
              <span className="text-tx-6">domaine, type d'entreprise, adresse, horaires</span>
            </button>
          </div>

          {details ? (
            <>
              <FormField label="Domaine professionnel du poste">
                {(props) => (
                  <Select {...props} {...form.register("professional_domain_id")}>
                    <option value="">Non renseigné</option>
                    {referentials.data.professional_domains.map((domain) => (
                      <option key={domain.code} value={domain.code}>
                        {domain.name}
                      </option>
                    ))}
                  </Select>
                )}
              </FormField>

              <FormField
                label="Type d'entreprise"
                hint={company?.company_type_name ? "hérité si vide" : undefined}
              >
                {(props) => (
                  <Select {...props} {...form.register("company_type_id")}>
                    <option value="">
                      {company?.company_type_name
                        ? `Hériter — ${company.company_type_name}`
                        : "Hériter de l'entreprise"}
                    </option>
                    {referentials.data.company_types.map((type) => (
                      <option key={type.code} value={type.code}>
                        {type.name}
                      </option>
                    ))}
                  </Select>
                )}
              </FormField>

              <FormField
                label="Adresse du poste"
                hint={company?.address ? "héritée si vide" : undefined}
                className="col-span-2"
              >
                {(props) => (
                  <TextInput
                    {...props}
                    {...form.register("address")}
                    placeholder={company?.address ?? "Adresse, si elle diffère de l'entreprise"}
                  />
                )}
              </FormField>

              <FormField label="Régime horaire">
                {(props) => (
                  <Select {...props} {...form.register("weekly_work_schedule")}>
                    {WeeklyWorkSchedules.map((schedule) => (
                      <option key={schedule.value} value={schedule.value}>
                        {schedule.label}
                      </option>
                    ))}
                  </Select>
                )}
              </FormField>

              <FormField label="Heures par semaine" hint="h / semaine" error={errors.weekly_hours?.message}>
                {(props) => (
                  <TextInput
                    {...props}
                    {...form.register("weekly_hours")}
                    inputMode="decimal"
                    placeholder="35"
                    invalid={Boolean(errors.weekly_hours)}
                  />
                )}
              </FormField>
            </>
          ) : null}

          <p className="col-span-2 flex items-start gap-2 rounded-r8 bg-group px-3 py-2 text-sub text-tx-3">
            <span aria-hidden className="mt-1.5 size-1.5 flex-none rounded-full bg-ac" />
            Dès qu'elle est créée, vous pourrez générer un CV et une lettre ciblés sur cette
            offre.
          </p>

          {/* Soumission au ⏎ depuis un champ : un formulaire HTML a besoin d'un bouton. */}
          <button type="submit" hidden disabled={incomplete || busy} />
        </form>
      </ModalHost>

      <ConfirmDialog
        open={discard}
        register="confirmation"
        title="Fermer sans enregistrer ?"
        description="Les modifications de cette fiche seront perdues. La candidature enregistrée reste telle qu'elle était."
        confirmLabel="Fermer sans enregistrer"
        cancelLabel="Continuer à modifier"
        initialFocus="cancel"
        onCancel={() => setDiscard(false)}
        onConfirm={() => {
          setDiscard(false);
          onClose();
        }}
      />
    </>
  );
}
