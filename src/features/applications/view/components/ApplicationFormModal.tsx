import { useEffect } from "react";
import { Controller, useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  applicationFormSchema,
  type ApplicationFormInput,
  type ApplicationFormValues,
} from "../../model/schemas/application-form.schema";
import { versDateAffichee } from "@/shared/lib/dates";
import { Statuses } from "../../model/statuses";
import type { Application, NewApplication } from "../../services/applicationService";
import { useQuery } from "@tanstack/react-query";
import { companyService } from "@/features/companies/services/companyService";
import {
  ApplicationTypes,
  WeeklyWorkSchedules,
  useReferentials,
} from "@/features/referentials";
import {
  DateInput,
  EntityPicker,
  FormField,
  ModalHost,
  Select,
  TextArea,
  TextInput,
} from "@/shared/ui";

/** Date du jour au format saisi, valeur par défaut du champ « Date d'envoi ». */
function today(): string {
  return versDateAffichee(new Date().toISOString().slice(0, 10));
}

function vide(status: ApplicationFormInput["status"] = "EN_ATTENTE"): ApplicationFormInput {
  return {
    job_title: "",
    company_id: "",
    contact_id: "",
    application_type: "OFFRE",
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
    application_type: application.application_type,
    contract_type_code: application.contract_type_code,
    weekly_work_schedule: application.weekly_work_schedule,
    weekly_hours: application.weekly_hours === null ? "" : String(application.weekly_hours),
    professional_domain_id: application.professional_domain_id ?? "",
    city: application.city ?? "",
    address: application.address ?? "",
    company_type_id: application.company_type_id ?? "",
    status: application.status,
    sent_date: versDateAffichee(application.sent_date),
    job_url: application.job_url ?? "",
    notes: application.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'une candidature.
 *
 * Trois sections : le poste visé, le suivi, et les informations propres à la candidature —
 * celles qui surchargent l'entreprise. Le lien de l'offre n'apparaît que pour une réponse à
 * une offre : une candidature spontanée n'en a pas.
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
  /** Statut proposé à la création, typiquement celui de la colonne Kanban. */
  defaultStatus?: Application["status"] | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (values: NewApplication) => Promise<unknown>;
}) {
  const referentials = useReferentials();

  const form = useForm<ApplicationFormInput, unknown, ApplicationFormValues>({
    resolver: zodResolver(applicationFormSchema),
    defaultValues: vide(),
  });

  useEffect(() => {
    if (open) {
      form.reset(application ? from(application) : vide(defaultStatus ?? "EN_ATTENTE"));
    }
  }, [open, application, defaultStatus, form]);

  const applicationType = useWatch({ control: form.control, name: "application_type" });
  const companyId = useWatch({ control: form.control, name: "company_id" });
  const company = useQueryCompany(companyId || null).data ?? null;

  const save = form.handleSubmit(async (values) => {
    await onSubmit(values);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="work"
      title={application ? "Modifier la candidature" : "Nouvelle candidature"}
      subtitle={
        application
          ? `${application.job_title} — ${application.company_name ?? ""}`
          : "Renseignez le poste et l'entreprise visés"
      }
      footer_note="Les dates sont saisies au format JJ-MM-AAAA."
      busy={busy}
      onClose={onClose}
      onSubmit={() => void save()}
      width="680px"
    >
      <form onSubmit={(event) => void save(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Poste visé</legend>
          <div className="flex flex-col gap-4">
            <FormField label="Poste" required error={errors.job_title?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("job_title")}
                  placeholder="Développeur Frontend"
                  invalid={Boolean(errors.job_title)}
                />
              )}
            </FormField>

            <FormField label="Entreprise" required error={errors.company_id?.message}>
              {(props) => (
                <Controller
                  control={form.control}
                  name="company_id"
                  render={({ field }) => (
                    <CompanyFieldPicker
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

            <div className="grid grid-cols-2 gap-4">
              <FormField label="Domaine professionnel">
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

              <FormField label="Type de candidature">
                {(props) => (
                  <Select {...props} {...form.register("application_type")}>
                    {ApplicationTypes.map((type) => (
                      <option key={type.value} value={type.value}>
                        {type.label}
                      </option>
                    ))}
                  </Select>
                )}
              </FormField>
            </div>
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Contrat</legend>
          <div className="grid grid-cols-3 gap-4">
            <FormField
              label="Type de contrat"
              required
              error={errors.contract_type_code?.message}
            >
              {(props) => (
                <Select {...props} {...form.register("contract_type_code")}>
                  {referentials.data.contract_types.map((contract) => (
                    <option key={contract.code} value={contract.code}>
                      {contract.name}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="Durée hebdomadaire">
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

            <FormField
              label="Heures par semaine"
              help="heures / semaine"
              error={errors.weekly_hours?.message}
            >
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
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">
            Informations propres à cette candidature
          </legend>
          <div className="grid grid-cols-2 gap-4">
            <FormField
              label="Ville"
              help={heritage(company?.city ?? null, "Ville de l'entreprise")}
            >
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("city")}
                  placeholder={company?.city ?? "Ville du poste"}
                />
              )}
            </FormField>

            <FormField
              label="Type d'entreprise"
              help={heritage(company?.company_type_name ?? null, "Type de l'entreprise")}
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

            <div className="col-span-2">
              <FormField
                label="Adresse"
                help={heritage(company?.address ?? null, "Adresse de l'entreprise")}
              >
                {(props) => (
                  <TextInput
                    {...props}
                    {...form.register("address")}
                    placeholder={company?.address ?? "Adresse du poste"}
                  />
                )}
              </FormField>
            </div>
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Suivi</legend>
          <div className="grid grid-cols-3 gap-4">
            <FormField label="Statut">
              {(props) => (
                <Select {...props} {...form.register("status")}>
                  {Statuses.map((status) => (
                    <option key={status.value} value={status.value}>
                      {status.label}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <FormField label="Date d'envoi" required error={errors.sent_date?.message}>
              {(props) => (
                <DateInput
                  {...props}
                  {...form.register("sent_date")}
                  invalid={Boolean(errors.sent_date)}
                />
              )}
            </FormField>

            {applicationType === "OFFRE" ? (
              <div className="col-span-3">
                <FormField label="Lien de l'offre" required error={errors.job_url?.message}>
                  {(props) => (
                    <TextInput
                      {...props}
                      {...form.register("job_url")}
                      placeholder="https://…"
                      invalid={Boolean(errors.job_url)}
                    />
                  )}
                </FormField>
              </div>
            ) : null}

            <div className="col-span-3">
              <FormField label="Notes">
                {(props) => (
                  <TextArea
                    {...props}
                    {...form.register("notes")}
                    placeholder="Contexte, personne rencontrée, éléments à retenir…"
                  />
                )}
              </FormField>
            </div>
          </div>
        </fieldset>
      </form>
    </ModalHost>
  );
}

/** Indication de la valeur héritée de l'entreprise, si elle existe. */
function heritage(value: string | null, prefix: string): string | undefined {
  return value ? `${prefix} : ${value}` : undefined;
}

/**
 * Sélecteur d'entreprise du formulaire, branché sur le répertoire paginé.
 *
 * Le libellé de la sélection est chargé séparément : le sélecteur ne connaît que la page
 * courante, et l'entreprise déjà choisie peut n'y figurer pas.
 */
function CompanyFieldPicker({
  id,
  describedBy,
  invalid,
  value,
  onChange,
}: {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
  value: string | null;
  onChange: (id: string | null) => void;
}) {
  const selected = useQueryCompany(value).data ?? null;

  return (
    <EntityPicker
      id={id}
      describedBy={describedBy}
      invalid={invalid}
      value={value}
      selectedLabel={selected?.name ?? null}
      placeholder="Rechercher une entreprise…"
      emptyHelp="Aucun résultat. Créez l'entreprise depuis l'écran Relations."
      queryKey={["entreprises"]}
      onChange={onChange}
      fetchPage={async ({ page, page_size, search }) => {
        const resultat = await companyService.listPage({
          page,
          page_size,
          filter: {
            search,
            sector_id: null,
            company_type_id: null,
            company_size: null,
          },
        });
        return {
          ...resultat,
          items: resultat.items.map((company) => ({
            id: company.id,
            label: company.name,
            meta:
              [company.sector_name, company.city].filter(Boolean).join(" · ") || undefined,
          })),
        };
      }}
    />
  );
}

/** Fiche complète d'une entreprise, pour son libellé et ses valeurs héritables. */
function useQueryCompany(id: string | null) {
  return useQuery({
    queryKey: ["entreprises", "detail", id],
    queryFn: () => companyService.get(id as string),
    enabled: id !== null,
  });
}
