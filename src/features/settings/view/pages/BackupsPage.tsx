import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import { Button, ConfirmDialog, PageHeader } from "@/shared/ui";
import { useUiStore } from "@/shared/lib/ui-store";
import { settingsService } from "../../services/settingsService";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

/** Export, restauration et réinitialisation de la base locale. */
export function BackupsPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [busy, setBusy] = useState<"export" | "import" | "reset" | null>(null);
  const [resetOpen, setResetOpen] = useState(false);
  const [restoreOpen, setRestoreOpen] = useState(false);

  const exporter = async () => {
    setBusy("export");
    try {
      const exported = await settingsService.export();
      if (!exported) return;
      notify({ tone: "success", title: "Sauvegarde créée" });
    } catch (error) {
      notify({
        tone: "error",
        title: "Export impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    } finally {
      setBusy(null);
    }
  };

  const restore = async () => {
    setBusy("import");
    try {
      const restored = await settingsService.restore();
      if (!restored) {
        setRestoreOpen(false);
        return;
      }
      await queryClient.invalidateQueries();
      notify({ tone: "success", title: "Sauvegarde restaurée" });
      setRestoreOpen(false);
    } catch (error) {
      notify({
        tone: "error",
        title: "Restauration impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    } finally {
      setBusy(null);
    }
  };

  const reset = async () => {
    setBusy("reset");
    try {
      const outcome = await settingsService.reset();
      await queryClient.invalidateQueries();
      if (!outcome.data_cleared) {
        notify({
          tone: "error",
          title: "Réinitialisation incomplète",
          detail: "Les données locales n’ont pas toutes été supprimées.",
        });
      } else if (!outcome.secret_cleared) {
        notify({
          tone: "error",
          title: "Données effacées, clé encore présente",
          detail: "Supprimez manuellement la clé Candilog dans le coffre de mots de passe du système.",
        });
      } else {
        notify({ tone: "success", title: "Données et clé API réinitialisées" });
      }
      setResetOpen(false);
    } catch (error) {
      notify({
        tone: "error",
        title: "Réinitialisation impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    } finally {
      setBusy(null);
    }
  };

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
            <Button variant="primary" icon="download" disabled={busy !== null} onClick={() => void exporter()}>
              Export
            </Button>
          </ActionCard>
          <ActionCard
            icon="upload"
            title="Restaurer une sauvegarde"
            description="Choisissez un fichier Candilog existant avant de confirmer la restauration."
          >
            <Button variant="secondary" icon="folder_open" disabled={busy !== null} onClick={() => setRestoreOpen(true)}>
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
            <Button variant="danger" icon="delete" onClick={() => setResetOpen(true)}>
              Réinitialiser
            </Button>
          </div>
        </SettingsCard>
      </SettingsBody>

      <ConfirmDialog
        open={restoreOpen}
        title="Restaurer cette sauvegarde ?"
        description="La base actuelle sera remplacée par le fichier choisi. Une copie de secours est prise avant l’écriture."
        note="En cas d’échec, vos données d’origine sont remises en place."
        confirmLabel="Restaurer"
        busy={busy === "import"}
        onCancel={() => setRestoreOpen(false)}
        onConfirm={() => void restore()}
      />
      <ConfirmDialog
        open={resetOpen}
        title="Réinitialiser Candilog ?"
        description="Toutes vos candidatures, contacts, documents et réglages seront effacés."
        note="Le référentiel des secteurs d’activité est conservé."
        confirmLabel="Tout effacer"
        busy={busy === "reset"}
        onCancel={() => setResetOpen(false)}
        onConfirm={() => void reset()}
      />
    </div>
  );
}
