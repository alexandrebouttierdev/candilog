import { useState } from "react";
import { FormField, ModalHost, TextInput } from "@/shared/ui";

const MAX_NAME = 60;

/**
 * Enregistrer ou renommer une vue : un seul champ, le nom. Validé avant l'envoi — le
 * backend le revalide de toute façon.
 */
export function SaveViewDialog({
  open,
  title,
  initialName,
  submitLabel,
  busy,
  onClose,
  onSubmit,
}: {
  open: boolean;
  title: string;
  initialName: string;
  submitLabel: string;
  busy: boolean;
  onClose: () => void;
  onSubmit: (name: string) => Promise<unknown>;
}) {
  const [name, setName] = useState(initialName);
  const [error, setError] = useState<string | null>(null);
  const [openedWith, setOpenedWith] = useState<string | null>(null);
  const key = open ? initialName : null;
  if (key !== openedWith) {
    setOpenedWith(key);
    if (key !== null) {
      setName(initialName);
      setError(null);
    }
  }

  const submit = () => {
    const trimmed = name.trim();
    if (!trimmed) return setError("Donnez un nom à la vue.");
    if (trimmed.length > MAX_NAME) return setError(`Le nom tient en ${MAX_NAME} caractères.`);
    onSubmit(trimmed).then(onClose, () => undefined);
  };

  return (
    <ModalHost
      open={open}
      title={title}
      subtitle="Les filtres et la recherche actuels seront rejouables en un clic depuis la navigation."
      submitLabel={submitLabel}
      busy={busy}
      onClose={onClose}
      onSubmit={submit}
      width="470px"
    >
      <FormField label="Nom de la vue" required {...(error ? { error } : {})}>
        {(props) => (
          <TextInput
            {...props}
            autoFocus
            value={name}
            maxLength={MAX_NAME + 20}
            placeholder="À relancer, Entretiens à venir…"
            onChange={(event) => {
              setName(event.target.value);
              setError(null);
            }}
          />
        )}
      </FormField>
    </ModalHost>
  );
}
