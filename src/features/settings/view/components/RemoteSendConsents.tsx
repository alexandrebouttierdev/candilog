import { Button } from "@/shared/ui";
import { useRemoteSendConsents } from "../../viewmodel/useRemoteSendConsents";
import { SettingsRow } from "./SettingsSection";

/** Ligne « Confirmation avant envoi distant » des Réglages (D4 : réinitialisable). */
export function RemoteSendConsents() {
  const vm = useRemoteSendConsents();
  return (
    <SettingsRow
      label="Confirmation avant envoi distant"
      hint={
        vm.labels.length === 0
          ? "Candilog demande avant le premier envoi à chaque service distant."
          : `Plus de confirmation pour : ${vm.labels.join(", ")}.`
      }
    >
      <Button size="compact" disabled={vm.labels.length === 0 || vm.isResetting} onClick={vm.reset}>
        Redemander
      </Button>
    </SettingsRow>
  );
}
