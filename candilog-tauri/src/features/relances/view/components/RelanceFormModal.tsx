import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  relanceFormSchema,
  type RelanceFormInput,
  type RelanceFormValues,
} from "../../model/schemas/relance-form.schema";
import { CANAUX_RELANCE } from "../../model/types";
import type { NouvelleRelance, Relance } from "../../services/relance.service";
import { versDateAffichee } from "@/shared/lib/dates";
import {
  CandidaturePicker,
  FormField,
  ModalHost,
  Select,
  TextArea,
  TextInput,
} from "@/shared/ui";

function vide(candidatureId: string | null, jour: string | null): RelanceFormInput {
  return {
    candidatureId: candidatureId ?? "",
    dateRelance: versDateAffichee(jour ?? new Date().toISOString().slice(0, 10)),
    type: "Email",
    notes: "",
  };
}

function depuis(relance: Relance): RelanceFormInput {
  return {
    candidatureId: relance.candidatureId,
    dateRelance: versDateAffichee(relance.dateRelance),
    type: relance.type,
    notes: relance.notes ?? "",
  };
}

/**
 * Modale de programmation et de modification d'une relance.
 *
 * Structure reprise de `SPECDESIGN/Modales.dc.html` : « Candidature concernée » puis
 * « Planification ». Pas de champ heure, contrairement à l'entretien : une relance se
 * programme au jour.
 */
export function RelanceFormModal({
  open,
  relance,
  candidatureId,
  jour,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  relance: Relance | null;
  candidatureId?: string | null;
  jour?: string | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (valeurs: NouvelleRelance) => Promise<unknown>;
}) {
  const form = useForm<RelanceFormInput, unknown, RelanceFormValues>({
    resolver: zodResolver(relanceFormSchema),
    defaultValues: vide(null, null),
  });

  useEffect(() => {
    if (open) {
      form.reset(relance ? depuis(relance) : vide(candidatureId ?? null, jour ?? null));
    }
  }, [open, relance, candidatureId, jour, form]);

  const enregistrer = form.handleSubmit(async (valeurs) => {
    await onSubmit(valeurs);
    onClose();
  });

  const errors = form.formState.errors;

  return (
    <ModalHost
      open={open}
      icon="send"
      title={relance ? "Modifier la relance" : "Nouvelle relance"}
      subtitle={
        relance
          ? `${relance.candidaturePoste ?? ""} — ${relance.entrepriseNom ?? ""}`
          : "Programmez un suivi et son message"
      }
      footerNote="Le statut de la candidature n'est pas modifié automatiquement."
      submitLabel={relance ? "Enregistrer" : "Programmer"}
      busy={busy}
      onClose={onClose}
      onSubmit={() => void enregistrer()}
      width="560px"
    >
      <form onSubmit={(event) => void enregistrer(event)} className="flex flex-col gap-5">
        <fieldset className="flex flex-col gap-3">
          <legend className="text-eyebrow uppercase text-ink-faint">
            Candidature concernée
          </legend>
          <FormField label="Candidature" required error={errors.candidatureId?.message}>
            {(props) => (
              <Controller
                control={form.control}
                name="candidatureId"
                render={({ field }) => (
                  <CandidaturePicker
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
            <FormField label="Date" required error={errors.dateRelance?.message}>
              {(props) => (
                <TextInput
                  {...props}
                  {...form.register("dateRelance")}
                  placeholder="JJ-MM-AAAA"
                  inputMode="numeric"
                  invalid={Boolean(errors.dateRelance)}
                />
              )}
            </FormField>

            <FormField label="Canal">
              {(props) => (
                <Select {...props} {...form.register("type")}>
                  {CANAUX_RELANCE.map((canal) => (
                    <option key={canal} value={canal}>
                      {canal}
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

