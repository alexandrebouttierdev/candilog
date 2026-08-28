import { useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useQueryClient } from "@tanstack/react-query";
import { AppError } from "@/shared/types/app-error";
import { Button, ConfirmDialog, PageHeader } from "@/shared/ui";
import { useUiStore } from "@/shared/lib/ui-store";
import { parametresService } from "../../services/parametres.service";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

/** Export, restauration et réinitialisation de la base locale. */
export function SauvegardesPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [busy, setBusy] = useState<"export" | "import" | "reset" | null>(null);
  const [resetOpen, setResetOpen] = useState(false);
  const [importPath, setImportPath] = useState<string | null>(null);

  const exporter = async () => {
    const chemin = await save({
      title: "Exporter une sauvegarde Candilog",
      defaultPath: "candilog.sqlite",
      filters: [{ name: "Base SQLite", extensions: ["sqlite"] }],
    });
    if (chemin === null) return;
    setBusy("export");
    try {
      await parametresService.exporter(chemin);
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

  const choisirRestauration = async () => {
    const fichier = await open({
      multiple: false,
      filters: [{ name: "Base SQLite", extensions: ["sqlite", "bak"] }],
    });
    if (typeof fichier === "string") setImportPath(fichier);
  };

  const restaurer = async () => {
    if (!importPath) return;
    setBusy("import");
    try {
      await parametresService.restaurer(importPath);
      await queryClient.invalidateQueries();
      notify({ tone: "success", title: "Sauvegarde restaurée" });
      setImportPath(null);
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

  const reinitialiser = async () => {
    setBusy("reset");
    try {
      await parametresService.reinitialiser();
      await queryClient.invalidateQueries();
      notify({ tone: "success", title: "Données réinitialisées" });
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
              Exporter
            </Button>
          </ActionCard>
          <ActionCard
            icon="upload"
            title="Restaurer une sauvegarde"
            description="Choisissez un fichier Candilog existant avant de confirmer la restauration."
          >
            <Button variant="secondary" icon="folder_open" disabled={busy !== null} onClick={() => void choisirRestauration()}>
              Choisir un fichier
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
        open={importPath !== null}
        title="Restaurer cette sauvegarde ?"
        description="La base actuelle sera remplacée par le fichier choisi. Une copie de secours est prise avant l’écriture."
        note="En cas d’échec, vos données d’origine sont remises en place."
        confirmLabel="Restaurer"
        busy={busy === "import"}
        onCancel={() => setImportPath(null)}
        onConfirm={() => void restaurer()}
      />
      <ConfirmDialog
        open={resetOpen}
        title="Réinitialiser Candilog ?"
        description="Toutes vos candidatures, contacts, documents et réglages seront effacés."
        note="Le référentiel des secteurs d’activité est conservé."
        confirmLabel="Tout effacer"
        busy={busy === "reset"}
        onCancel={() => setResetOpen(false)}
        onConfirm={() => void reinitialiser()}
      />
    </div>
  );
}
