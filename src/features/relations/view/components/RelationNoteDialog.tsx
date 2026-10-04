import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { toDisplayDate } from "@/shared/lib/dates";
import { DateInput, FormField, ModalHost, TextArea } from "@/shared/ui";
import { localToday } from "../../model/history";
import {
  MAX_NOTE_BODY,
  relationNoteFormSchema,
  type RelationNoteFormInput,
  type RelationNoteFormValues,
} from "../../model/schemas/relation-note.schema";

/**
 * « Note » de l'inspecteur de Relations : un fait daté ajouté à l'historique de la fiche
 * (un appel, une réponse, une rencontre). La date est celle du fait, aujourd'hui par défaut.
 *
 * React Hook Form + Zod comme tout formulaire d'entité (`docs/CODE_RULES.md` §8) : la borne
 * de 2 000 caractères et le format de date sont ainsi signalés dans le champ, et non par un
 * refus du backend au moment d'enregistrer.
 */
export function RelationNoteDialog({
  name,
  saving,
  error,
  onSave,
  onClose,
}: {
  /** Nom de l'entreprise ou du contact. */
  name: string;
  saving: boolean;
  error: string | null;
  /** Résout à `true` quand la note est enregistrée : le dialogue se ferme alors. */
  onSave: (body: string, notedOn: string) => Promise<boolean>;
  onClose: () => void;
}) {
  const form = useForm<RelationNoteFormInput, unknown, RelationNoteFormValues>({
    resolver: zodResolver(relationNoteFormSchema),
    defaultValues: { body: "", noted_on: toDisplayDate(localToday()) },
    mode: "onSubmit",
  });

  const submit = form.handleSubmit((values) => {
    return onSave(values.body, values.noted_on).then((saved) => {
      if (saved) onClose();
    });
  });

  return (
    <ModalHost
      open
      title="Ajouter une note"
      subtitle={`Dans l'historique de ${name}`}
      submitLabel="Ajouter la note"
      submitDisabled={saving}
      busy={saving}
      width="480px"
      onClose={onClose}
      onSubmit={() => void submit()}
    >
      <div className="flex flex-col gap-3">
        <Controller
          control={form.control}
          name="body"
          render={({ field, fieldState }) => (
            <FormField
              label="Note"
              required
              missing={field.value.trim() === ""}
              // L'erreur du backend prime : elle décrit un refus que le schéma n'a pas vu.
              error={error ?? fieldState.error?.message}
            >
              {(props) => (
                <TextArea
                  {...props}
                  {...field}
                  autoFocus
                  rows={4}
                  maxLength={MAX_NOTE_BODY}
                  placeholder="Un appel, une réponse, une rencontre…"
                />
              )}
            </FormField>
          )}
        />
        <Controller
          control={form.control}
          name="noted_on"
          render={({ field, fieldState }) => (
            <FormField
              label="Date"
              hint="celle du fait, pas de la saisie"
              error={fieldState.error?.message}
            >
              {(props) => (
                <DateInput {...props} {...field} invalid={fieldState.invalid} />
              )}
            </FormField>
          )}
        />
      </div>
    </ModalHost>
  );
}
