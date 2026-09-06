import { useState } from "react";
import type {
  LocalAiBackend,
  LocalAiState,
  LocalAiStatus,
  LocalModelDefinition,
  LocalModelProfile,
} from "@/shared/types/generated/ai";
import { Button, ConfirmDialog, ErrorBanner, Icon, Skeleton } from "@/shared/ui";
import { SettingsCard } from "./SettingsUi";
import { useLocalAiViewModel } from "../../viewmodel/useLocalAiViewModel";

const PROFILE_LABELS: Record<LocalModelProfile, string> = {
  light: "Léger",
  balanced: "Équilibré",
  quality: "Qualité",
};

const BACKEND_LABELS: Record<LocalAiBackend, string> = {
  metal: "Metal",
  cuda: "CUDA",
  vulkan: "Vulkan",
  cpu: "CPU",
};

export function MistralLocalPanel({ onConfigured }: { onConfigured?: () => void }) {
  const vm = useLocalAiViewModel(onConfigured);
  const [installCandidate, setInstallCandidate] = useState<LocalModelDefinition | null>(null);
  const [removeCandidate, setRemoveCandidate] = useState<LocalModelDefinition | null>(null);
  const recommended = vm.recommendation?.selected_model ?? null;
  const active = vm.status?.active_model ?? null;
  const downgrade = active && vm.status?.benchmark?.rating === "too_slow"
    ? findDowngrade(active, vm.recommendation?.evaluations.map((item) => item.model) ?? [])
    : null;

  return (
    <div className="flex flex-col gap-4" data-local-ai-state={vm.state}>
      <LocalPrivacyNotice />

      {vm.state === "detecting_hardware" ? <Detecting /> : null}
      {vm.error && vm.state === "error" ? (
        <ErrorBanner message={vm.error} onRetry={vm.reevaluate} />
      ) : null}
      {vm.recommendation && !recommended && !isBusy(vm.state) ? (
        <Unsupported onReevaluate={vm.reevaluate} />
      ) : null}
      {recommended && !active && !isBusy(vm.state) && vm.state !== "error" ? (
        <Recommendation model={recommended} onInstall={() => setInstallCandidate(recommended)} />
      ) : null}
      {isBusy(vm.state) ? (
        <Installation state={vm.state} model={recommended} progress={vm.progress} onCancel={vm.cancel} />
      ) : null}
      {vm.state === "ready" && active && vm.status ? (
        <Ready
          model={active}
          status={vm.status}
          testResult={vm.testResult}
          testing={vm.isTesting}
          onTest={vm.test}
          onBenchmark={vm.benchmark}
          onRemove={setRemoveCandidate}
          onReevaluate={vm.reevaluate}
          downgrade={downgrade}
          onDowngrade={() => downgrade && setInstallCandidate(downgrade)}
        />
      ) : null}

      <ConfirmDialog
        open={installCandidate !== null}
        title="Installer l'IA locale ?"
        description={
          installCandidate
            ? `Candilog téléchargera uniquement le profil ${PROFILE_LABELS[installCandidate.profile]} (${formatBytes(installCandidate.download_size_bytes)}).`
            : ""
        }
        note="Le fichier sera vérifié avant son installation. Vous pourrez annuler le téléchargement."
        confirmLabel="Installer"
        confirmIcon="download"
        onCancel={() => setInstallCandidate(null)}
        onConfirm={() => {
          if (installCandidate) vm.install(installCandidate.id);
          setInstallCandidate(null);
        }}
      />
      <ConfirmDialog
        open={removeCandidate !== null}
        title="Supprimer le modèle local ?"
        description={removeCandidate ? `${removeCandidate.display_name} sera retiré du stockage Candilog et libérera ${formatBytes(removeCandidate.download_size_bytes)}.` : ""}
        note="Vos CV, offres, lettres et réglages personnels sont conservés."
        confirmLabel="Supprimer"
        busy={vm.isRemoving}
        onCancel={() => setRemoveCandidate(null)}
        onConfirm={() => {
          if (removeCandidate) vm.remove(removeCandidate.id);
          setRemoveCandidate(null);
        }}
      />
    </div>
  );
}

function LocalPrivacyNotice() {
  return (
    <section className="rounded-card border border-accent-border bg-accent-tint px-[18px] py-4">
      <div className="flex gap-3">
        <Icon name="lock" size={20} className="mt-0.5 flex-none text-accent" />
        <div>
          <p className="text-item font-semibold text-ink">Fonctionne directement sur votre ordinateur</p>
          <p className="mt-1 text-body leading-relaxed text-ink-muted">
            Aucun compte ni clé API requis. Vos données restent sur votre appareil.
          </p>
        </div>
      </div>
    </section>
  );
}

