import { useState } from "react";
import { ConfirmDialog, Switch } from "@/shared/ui";
import { useRemoteSendStore } from "../../viewmodel/useRemoteSendGuard";

/** « Votre profil… » devient « votre profil… » après les deux-points. */
function lowerFirst(text: string): string {
  return text.charAt(0).toLowerCase() + text.slice(1);
}

/**
 * Premier envoi à un service distant (`states/dialog-confirm-remote-send.png`, D4) : qui
 * reçoit quoi, pour quelles tâches. « Ne plus demander » est désactivé par défaut
 * (`HANDOFF_CHANGELOG.md` A3) : se taire est un choix de l'utilisateur, jamais un défaut.
 */
export function RemoteSendDialog() {
  const pending = useRemoteSendStore((state) => state.pending);
  const answer = useRemoteSendStore((state) => state.answer);
  const [remember, setRemember] = useState(false);
  const close = (send: boolean) => {
    answer({ send, remember: send && remember });
    setRemember(false);
  };
  const recipient = pending?.destination.recipient ?? "";

  return (
    <ConfirmDialog
      open={pending !== null}
      register="confirmation"
      title={`Cette tâche sera envoyée à ${recipient}`}
      description={`Ce qui quitte votre ordinateur pour « ${pending?.taskLabel ?? ""} » : ${lowerFirst(pending?.content ?? "")}. ${recipient} le traite selon sa propre politique. Ce que les autres tâches confient à l’IA locale reste sur votre machine.`}
      consequences={
        pending
          ? [
              { label: "Destinataire", value: recipient },
              { label: "Tâche", value: pending.taskLabel },
              { label: "Tâches concernées", value: `${pending.tasks} sur 5` },
            ]
          : []
      }
      footnote="demandé une seule fois par service"
      cancelLabel="Annuler"
      confirmLabel="Envoyer"
      initialFocus="cancel"
      onCancel={() => close(false)}
      onConfirm={() => close(true)}
    >
      <label className="flex items-center gap-2.5 rounded-r8 bg-group px-3 py-2 text-small text-tx-2">
        <Switch checked={remember} onChange={setRemember} label={`Ne plus demander pour ${recipient}`} />
        Ne plus demander pour {recipient}
      </label>
    </ConfirmDialog>
  );
}
