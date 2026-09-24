import { ConfirmDialog } from "@/shared/ui";

/**
 * Arrêter une génération passe par un dialogue (`states/dialog-stop-gen.png`,
 * `INTERACTIONS.md` — Annulation) : le document en cours serait perdu, on dit où en est le
 * modèle et depuis combien de temps il travaille.
 */
export function StopGenerationDialog({
  open,
  step,
  elapsedMs,
  onKeepGoing,
  onStop,
}: {
  open: boolean;
  step: string | null;
  elapsedMs: number;
  onKeepGoing: () => void;
  onStop: () => void;
}) {
  return (
    <ConfirmDialog
      open={open}
      register="confirmation"
      title="Interrompre la génération ?"
      description={`Le document en cours sera perdu.${step ? ` Le modèle en est à l’étape « ${step} ».` : ""}`}
      footnote={`${Math.round(elapsedMs / 1000)} s écoulées`}
      cancelLabel="Laisser finir"
      confirmLabel="Interrompre"
      onCancel={onKeepGoing}
      onConfirm={onStop}
    />
  );
}
