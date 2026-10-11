import { Controller, useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useState } from "react";
import {
  Banner,
  Button,
  ConfirmDialog,
  ErrorBanner,
  FormField,
  SegmentedControl,
  SkeletonRows,
  StatusGlyph,
  Switch,
  TextInput,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";
import type { SyncProviderKind, SyncStatus } from "@/shared/types/generated/sync";
import { SettingsRow, SettingsSection } from "@/features/settings";
import {
  formatMoment,
  pendingLabel,
  PHASE_LABELS,
  phaseTone,
  PROVIDER_LABELS,
} from "../model/syncStatus";
import { unlockSchema, type UnlockValues } from "../model/schemas/sync-secret.schema";
import { useSyncSettingsViewModel, type SyncSettingsViewModel } from "../viewmodel/useSyncSettingsViewModel";
import { RecoveryKeyDialog } from "./RecoveryKeyDialog";
import { SyncConflictList } from "./SyncConflictList";
import { SyncSetupDialog } from "./SyncSetupDialog";

/**
 * Section « Synchronisation et sauvegarde » des Réglages : choix du mode de stockage,
 * état réel, conflits, restauration et désactivation. Le stockage local reste le mode par
 * défaut ; rien ne quitte l'ordinateur sans une mise en service explicite.
 */
export function SyncSettings() {
  const vm = useSyncSettingsViewModel();
  const status = vm.status;

  return (
    <SettingsSection
      title="Synchronisation et sauvegarde"
      description="Vos données restent sur cet ordinateur. Vous pouvez aussi les retrouver sur vos autres ordinateurs, chiffrées, par le service de votre choix."
    >
      <dl className="mb-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 rounded-r9 bg-group px-3 py-2.5 text-sub">
        <dt className="font-medium text-tx-3">Sauvegarde</dt>
        <dd className="text-tx-5">une copie de la base dans un fichier, quand vous le décidez.</dd>
        <dt className="font-medium text-tx-3">Synchronisation</dt>
        <dd className="text-tx-5">les mêmes données sur plusieurs ordinateurs, envoyées chiffrées et en continu.</dd>
        <dt className="font-medium text-tx-3">Restauration</dt>
        <dd className="text-tx-5">remplace les données de cet ordinateur, après une copie de secours.</dd>
      </dl>

      {vm.isLoading ? <SkeletonRows rows={3} /> : null}
      {vm.statusError ? <ErrorBanner message={vm.statusError} /> : null}
      {status && status.provider === "local" ? <ProviderChoice vm={vm} /> : null}
      {status && status.provider !== "local" ? <SyncPanel vm={vm} status={status} /> : null}

      {vm.probe ? (
        <SyncSetupDialog probe={vm.probe} busy={vm.settingUp} onSubmit={vm.setup} onClose={vm.closeProbe} />
      ) : null}
      {vm.recoveryKey ? <RecoveryKeyDialog recoveryKey={vm.recoveryKey} onClose={vm.closeRecoveryKey} /> : null}
    </SettingsSection>
  );
}

function ProviderChoice({ vm }: { vm: SyncSettingsViewModel }) {
  const connecting = vm.connecting;
  const cloud = (provider: Exclude<SyncProviderKind, "local" | "folder">, available: boolean, hint: string) => (
    <SettingsRow
      label={PROVIDER_LABELS[provider]}
      hint={available ? hint : "Indisponible dans cette version : aucune application OAuth n'y a été configurée."}
    >
      {connecting === provider ? (
        <span className="flex items-center gap-2">
          <span className="text-sub text-tx-5">Connexion dans le navigateur…</span>
          <Button size="compact" onClick={vm.cancelConnect}>
            Annuler
          </Button>
        </span>
      ) : (
        <Button size="compact" disabled={!available || connecting !== null} onClick={() => vm.connect(provider)}>
          Connecter…
        </Button>
      )}
    </SettingsRow>
  );

  return (
    <div>
      <SettingsRow
        label="Stockage local uniquement"
        hint="Mode actuel. Aucun compte, aucune connexion : tout reste sur cet ordinateur."
      >
        <span className="flex items-center gap-1.5 text-sub text-tx-4">
          <StatusGlyph tone="g" /> Actif
        </span>
      </SettingsRow>
      {cloud(
        "google_drive",
        vm.availability.google_drive,
        "Dans un dossier caché réservé à Candilog. Vous vous connectez chez Google, dans votre navigateur.",
      )}
      {cloud(
        "one_drive",
        vm.availability.one_drive,
        "Dans le dossier Applications › Candilog de votre OneDrive. Connexion chez Microsoft, dans votre navigateur.",
      )}
      <SettingsRow
        label={PROVIDER_LABELS.folder}
        hint="Un dossier de votre choix, par exemple synchronisé par Dropbox, iCloud Drive ou Nextcloud. Candilog ne contrôle pas cet outil."
      >
        <Button size="compact" disabled={vm.choosingFolder || connecting !== null} onClick={vm.chooseFolder}>
          Choisir un dossier…
        </Button>
      </SettingsRow>
    </div>
  );
}

function SyncPanel({ vm, status }: { vm: SyncSettingsViewModel; status: SyncStatus }) {
  const working = status.phase === "syncing" || status.phase === "restoring";
  return (
    <div>
      <div className="flex items-start gap-2.5 rounded-r9 border border-bd-soft px-3 py-2.5" role="status" aria-live="polite">
        <span className="mt-[3px]">
          <StatusGlyph tone={phaseTone(status.phase)} />
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-row font-medium text-tx">{PHASE_LABELS[status.phase]}</p>
          {status.message ? <p className="mt-0.5 text-sub leading-[1.45] text-tx-4">{status.message}</p> : null}
          {status.next_attempt_at ? (
            <p className="mt-0.5 text-sub text-tx-5">Nouvel essai le {formatMoment(status.next_attempt_at)}.</p>
          ) : null}
        </div>
        {status.phase === "auth_required" ? (
          <Button size="compact" variant="primary" disabled={vm.reconnecting} onClick={() => vm.reconnect(status.provider)}>
            Se reconnecter…
          </Button>
        ) : null}
      </div>

      {status.phase === "key_required" ? <UnlockForm vm={vm} /> : null}

      {status.missing_files > 0 ? (
        <Banner
          className="mt-3"
          tone="warning"
          title="Photo du profil absente"
          message="Le profil reçu référence une photo qui n'est pas (encore) disponible dans l'espace. Elle sera récupérée dès qu'elle y sera."
        />
      ) : null}

      <div className="mt-2">
        <SettingsRow label="Emplacement" {...(status.provider === "folder" && status.location ? { hint: status.location } : {})}>
          <span className="text-sub text-tx-3">{PROVIDER_LABELS[status.provider]}</span>
        </SettingsRow>
        <SettingsRow label="Dernière synchronisation confirmée" hint="Le fournisseur a confirmé l'enregistrement de toutes vos modifications.">
          <span className="font-mono text-caps text-tx-3">{formatMoment(status.last_success_at)}</span>
        </SettingsRow>
        <SettingsRow
          label="Modifications en attente"
          hint={
            status.waiting_changes > 0
              ? `${status.waiting_changes} changement(s) reçu(s) attendent une fiche liée pas encore arrivée.`
              : "Conservées sur cet ordinateur jusqu'à confirmation du fournisseur, même après un redémarrage."
          }
        >
          <span className="font-mono text-caps text-tx-3">{pendingLabel(status.pending_changes)}</span>
        </SettingsRow>
        {status.computers > 0 ? (
          <SettingsRow label="Ordinateurs reliés" hint="Ordinateurs ayant déjà publié dans cet espace, celui-ci compris.">
            <span className="font-mono text-caps text-tx-3">{status.computers}</span>
          </SettingsRow>
        ) : null}
        <SettingsRow label="Synchronisation automatique" hint="Au démarrage, après vos modifications et régulièrement pour recevoir celles des autres ordinateurs.">
          <Switch label="Synchronisation automatique" checked={status.auto_sync} onChange={vm.setAuto} />
        </SettingsRow>
        <SettingsRow label="Synchroniser maintenant" hint="Envoie les modifications en attente et reçoit celles des autres ordinateurs.">
          <Button size="compact" variant="primary" disabled={working} onClick={vm.syncNow}>
            {working ? "En cours…" : "Synchroniser"}
          </Button>
        </SettingsRow>
      </div>

      <SyncConflictList
        conflicts={vm.conflicts}
        busy={vm.resolving}
        onResolve={vm.resolve}
        onResolveAll={vm.resolveAll}
      />

      <div className="mt-5">
        <SettingsRow
          label="Restaurer depuis l'espace"
          hint="Remplace les données de cet ordinateur par celles de l'espace. Une sauvegarde est prise avant."
        >
          <Button size="compact" disabled={working || vm.restoring} onClick={() => vm.openDialog("restore")}>
            Restaurer…
          </Button>
        </SettingsRow>
        <SettingsRow
          label="Désactiver la synchronisation"
          hint="Repasse en stockage local. Pour changer de fournisseur, désactivez puis choisissez-en un autre."
        >
          <Button size="compact" disabled={vm.disconnecting} onClick={() => vm.openDialog("disconnect")}>
            Désactiver…
          </Button>
        </SettingsRow>
      </div>

      <ConfirmDialog
        open={vm.dialog === "restore"}
        register="confirmation"
        title="Restaurer depuis l'espace ?"
        description="Les données de cet ordinateur seront remplacées par celles de l'espace de synchronisation."
        consequences={[
          { label: "Une sauvegarde complète est prise avant, dans le dossier des exports" },
          { label: "Les modifications non encore envoyées seront perdues ici" },
          { label: "L'espace distant n'est pas modifié" },
        ]}
        confirmLabel="Restaurer"
        busy={vm.restoring}
        onCancel={vm.closeDialog}
        onConfirm={vm.restore}
      />
      <ConfirmDialog
        open={vm.dialog === "disconnect"}
        register="confirmation"
        title="Désactiver la synchronisation ?"
        description="Cet ordinateur repasse en stockage local."
        consequences={[
          { label: "Vos données restent sur cet ordinateur" },
          { label: "L'espace distant et les autres ordinateurs ne sont pas modifiés" },
          { label: "La clé de l'espace et la connexion au fournisseur sont retirées d'ici" },
        ]}
        note={
          status.pending_changes > 0
            ? `${pendingLabel(status.pending_changes)} n'ont pas encore été envoyées : elles resteront seulement sur cet ordinateur.`
            : undefined
        }
        confirmLabel="Désactiver"
        busy={vm.disconnecting}
        onCancel={vm.closeDialog}
        onConfirm={vm.disconnect}
      />
    </div>
  );
}

/** Déverrouillage d'un espace dont la clé manque sur cet ordinateur. */
function UnlockForm({ vm }: { vm: SyncSettingsViewModel }) {
  const [error, setError] = useState<string | null>(null);
  const form = useForm<UnlockValues>({
    resolver: zodResolver(unlockSchema),
    defaultValues: { method: "passphrase", secret: "" },
  });
  const method = useWatch({ control: form.control, name: "method" });
  const submit = form.handleSubmit(async (values) => {
    setError(null);
    try {
      await vm.unlock({
        passphrase: values.method === "passphrase" ? values.secret : null,
        recovery: values.method === "recovery" ? values.secret : null,
      });
      form.reset();
    } catch (caught) {
      setError(caught instanceof AppError ? caught.message : "Déverrouillage impossible.");
    }
  });

  return (
    <form
      className="mt-3 rounded-r9 bg-group px-3 py-3"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <div className="mb-2 flex items-center justify-between gap-3">
        <span className="text-sub font-mid text-tx-4">Déverrouiller l'espace avec</span>
        <Controller
          control={form.control}
          name="method"
          render={({ field }) => (
            <SegmentedControl
              dense
              label="Secret de l'espace"
              value={field.value}
              onChange={field.onChange}
              options={[
                { value: "passphrase", label: "Phrase secrète" },
                { value: "recovery", label: "Clé de récupération" },
              ]}
            />
          )}
        />
      </div>
      <div className="flex items-end gap-2">
        <Controller
          control={form.control}
          name="secret"
          render={({ field, fieldState }) => (
            <FormField
              className="flex-1"
              label={method === "passphrase" ? "Phrase secrète" : "Clé de récupération"}
              error={fieldState.error?.message ?? error ?? undefined}
            >
              {(props) => (
                <TextInput
                  {...props}
                  {...field}
                  type={method === "passphrase" ? "password" : "text"}
                  autoComplete="off"
                  spellCheck={false}
                  invalid={fieldState.invalid}
                />
              )}
            </FormField>
          )}
        />
        <Button type="submit" variant="primary" disabled={vm.unlocking}>
          Déverrouiller
        </Button>
      </div>
    </form>
  );
}
