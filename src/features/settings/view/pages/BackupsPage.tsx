import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, ConfirmDialog, PageHeader } from "@/shared/ui";
import { useBackupsViewModel } from "../../viewmodel/useBackupsViewModel";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

/** Export, restauration et réinitialisation de la base locale. */
export function BackupsPage() {
  const vm = useBackupsViewModel();

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader icon="save" title="Sauvegardes" subtitle="Export, restauration et maintenance" />
      <SettingsBody>
        <SettingsHero
          kicker="Vos données"
          title="Une copie sûre, quand vous le décidez."
          description="Exportez ou restaurez toute votre base Candilog depuis un fichier local."
        />
        <div className="grid gap-4 md:grid-cols-2">
          <ActionCard
            icon="download"
            title="Créer une sauvegarde"
            description="Générez une archive complète et conservez-la où vous le souhaitez."
          >
            <Button variant="primary" icon="download" disabled={vm.busy !== null} onClick={() => void vm.exportBackup()}>
              Export
            </Button>
          </ActionCard>
          <ActionCard
            icon="upload"
            title="Restaurer une sauvegarde"
            description="Choisissez un fichier Candilog existant avant de confirmer la restauration."
          >
            <Button variant="secondary" icon="folder_open" disabled={vm.busy !== null} onClick={vm.openRestore}>
              Restaurer
            </Button>
          </ActionCard>
        </div>
        <SettingsCard icon="settings" title="Maintenance locale">
          <div className="flex flex-wrap items-center gap-3">
            <div className="min-w-0 flex-1">
              <p className="text-body text-ink">Réinitialiser les données</p>
              <p className="text-meta text-ink-muted">
                Efface candidatures, profil et documents. Le référentiel des secteurs est conservé.
              </p>
            </div>
            <Button variant="danger" icon="delete" onClick={vm.openReset}>
              Réinitialiser
            </Button>
          </div>
        </SettingsCard>
      </SettingsBody>

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
    </div>
  );
}
