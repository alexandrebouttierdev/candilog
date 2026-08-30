import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  interviewFormSchema,
  type InterviewFormInput,
  type InterviewFormValues,
} from "../../model/schemas/interview-form.schema";
import { TYPES_INTERVIEW } from "../../model/types";
import type { Interview, NewInterview } from "../../services/interviewService";
import { contactService } from "@/features/contacts/services/contactService";
import { dateFromTimestamp, timeFromTimestamp, versDateAffichee } from "@/shared/lib/dates";
import {
  ApplicationPicker,
  DateInput,
  EntityPicker,
  FormField,
  ModalHost,
  Select,
  TextArea,
  TextInput,
  TimeInput,
} from "@/shared/ui";

function vide(application_id: string | null, day: string | null): InterviewFormInput {
  return {
    application_id: application_id ?? "",
    contact_id: "",
    date: versDateAffichee(day ?? new Date().toISOString().slice(0, 10)),
    time: "14:00",
    type: "Visio",
    location: "",
    notes: "",
    minutes: "",
  };
}

function from(interview: Interview): InterviewFormInput {
  return {
    application_id: interview.application_id,
    contact_id: interview.contact_id ?? "",
    date: dateFromTimestamp(interview.interview_date),
    time: timeFromTimestamp(interview.interview_date),
    type: interview.type,
    location: interview.location ?? "",
    notes: interview.notes ?? "",
    minutes: interview.minutes ?? "",
  };
}

/**
 * Modale de planification et de modification d'un entretien.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : sections « Context » et
 * « Organisation », date et heure séparées, champ « Url / lieu » unique.
 *
 * Le compte rendu n'est proposé qu'en modification : il se rédige après l'entretien, et
 * l'offrir à la planification suggérerait qu'il faut l'écrire tout de suite.
 */
export function InterviewFormModal({
  open,
  interview,
  application_id,
  day,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  interview: Interview | null;
  /** Application présélectionnée, quand la modale s'ouvre depuis une fiche. */
  application_id?: string | null;
  /** Day présélectionné, quand la modale s'ouvre depuis une case du calendrier. */
  day?: string | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (values: NewInterview) => Promise<unknown>;
}) {
  const form = useForm<InterviewFormInput, unknown, InterviewFormValues>({
    resolver: zodResolver(interviewFormSchema),
    defaultValues: vide(null, null),
  });

  useEffect(() => {
    if (open) {
      form.reset(interview ? from(interview) : vide(application_id ?? null, day ?? null));
    }
  }, [open, interview, application_id, day, form]);

  const save = form.handleSubmit(async (values) => {
    await onSubmit(values);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="event"
      title={interview ? "Modifier l'entretien" : "Nouvel entretien"}
      subtitle={
        interview
          ? `${interview.application_job_title ?? ""} — ${interview.company_name ?? ""}`
          : "Planifiez un échange et son contexte"
      }
      footer_note="La candidature passera au statut « Entretien »."
      submitLabel={interview ? "Enregistrer" : "Planifier"}
      busy={busy}
      onClose={onClose}
      onSubmit={() => void save()}
      width="600px"
    >
      <form onSubmit={(event) => void save(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Contexte</legend>
          <div className="flex flex-col gap-4">
            <FormField label="Candidature" required error={errors.application_id?.message}>
              {(props) => (
                <Controller
                  control={form.control}
                  name="application_id"
                  render={({ field }) => (
                    <ApplicationPicker
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

            <FormField label="Interlocuteur">
              {(props) => (
                <Controller
                  control={form.control}
                  name="contact_id"
                  render={({ field }) => (
                    <EntityPicker
                      id={props.id}
                      describedBy={props["aria-describedby"]}
                      invalid={props["aria-invalid"]}
                      value={field.value || null}
                      selectedLabel={
                        interview && interview.contact_id === field.value
                          ? interview.contact_name
                          : null
                      }
                      placeholder="Rechercher un contact…"
                      emptyHelp="Aucun contact trouvé."
                      queryKey={["contacts"]}
                      fetchPage={async (params) => {
                        const result = await contactService.listPage({
                          ...params,
                          tracking_role: null,
                        });
                        return {
                          ...result,
                          items: result.items.map((contact) => ({
                            id: contact.id,
                            label: `${contact.first_name} ${contact.name}`,
                            meta: contact.company_name ?? undefined,
                          })),
                        };
                      }}
                      onChange={(id) => field.onChange(id ?? "")}
                    />
                  )}
                />
              )}
            </FormField>
          </div>
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Organisation</legend>
          <div className="grid grid-cols-3 gap-4">
            <FormField label="Date" required error={errors.date?.message}>
              {(props) => (
                <DateInput
                  {...props}
                  {...form.register("date")}
                  invalid={Boolean(errors.date)}
                />
              )}
            </FormField>

            <FormField label="Heure" required error={errors.time?.message}>
              {(props) => (
                <TimeInput
                  {...props}
                  {...form.register("time")}
                  invalid={Boolean(errors.time)}
                />
              )}
            </FormField>

            <FormField label="Format">
              {(props) => (
                <Select {...props} {...form.register("type")}>
                  {TYPES_INTERVIEW.map((type) => (
                    <option key={type} value={type}>
                      {type}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <div className="col-span-3">
              <FormField label="Lien ou lieu">
                {(props) => (
                  <TextInput
                    {...props}
                    {...form.register("location")}
                    placeholder="https://meet… ou adresse"
                  />
                )}
              </FormField>
            </div>

            <div className="col-span-3">
              <FormField label="Préparation">
                {(props) => (
                  <TextArea
                    {...props}
                    {...form.register("notes")}
                    placeholder="Questions à poser, points à préparer…"
                  />
                )}
              </FormField>
            </div>

            {interview ? (
              <div className="col-span-3">
                <FormField label="Compte rendu">
                  {(props) => (
                    <TextArea
                      {...props}
                      {...form.register("minutes")}
                      rows={4}
                      placeholder="Ce qui s'est dit, impressions, suites à donner…"
                    />
                  )}
                </FormField>
              </div>
            ) : null}
          </div>
        </fieldset>
      </form>
    </ModalHost>
  );
}