function Detecting() {
  return (
    <SettingsCard icon="smart_toy" title="Analyse de votre ordinateur">
      <div role="status" aria-label="Détection de la configuration" className="space-y-3">
        <p className="text-body text-ink-muted">Candilog évalue la mémoire et l'accélération disponibles…</p>
        <Skeleton className="h-2 w-full rounded-full" />
      </div>
    </SettingsCard>
  );
}

function Unsupported({ onReevaluate }: { onReevaluate: () => void }) {
  return (
    <SettingsCard icon="info" title="IA locale non recommandée">
      <p className="max-w-2xl text-body leading-relaxed text-ink-muted">
        Votre ordinateur ne dispose pas de suffisamment de ressources pour garantir une utilisation
        fluide de l'IA locale. Vous pouvez utiliser un fournisseur IA distant ou configurer Ollama.
      </p>
      <Button className="mt-4" icon="refresh" onClick={onReevaluate}>
        Réévaluer ma configuration
      </Button>
    </SettingsCard>
  );
}

function Recommendation({ model, onInstall }: { model: LocalModelDefinition; onInstall: () => void }) {
  return (
    <SettingsCard icon="check_circle" title="Votre ordinateur est compatible avec l'IA locale">
      <div className="grid max-w-2xl gap-3 sm:grid-cols-3">
        <Metric label="Modèle recommandé" value={PROFILE_LABELS[model.profile]} />
        <Metric label="Qualité" value={model.profile === "light" ? "Bonne" : "Élevée"} />
        <Metric label="Téléchargement" value={formatBytes(model.download_size_bytes)} />
      </div>
      <Button variant="primary" icon="download" className="mt-5" onClick={onInstall}>
        Installer l'IA locale
      </Button>
    </SettingsCard>
  );
}

function Installation({
  state,
  model,
  progress,
  onCancel,
}: {
  state: LocalAiState;
  model: LocalModelDefinition | null;
  progress: ReturnType<typeof useLocalAiViewModel>["progress"];
  onCancel: () => void;
}) {
  const titles: Partial<Record<LocalAiState, string>> = {
    downloading: "Téléchargement de l'IA locale",
    verifying: "Vérification du téléchargement",
    installing: "Installation du modèle",
    benchmarking: "Mesure des performances",
  };
  const title = titles[state] ?? "Préparation de l'IA locale";
  const percentage = progress?.progress ?? 0;
  return (
    <SettingsCard icon="download" title={title}>
      <div role="status" aria-live="polite">
        <div className="h-2 overflow-hidden rounded-full bg-fill">
          <div className="h-full rounded-full bg-accent transition-[width]" style={{ width: `${percentage}%` }} />
        </div>
        <div className="mt-2 flex flex-wrap justify-between gap-2 font-mono text-note tabular-nums text-ink-muted">
          <span>{percentage} %</span>
          {progress ? (
            <span>
              {formatBytes(progress.downloaded_bytes)} / {formatBytes(progress.total_bytes)}
              {progress.bytes_per_second > 0 ? ` · ${formatBytes(progress.bytes_per_second)}/s` : ""}
            </span>
          ) : model ? <span>{formatBytes(model.download_size_bytes)}</span> : null}
        </div>
        {state === "benchmarking" ? (
          <p className="mt-3 text-note text-ink-faint">Test synthétique local, sans aucune donnée personnelle.</p>
        ) : null}
        {state === "downloading" ? (
          <Button className="mt-4" icon="close" onClick={onCancel}>Annuler</Button>
        ) : null}
      </div>
    </SettingsCard>
  );
}

