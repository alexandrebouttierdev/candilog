import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  applicationFormSchema,
  type ApplicationFormInput,
  type ApplicationFormValues,
} from "../../model/schemas/application-form.schema";
import { versDateAffichee } from "@/shared/lib/dates";
import { Contracts, Statuses, contract_label } from "../../model/statuses";
import type { Application, NewApplication } from "../../services/applicationService";
import { useQuery } from "@tanstack/react-query";
import { companyService } from "@/features/companies/services/companyService";
import { EntityPicker, FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";

/** Date du jour au format saisi, valeur par défaut du champ « Date d'envoi ». */
function today(): string {
  return versDateAffichee(new Date().toISOString().slice(0, 10));
}

function vide(status: ApplicationFormInput["status"] = "EN_ATTENTE"): ApplicationFormInput {
  return {
    job_title: "",
    company_id: "",
    contract_type: "CDI",
    status,
    sent_date: today(),
    job_url: "",
    notes: "",
  };
}

function from(application: Application): ApplicationFormInput {
  return {
    job_title: application.job_title,
    company_id: application.company_id,
    contract_type: application.contract_type,
    status: application.status,
    sent_date: versDateAffichee(application.sent_date),
    job_url: application.job_url ?? "",
    notes: application.notes ?? "",
  };
}

/**
 * Modale de création et de modification d'une candidature.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : sections « Poste visé » et « Suivi »,
 * sélecteur d'entreprise paginé, rappel du format de date en pied.
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
  const form = useForm<ApplicationFormInput, unknown, ApplicationFormValues>({
    resolver: zodResolver(applicationFormSchema),
    defaultValues: vide(),
  });

  useEffect(() => {
    if (open) {
      form.reset(application ? from(application) : vide(defaultStatus ?? "EN_ATTENTE"));
    }
  }, [open, application, defaultStatus, form]);

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
      width="620px"
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
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Suivi</legend>
          <div className="grid grid-cols-3 gap-4">
            <FormField label="Contrat">
              {(props) => (
                <Select {...props} {...form.register("contract_type")}>
                  {Contracts.map((contract) => (
                    <option key={contract} value={contract}>
                      {contract_label(contract)}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

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
                <TextInput
                  {...props}
                  {...form.register("sent_date")}
                  placeholder="JJ-MM-AAAA"
                  inputMode="numeric"
                  invalid={Boolean(errors.sent_date)}
                />
              )}
            </FormField>

            <div className="col-span-3">
              <FormField label="Lien de l'offre" error={errors.job_url?.message}>
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
  return (
    <EntityPicker
      id={id}
      describedBy={describedBy}
      invalid={invalid}
      value={value}
      selectedLabel={useLabelCompany(value)}
      placeholder="Rechercher une entreprise…"
      emptyHelp="Aucun résultat. Créez l'entreprise depuis l'écran Relations."
      queryKey={["entreprises"]}
      onChange={onChange}
      fetchPage={async ({ page, page_size, search }) => {
        const resultat = await companyService.listPage({
          page,
          page_size,
          search,
          company_type: null,
        });
        return {
          ...resultat,
          items: resultat.items.map((company) => ({
            id: company.id,
            label: company.name,
            meta:
              [company.sector, company.city].filter(Boolean).join(" · ") || undefined,
          })),
        };
      }}
    />
  );
}

/** Libellé de l'entreprise sélectionnée, chargé hors de la page courante du sélecteur. */
function useLabelCompany(id: string | null): string | null {
  const { data } = useQueryCompany(id);
  return data?.name ?? null;
}

function useQueryCompany(id: string | null) {
  return useQuery({
    queryKey: ["entreprises", "detail", id],
    queryFn: () => companyService.get(id as string),
    enabled: id !== null,
  });
}
