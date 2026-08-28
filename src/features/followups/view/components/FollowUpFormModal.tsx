import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  followUpFormSchema,
  type FollowUpFormInput,
  type FollowUpFormValues,
} from "../../model/schemas/follow-up-form.schema";
import { CANAUX_FOLLOW_UP } from "../../model/types";
import type { NewFollowUp, FollowUp } from "../../services/followUpService";
import { versDateAffichee } from "@/shared/lib/dates";
import {
  ApplicationPicker,
  FormField,
  ModalHost,
  Select,
  TextArea,
  TextInput,
} from "@/shared/ui";

function vide(application_id: string | null, day: string | null): FollowUpFormInput {
  return {
    application_id: application_id ?? "",
    follow_up_date: versDateAffichee(day ?? new Date().toISOString().slice(0, 10)),
    type: "Email",
    notes: "",
  };
}

function from(follow_up: FollowUp): FollowUpFormInput {
  return {
    application_id: follow_up.application_id,
    follow_up_date: versDateAffichee(follow_up.follow_up_date),
    type: follow_up.type,
    notes: follow_up.notes ?? "",
  };
}

/**
 * Modale de programmation et de modification d'une relance.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : « Application concernée » puis
 * « Planification ». Pas de champ heure, contrairement à l'entretien : une relance se
 * programme au jour.
 */
export function FollowUpFormModal({
  open,
  follow_up,
  application_id,
  day,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  follow_up: FollowUp | null;
  application_id?: string | null;
  day?: string | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (values: NewFollowUp) => Promise<unknown>;
}) {
  const form = useForm<FollowUpFormInput, unknown, FollowUpFormValues>({
    resolver: zodResolver(followUpFormSchema),
    defaultValues: vide(null, null),
  });

  useEffect(() => {
    if (open) {
      form.reset(follow_up ? from(follow_up) : vide(application_id ?? null, day ?? null));
    }
  }, [open, follow_up, application_id, day, form]);

  const save = form.handleSubmit(async (values) => {
    await onSubmit(values);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="send"
      title={follow_up ? "Modifier la relance" : "Nouvelle relance"}
      subtitle={
        follow_up
          ? `${follow_up.application_job_title ?? ""} — ${follow_up.company_name ?? ""}`
          : "Programmez un suivi et son message"
      }
      footer_note="Le statut de la candidature n'est pas modifié automatiquement."
      submitLabel={follow_up ? "Enregistrer" : "Programmer"}
      busy={busy}
      onClose={onClose}
      onSubmit={() => void save()}
      width="560px"
    >
      <form onSubmit={(event) => void save(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">
            Candidature concernée
          </legend>
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
        </fieldset>

        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">Planification</legend>
          <div className="grid grid-cols-2 gap-4">
            <FormField label="Date" required error={errors.follow_up_date?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("follow_up_date")}
                  placeholder="JJ-MM-AAAA"
                  inputMode="numeric"
                  invalid={Boolean(errors.follow_up_date)}
                />
              )}
            </FormField>

            <FormField label="Canal">
              {(props) => (
                <Select {...props} {...form.register("type")}>
                  {CANAUX_FOLLOW_UP.map((channel) => (
                    <option key={channel} value={channel}>
                      {channel}
                    </option>
                  ))}
                </Select>
              )}
            </FormField>

            <div className="col-span-2">
              <FormField label="Message">
                {(props) => (
                  <TextArea
                    {...props}
                    {...form.register("notes")}
                    placeholder="Angle de relance, informations utiles…"
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

