import { useState } from "react";
import { toDisplayDate, toIsoDate } from "@/shared/lib/dates";
import { DateInput, FormField, ModalHost, TextArea } from "@/shared/ui";
import { localToday } from "../../model/history";

const MAX_NOTE = 2000;

/**
 * « Note » de l'inspecteur de Relations : un fait daté ajouté à l'historique de la fiche
 * (un appel, une réponse, une rencontre). La date est celle du fait, aujourd'hui par défaut.
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
  const [body, setBody] = useState("");
  const [date, setDate] = useState(() => toDisplayDate(localToday()));
  const notedOn = toIsoDate(date);
  const ready = body.trim() !== "" && notedOn !== null && !saving;

  const submit = () => {
    if (!ready) return;
    void onSave(body, notedOn).then((saved) => {
      if (saved) onClose();
    });
  };

  return (
    <ModalHost
      open
      title="Ajouter une note"
      subtitle={`Dans l'historique de ${name}`}
      submitLabel="Ajouter la note"
      submitDisabled={!ready}
      busy={saving}
      width="480px"
      onClose={onClose}
      onSubmit={submit}
    >
      <div className="flex flex-col gap-3">
        <FormField label="Note" required missing={body.trim() === ""} error={error ?? undefined}>
          {(props) => (
            <TextArea
              {...props}
              autoFocus
              rows={4}
              maxLength={MAX_NOTE}
              placeholder="Un appel, une réponse, une rencontre…"
              value={body}
              onChange={(event) => setBody(event.target.value)}
            />
          )}
        </FormField>
        <FormField
          label="Date"
          hint="celle du fait, pas de la saisie"
          error={notedOn === null ? "Date invalide (JJ-MM-AAAA)." : undefined}
        >
          {(props) => (
            <DateInput
              {...props}
              value={date}
              invalid={notedOn === null}
              onChange={(event) => setDate(event.target.value)}
            />
          )}
        </FormField>
      </div>
    </ModalHost>
  );
}