function Ready({
  model,
  status,
  testResult,
  testing,
  onTest,
  onBenchmark,
  onRemove,
  onReevaluate,
  downgrade,
  onDowngrade,
}: {
  model: LocalModelDefinition;
  status: LocalAiStatus;
  testResult: string | null;
  testing: boolean;
  onTest: () => void;
  onBenchmark: () => void;
  onRemove: (model: LocalModelDefinition) => void;
  onReevaluate: () => void;
  downgrade: LocalModelDefinition | null;
  onDowngrade: () => void;
}) {
  return (
    <SettingsCard icon="check_circle" title="IA locale prête">
      <div className="grid gap-3 sm:grid-cols-4">
        <Metric label="Profil" value={PROFILE_LABELS[model.profile]} />
        <Metric label="Modèle" value={model.display_name.replace(" Instruct", "")} />
        <Metric label="Stockage" value={formatBytes(model.download_size_bytes)} />
        <Metric
          label="Vitesse"
          value={status.benchmark ? `${formatDecimal(status.benchmark.tokens_per_second)} tokens/s` : "À mesurer"}
        />
      </div>
      <div className="mt-5 flex flex-wrap gap-2">
        <Button variant="primary" icon="smart_toy" disabled={testing} onClick={onTest}>Tester l'IA</Button>
        <Button icon="delete" onClick={() => onRemove(model)}>Supprimer le modèle</Button>
        <Button icon="refresh" onClick={onReevaluate}>Réévaluer ma configuration</Button>
      </div>
      {testResult ? <p role="status" className="mt-3 text-body text-success">{testResult}</p> : null}
      {downgrade ? (
        <div className="mt-4 rounded-field border border-warning bg-warning-tint px-3 py-3">
          <p className="text-body font-mid text-ink">Ce profil est trop lent sur votre ordinateur.</p>
          <p className="mt-1 text-note text-ink-muted">
            Le profil {PROFILE_LABELS[downgrade.profile]} devrait être plus fluide. Il ne sera téléchargé qu'après votre confirmation.
          </p>
          <Button className="mt-3" icon="download" onClick={onDowngrade}>
            Installer le profil {PROFILE_LABELS[downgrade.profile]}
          </Button>
        </div>
      ) : null}
      {status.installed_models.filter((installed) => installed.id !== model.id).map((installed) => (
        <div key={installed.id} className="mt-4 rounded-field border border-line bg-fill px-3 py-3">
          <p className="text-body font-mid text-ink">Ancien profil encore disponible : {PROFILE_LABELS[installed.profile]}</p>
          <p className="mt-1 text-note text-ink-muted">
            Le nouveau modèle est validé. Vous pouvez supprimer l'ancien et libérer {formatBytes(installed.download_size_bytes)}.
          </p>
          <Button className="mt-3" icon="delete" onClick={() => onRemove(installed)}>
            Supprimer l'ancien modèle
          </Button>
        </div>
      ))}
      <details className="mt-5 border-t border-line-soft pt-4">
        <summary className="cursor-pointer text-label font-mid text-ink">Configuration avancée</summary>
        <dl className="mt-3 grid gap-x-6 gap-y-2 text-note sm:grid-cols-2">
          <Advanced label="Modèle" value={model.display_name} />
          <Advanced label="Quantification" value={model.quantization} />
          <Advanced label="Runtime" value={model.runtime} />
          <Advanced label="Backend" value={status.backend ? BACKEND_LABELS[status.backend] : "CPU"} />
          <Advanced label="Contexte" value={`${model.context_size} tokens`} />
          <Advanced label="RAM estimée" value={`${model.estimated_ram_mb} Mo`} />
        </dl>
        <Button className="mt-3" icon="refresh" onClick={onBenchmark}>Relancer le benchmark</Button>
      </details>
    </SettingsCard>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="rounded-field bg-fill px-3 py-2.5"><p className="text-meta text-ink-faint">{label}</p><p className="mt-1 text-item font-semibold text-ink">{value}</p></div>;
}

function Advanced({ label, value }: { label: string; value: string }) {
  return <div className="flex justify-between gap-3 border-b border-line-soft py-1"><dt className="text-ink-faint">{label}</dt><dd className="font-mono text-right text-ink">{value}</dd></div>;
}

function isBusy(state: LocalAiState): boolean {
  return ["downloading", "verifying", "installing", "benchmarking"].includes(state);
}

function formatBytes(bytes: number): string {
  if (bytes < 1_000_000_000) return `${(bytes / 1_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 0 })} Mo`;
  return `${(bytes / 1_000_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} Go`;
}

function formatDecimal(value: number): string {
  return value.toLocaleString("fr-FR", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
}

function findDowngrade(
  active: LocalModelDefinition,
  models: LocalModelDefinition[],
): LocalModelDefinition | null {
  const target: Partial<Record<LocalModelProfile, LocalModelProfile>> = {
    quality: "balanced",
    balanced: "light",
  };
  const profile = target[active.profile];
  return profile ? models.find((model) => model.profile === profile) ?? null : null;
}
