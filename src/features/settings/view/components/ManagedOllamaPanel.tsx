import { useState, type ReactNode } from "react";
import type {
  ManagedModelCategory,
  ManagedModelStatus,
  ManagedRuntimeState,
  MachineFit,
} from "@/shared/types/generated/ai";
import { cn } from "@/shared/lib/cn";
import { Button, ConfirmDialog, ErrorBanner, Icon, Skeleton, StatusPill, Tag } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import { ManagedPublisherLogo } from "./ProviderGrid";
import { SettingsCard } from "./SettingsUi";
import { isManagedOllamaBusy } from "../../model/managedOllamaStatus";
import type { ManagedOllamaViewModel } from "../../viewmodel/useManagedOllamaViewModel";

const CATEGORY_LABELS: Record<ManagedModelCategory, string> = {
  ultra_light: "Très léger",
  light: "Léger",
  balanced: "Équilibré",
  powerful: "Puissant",
  max_quality: "Qualité maximale",
};

const MACHINE_FIT: Record<MachineFit, { label: string; tone: Tone }> = {
  recommended: { label: "Recommandé", tone: "accent" },
  compatible: { label: "Compatible", tone: "success" },
  may_be_slow: { label: "Peut être lent", tone: "warning" },
  insufficient_memory: { label: "Mémoire insuffisante", tone: "danger" },
};

export function ManagedOllamaPanel({
  vm,
  onTestModel,
}: {
  vm: ManagedOllamaViewModel;
  onTestModel?: (model: ManagedModelStatus) => void;
}) {
  const [installCandidate, setInstallCandidate] = useState<ManagedModelStatus | null>(null);
  const [removeCandidate, setRemoveCandidate] = useState<ManagedModelStatus | null>(null);
  const busy = isManagedOllamaBusy(vm.runtimeState);

  if (vm.status === null && !vm.error) {
    return (
      <SettingsCard icon="smart_toy" title="Modèles optimisés pour votre ordinateur">
        <div role="status" aria-label="Chargement des modèles locaux" className="space-y-3">
          <Skeleton className="h-4 w-2/3" />
          <Skeleton className="h-32 w-full rounded-card" />
        </div>
      </SettingsCard>
    );
  }

  return (
    <div className="flex flex-col gap-4" data-managed-ollama-state={vm.runtimeState}>
      <SettingsCard icon="info" title="Modèles optimisés pour votre ordinateur">
        <p className="max-w-2xl text-body leading-relaxed text-ink-muted">
          Nous mettons en avant le modèle le mieux adapté à votre machine — en pratique
          Ministral&nbsp;3&nbsp;3B dès que la mémoire le permet. Candilog gère le runtime
          Ollama et les téléchargements pour vous.
        </p>
      </SettingsCard>

      {vm.error && vm.runtimeState === "error" ? (
        <ErrorBanner message={vm.error} onRetry={vm.reload} />
      ) : null}

      {busy ? (
        <Installation runtimeState={vm.runtimeState} progress={vm.progress} onCancel={vm.cancel} />
      ) : null}

      {vm.status ? (
        <div className="grid gap-3 min-[900px]:grid-cols-2 min-[1200px]:grid-cols-3">
          {vm.status.models.map((model) => (
            <ModelCard
              key={model.definition.id}
              model={model}
              disabled={busy || vm.isInstalling || vm.isActivating}
              onInstall={() => setInstallCandidate(model)}
              onActivate={() => vm.activate(model.definition.id)}
              onRemove={() => setRemoveCandidate(model)}
              {...(onTestModel ? { onTestModel } : {})}
            />
          ))}
        </div>
      ) : null}

      <ConfirmDialog
        open={installCandidate !== null}
        title="Télécharger ce modèle ?"
        description={
          installCandidate
            ? `Candilog téléchargera ${installCandidate.definition.display_name} (${formatBytes(installCandidate.definition.approximate_download_bytes)}).`
            : ""
        }
        note="Le runtime Ollama sera installé si nécessaire. Vous pourrez annuler le téléchargement."
        confirmLabel="Télécharger"
        confirmIcon="download"
        onCancel={() => setInstallCandidate(null)}
        onConfirm={() => {
          if (installCandidate) vm.install(installCandidate.definition.id);
          setInstallCandidate(null);
        }}
      />
      <ConfirmDialog
        open={removeCandidate !== null}
        title="Supprimer ce modèle ?"
        description={
          removeCandidate
            ? `${removeCandidate.definition.display_name} sera retiré du stockage local.`
            : ""
        }
        note="Vos CV, offres, lettres et réglages personnels sont conservés."
        confirmLabel="Supprimer"
        busy={vm.isRemoving}
        onCancel={() => setRemoveCandidate(null)}
        onConfirm={() => {
          if (removeCandidate) vm.remove(removeCandidate.definition.id);
          setRemoveCandidate(null);
        }}
      />
    </div>
  );
}

