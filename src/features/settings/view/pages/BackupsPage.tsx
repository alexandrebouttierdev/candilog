import { Button, ConfirmDialog } from "@/shared/ui";
import { useBackupsViewModel } from "../../viewmodel/useBackupsViewModel";
import { SettingsRow, SettingsSection } from "../components/SettingsSection";

/** Données : sauvegarde, restauration et remise à zéro de la base locale. */
export function BackupsPage() {
  const vm = useBackupsViewModel();

  return (
    <SettingsSection
      title="Données"
      description="Tout reste sur cet ordinateur. Une copie de sauvegarde se fait quand vous le décidez, dans le dossier de votre choix."
    >
      <SettingsRow label="Créer une sauvegarde" hint="Une archive complète de la base, à conserver où vous voulez.">
        <Button variant="primary" size="compact" disabled={vm.busy !== null} onClick={() => void vm.exportBackup()}>
          {vm.busy === "export" ? "Export…" : "Exporter"}
        </Button>
      </SettingsRow>
      <SettingsRow
        label="Restaurer une sauvegarde"
        hint="Remplace la base actuelle par un fichier Candilog ; une copie de secours est prise avant."
      >
        <Button size="compact" disabled={vm.busy !== null} onClick={vm.openRestore}>
          Restaurer…
        </Button>
      </SettingsRow>
      <SettingsRow
        label="Réinitialiser les données"
        hint="Efface candidatures, profil et documents. Le référentiel des secteurs est conservé."
      >
        <Button variant="danger" size="compact" disabled={vm.busy !== null} onClick={vm.openReset}>
          Réinitialiser…
        </Button>
      </SettingsRow>

      <ConfirmDialog
        open={vm.restoreOpen}
        title="Restaurer cette sauvegarde ?"
        description="La base actuelle sera remplacée par le fichier choisi. Une copie de secours est prise avant l’écriture."
        note="En cas d’échec, vos données d’origine sont remises en place."
        confirmLabel="Restaurer"
        busy={vm.busy === "import"}
        onCancel={vm.closeRestore}
        onConfirm={() => void vm.restoreBackup()}
      />
      <ConfirmDialog
        open={vm.resetOpen}
        title="Réinitialiser Candilog ?"
        description="Toutes vos candidatures, contacts, documents et réglages seront effacés."
        note="Le référentiel des secteurs d’activité est conservé."
        confirmLabel="Tout effacer"
        busy={vm.busy === "reset"}
        onCancel={vm.closeReset}
        onConfirm={() => void vm.resetData()}
      />
    </SettingsSection>
  );
}
