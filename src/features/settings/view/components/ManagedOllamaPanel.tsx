import { useState } from "react";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { formatDuration } from "@/shared/lib/duration";
import { Button, ConfirmDialog, ErrorBanner, Skeleton } from "@/shared/ui";
import { ManagedPublisherLogo } from "./ProviderGrid";
import { isManagedOllamaBusy } from "../../model/managedOllamaStatus";
import { CATEGORY_LABELS, formatBytes, MACHINE_FIT_LABELS } from "../../model/localInstall";
import type { ManagedOllamaViewModel } from "../../viewmodel/useManagedOllamaViewModel";

/**
 * IA locale dans l'écran Intelligence artificielle : les modèles installés, en lignes, avec
 * Utiliser / Tester / Supprimer. Installer un modèle ouvre la surcouche guidée
 * (`screens/14`) ; une installation poursuivie après sa fermeture reste suivie ici.
 */
export function ManagedOllamaPanel({
  vm,
  onInstall,
  onTestModel,
}: {
  vm: ManagedOllamaViewModel;
  /** Ouvre la surcouche « Installer l'IA locale ». */
  onInstall: () => void;
  onTestModel?: (model: ManagedModelStatus) => void;
}) {
  const [removeCandidate, setRemoveCandidate] = useState<ManagedModelStatus | null>(null);
  const busy = isManagedOllamaBusy(vm.runtimeState);

  if (vm.status === null && !vm.error) {
    return (
      <div role="status" aria-label="Chargement des modèles locaux" className="flex flex-col gap-2">
        <Skeleton className="h-4 w-1/3" />
        <Skeleton className="h-12 w-full" />
      </div>
    );
  }

  const installed = (vm.status?.models ?? []).filter((model) => model.installed);
  const available = (vm.status?.models ?? []).some((model) => !model.installed);
  const disabled = busy || vm.isInstalling || vm.isActivating;

  return (
    <section aria-label="Modèles locaux" className="flex flex-col gap-3" data-managed-ollama-state={vm.runtimeState}>
      <div className="flex items-center gap-3">
        <h2 className="caps">Modèles installés</h2>
        {vm.status && vm.status.models_disk_bytes > 0 ? (
          <span className="font-mono text-caps text-tx-6">{formatBytes(vm.status.models_disk_bytes)} sur le disque</span>
        ) : null}
        {installed.length > 0 && available ? (
          <span className="ml-auto">
            <Button size="compact" disabled={disabled} onClick={onInstall}>
              Installer un modèle
            </Button>
          </span>
        ) : null}
      </div>

      {vm.error && vm.runtimeState === "error" ? <ErrorBanner message={vm.error} onRetry={vm.reload} /> : null}

      {busy ? (
        <div role="status" aria-live="polite" className="rounded-r9 bg-group px-3.5 py-3">
          <p className="flex items-center gap-3 text-small text-tx-2">
            {vm.progress?.label ?? "Préparation de l’IA locale…"}
            <span className="ml-auto">
              <Button size="compact" onClick={vm.cancel}>
                Annuler
              </Button>
            </span>
          </p>
          <div aria-hidden className="mt-2 h-1 overflow-hidden rounded-r2 bg-chip">
            {vm.progress && vm.progress.total_bytes > 0 ? (
              <div className="h-full rounded-r2 bg-ac transition-[width]" style={{ width: `${vm.progress.progress}%` }} />
            ) : (
              <div className="import-indeterminate h-full w-1/3 rounded-r2 bg-ac" />
            )}
          </div>
          {vm.progress && vm.progress.total_bytes > 0 ? (
            <p className="mt-1.5 font-mono text-caps text-tx-5">
              {vm.progress.progress} % · {formatBytes(vm.progress.downloaded_bytes)} / {formatBytes(vm.progress.total_bytes)}
            </p>
          ) : null}
        </div>
      ) : null}

      {installed.length === 0 ? (
        busy ? null : (
          <div className="rounded-r9 bg-group px-3.5 py-3.5">
            <p className="text-small leading-[1.5] text-tx-3">
              Aucun modèle installé. Candilog télécharge le moteur et le modèle pour vous, dans son propre dossier.
            </p>
            <Button className="mt-2.5" variant="primary" size="compact" onClick={onInstall}>
              Installer l’IA locale
            </Button>
          </div>
        )
      ) : (
        <ul className="overflow-hidden rounded-r9 bg-group">
          {installed.map((model) => (
            <li
              key={model.definition.id}
              className="flex items-center gap-3 border-b border-bd-soft px-3.5 py-2.5 last:border-b-0"
            >
              <span aria-hidden className="flex size-7 flex-none items-center justify-center rounded-r7 bg-chip">
                <ManagedPublisherLogo publisher={model.definition.publisher} className="size-4" />
              </span>
              <span className="min-w-0 flex-1">
                <span className="flex items-center gap-2">
                  <span className="truncate text-ui font-medium text-tx">{model.definition.display_name}</span>
                  {model.active ? (
                    <span className="rounded-r5 bg-tint-ac-bg px-1.5 text-tiny text-tint-ac-tx">actif</span>
                  ) : null}
                </span>
                <span className="mt-0.5 block truncate text-tiny text-tx-5">
                  {model.definition.publisher_label} · {CATEGORY_LABELS[model.definition.category]} ·{" "}
                  {formatBytes(model.definition.approximate_download_bytes)} · {MACHINE_FIT_LABELS[model.machine_fit]}
                  {model.last_benchmark
                    ? ` · dernier test ${model.last_benchmark.score}/100 en ${formatDuration(model.last_benchmark.total_ms)}`
                    : ""}
                </span>
              </span>
              <span className="flex flex-none items-center gap-1.5">
                {model.active ? null : (
                  <Button size="compact" variant="primary" disabled={disabled} onClick={() => vm.activate(model.definition.id)}>
                    Utiliser
                  </Button>
                )}
                {onTestModel ? (
                  <Button size="compact" disabled={disabled} onClick={() => onTestModel(model)}>
                    Tester
                  </Button>
                ) : null}
                {model.active ? null : (
                  <Button size="compact" variant="ghost" disabled={disabled} onClick={() => setRemoveCandidate(model)}>
                    Supprimer
                  </Button>
                )}
              </span>
            </li>
          ))}
        </ul>
      )}

      <ConfirmDialog
        open={removeCandidate !== null}
        title="Supprimer ce modèle ?"
        description={removeCandidate ? `${removeCandidate.definition.display_name} sera retiré du stockage local.` : ""}
        note="Vos CV, offres, lettres et réglages personnels sont conservés."
        confirmLabel="Supprimer"
        busy={vm.isRemoving}
        onCancel={() => setRemoveCandidate(null)}
        onConfirm={() => {
          if (removeCandidate) vm.remove(removeCandidate.definition.id);
          setRemoveCandidate(null);
        }}
      />
    </section>
  );
}