function Installation({
  runtimeState,
  progress,
  onCancel,
}: {
  runtimeState: ManagedRuntimeState;
  progress: ManagedOllamaViewModel["progress"];
  onCancel: () => void;
}) {
  const titles: Partial<Record<ManagedRuntimeState, string>> = {
    downloading: "Téléchargement en cours",
    installing: "Extraction du moteur",
    starting: "Démarrage du moteur local",
    updating: "Mise à jour du runtime",
  };
  const title = titles[runtimeState] ?? "Préparation de l'IA locale";
  const percentage = progress?.progress ?? 0;
  const statusLabel =
    progress?.label ??
    (runtimeState === "installing"
      ? "Extraction du moteur…"
      : runtimeState === "starting"
        ? "Démarrage du moteur…"
        : "Préparation du téléchargement…");
  const showDeterminate =
    runtimeState === "downloading" && progress !== null && progress.total_bytes > 0;

  return (
    <SettingsCard icon="progress_activity" title={title}>
      <div role="status" aria-live="polite" className="space-y-3">
        <div className="flex items-start gap-3">
          <Icon name="progress_activity" size={20} className="mt-0.5 flex-none animate-spin text-accent" />
          <p className="text-body text-ink-muted">{statusLabel}</p>
        </div>
        {showDeterminate && progress ? (
          <>
            <div className="h-2 overflow-hidden rounded-full bg-fill">
              <div
                className="h-full rounded-full bg-accent transition-[width]"
                style={{ width: `${percentage}%` }}
              />
            </div>
            <div className="flex flex-wrap justify-between gap-2 font-mono text-note tabular-nums text-ink-muted">
              <span>{percentage} %</span>
              <span>
                {formatBytes(progress.downloaded_bytes)} / {formatBytes(progress.total_bytes)}
              </span>
            </div>
          </>
        ) : (
          <>
            <div className="h-2 overflow-hidden rounded-full bg-fill">
              <div className="import-indeterminate h-full w-1/3 rounded-full bg-accent" />
            </div>
            {runtimeState === "installing" ? (
              <p className="text-note leading-relaxed text-ink-muted">
                Décompression de l&apos;archive Ollama (~2&nbsp;Go). La première installation peut
                prendre plusieurs minutes selon votre disque.
              </p>
            ) : null}
          </>
        )}
        {runtimeState === "downloading" ||
        runtimeState === "installing" ||
        runtimeState === "starting" ? (
          <Button className="mt-1" icon="close" onClick={onCancel}>
            Annuler
          </Button>
        ) : null}
      </div>
    </SettingsCard>
  );
}

function ModelCard({
  model,
  disabled,
  onInstall,
  onActivate,
  onRemove,
  onTestModel,
}: {
  model: ManagedModelStatus;
  disabled: boolean;
  onInstall: () => void;
  onActivate: () => void;
  onRemove: () => void;
  onTestModel?: (model: ManagedModelStatus) => void;
}) {
  const fit = MACHINE_FIT[model.machine_fit];
  const canInstall = !model.installed && model.machine_fit !== "insufficient_memory";
  return (
    <article
      className={cn(
        "flex min-h-[248px] flex-col overflow-hidden rounded-card border transition-[background-color,border-color] duration-hover",
        model.active
          ? "border-accent bg-accent-tint-12"
          : "border-line bg-surface hover:border-control-strong",
      )}
    >
      <div className="flex flex-1 flex-col gap-3 px-4 pt-3.5 pb-3">
        <div className="flex items-start justify-between gap-2">
          <div className="flex min-w-0 items-start gap-3">
            <span
              className={cn(
                "mt-0.5 flex size-10 flex-none items-center justify-center rounded-tile",
                model.active ? "bg-surface" : "bg-fill",
              )}
              aria-hidden="true"
            >
              <ManagedPublisherLogo publisher={model.definition.publisher} className="size-5" />
            </span>
            <div className="min-w-0">
              <p className="text-label font-mid text-ink-muted">{model.definition.publisher_label}</p>
              <h3 className="mt-0.5 truncate text-item font-semibold text-ink">
                {model.definition.display_name}
              </h3>
              <p className="mt-1 text-eyebrow uppercase text-ink-label">
                {CATEGORY_LABELS[model.definition.category]}
              </p>
            </div>
          </div>
          {model.active ? (
            <StatusPill tone="accent" icon="check_circle">
              Actif
            </StatusPill>
          ) : model.recommended ? (
            <Tag className="flex-none">Recommandé</Tag>
          ) : null}
        </div>

        <p className="line-clamp-3 text-note leading-relaxed text-ink-muted">
          {model.definition.description}
        </p>

        <div className="mt-auto flex flex-wrap items-center gap-1.5">
          <MetaChip icon="inventory_2">{formatBytes(model.definition.approximate_download_bytes)}</MetaChip>
          <MetaChip icon="monitoring">RAM {model.definition.recommended_ram_gb}&nbsp;Go</MetaChip>
          <StatusPill tone={fit.tone} compact>
            {fit.label}
          </StatusPill>
          {model.installed && !model.active ? (
            <StatusPill tone="success" compact>
              Installé
            </StatusPill>
          ) : null}
        </div>

        {model.last_benchmark ? (
          <p className="flex items-center gap-1.5 text-meta text-ink-faint">
            <Icon name="bolt" size={14} className="flex-none text-ink-subtle" />
            <span>
              Dernier test : {model.last_benchmark.score}&nbsp;/&nbsp;100 ·{" "}
              {formatSeconds(model.last_benchmark.total_ms)}
            </span>
          </p>
        ) : null}
      </div>

      <div className="flex flex-wrap gap-2 border-t border-line-soft bg-surface-alt px-4 py-2.5">
        {!model.installed ? (
          <Button
            variant="primary"
            icon="download"
            disabled={disabled || !canInstall}
            onClick={onInstall}
          >
            Télécharger
          </Button>
        ) : model.active ? (
          <Button variant="secondary" icon="check" disabled>
            Utilisé
          </Button>
        ) : (
          <Button variant="primary" icon="check" disabled={disabled} onClick={onActivate}>
            Utiliser
          </Button>
        )}
        {model.installed && onTestModel ? (
          <Button
            variant="secondary"
            icon="bolt"
            disabled={disabled}
            onClick={() => onTestModel(model)}
          >
            Tester
          </Button>
        ) : null}
        {model.installed && !model.active ? (
          <Button variant="ghost" icon="delete" disabled={disabled} onClick={onRemove}>
            Supprimer
          </Button>
        ) : null}
      </div>
    </article>
  );
}

function MetaChip({ icon, children }: { icon: "inventory_2" | "monitoring"; children: ReactNode }) {
  return (
    <span className="inline-flex h-5 items-center gap-1 rounded-chip bg-fill px-1.5 text-meta text-ink-muted">
      <Icon name={icon} size={12} className="flex-none text-ink-faint" />
      {children}
    </span>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1_000_000_000) {
    return `${(bytes / 1_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 0 })} Mo`;
  }
  return `${(bytes / 1_000_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} Go`;
}

function formatSeconds(ms: number): string {
  return `${(ms / 1000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} s`;
}
