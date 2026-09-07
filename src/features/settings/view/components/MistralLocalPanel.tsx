import { useState } from "react";
import { cn } from "@/shared/lib/cn";
import type {
  LocalAiBackend,
  LocalAiState,
  LocalAiStatus,
  LocalModelDefinition,
  LocalModelEvaluation,
  LocalModelFamily,
  LocalModelProfile,
  ModelCompatibility,
} from "@/shared/types/generated/ai";
import logoMistral from "@/assets/providers/mistralai.svg";
import logoQwen from "@/assets/providers/qwen.svg";
import { Button, ConfirmDialog, ErrorBanner, Icon, Skeleton, StatusPill } from "@/shared/ui";
import type { IconName, Tone } from "@/shared/ui";
import { isLocalAiBusy } from "../../model/etatLocalIa";
import { SettingsCard } from "./SettingsUi";
import type { LocalAiViewModel } from "../../viewmodel/useLocalAiViewModel";

const PROFILE_LABELS: Record<LocalModelProfile, string> = {
  ultra_light: "Ultra léger",
  light: "Léger",
  balanced: "Équilibré",
  quality: "Qualité",
};

/// Icône de chaque profil : du plus frugal au plus exigeant.
const PROFILE_ICONS: Record<LocalModelProfile, IconName> = {
  ultra_light: "bolt",
  light: "rocket_launch",
  balanced: "tune",
  quality: "workspace_premium",
};

/// Logo et nom de chaque famille d'artefacts, choisis sur la propriété `family` renvoyée
/// par le backend — jamais sur le nom affiché du modèle.
const FAMILY_LOGOS: Record<LocalModelFamily, { src: string; label: string; mono: boolean }> = {
  mistral: { src: logoMistral, label: "Mistral", mono: false },
  qwen: { src: logoQwen, label: "Qwen", mono: false },
};

/// Traduction de la compatibilité mesurée par le backend.
///
/// La couleur ne porte jamais l'information seule : chaque état est écrit. Un profil
/// `unsupported` n'est pas seulement grisé, il annonce « Incompatible » et la raison
/// calculée côté Rust.
const COMPATIBILITY: Record<ModelCompatibility, { label: string; tone: Tone }> = {
  optimal: { label: "Recommandé pour votre ordinateur", tone: "accent" },
  supported: { label: "Compatible", tone: "success" },
  not_recommended: { label: "Déconseillé", tone: "warning" },
  unsupported: { label: "Incompatible", tone: "danger" },
};

const BACKEND_LABELS: Record<LocalAiBackend, string> = {
  metal: "Metal",
  cuda: "CUDA",
  vulkan: "Vulkan",
  cpu: "CPU",
};

export function MistralLocalPanel({ vm }: { vm: LocalAiViewModel }) {
  const [installCandidate, setInstallCandidate] = useState<LocalModelDefinition | null>(null);
  const [removeCandidate, setRemoveCandidate] = useState<LocalModelDefinition | null>(null);
  const recommended = vm.recommendation?.selected_model ?? null;
  const active = vm.status?.active_model ?? null;
  const tooSlow = vm.status?.benchmark?.rating === "too_slow";
  const downgrade = active && tooSlow
    ? findDowngrade(active, vm.recommendation?.evaluations ?? [])
    : null;

  return (
    <div className="flex flex-col gap-4" data-local-ai-state={vm.state}>
      {vm.state === "detecting_hardware" ? <Detecting /> : null}
      {vm.error && vm.state === "error" ? (
        <ErrorBanner message={vm.error} onRetry={vm.reevaluate} />
      ) : null}
      {vm.recommendation && !recommended && !isLocalAiBusy(vm.state) ? (
        <Unsupported onReevaluate={vm.reevaluate} />
      ) : null}
      {recommended && !active && !isLocalAiBusy(vm.state) && vm.state !== "error" ? (
        <Recommendation model={recommended} onInstall={() => setInstallCandidate(recommended)} />
      ) : null}
      {isLocalAiBusy(vm.state) ? (
        <Installation state={vm.state} model={recommended} progress={vm.progress} onCancel={vm.cancel} />
      ) : null}
      {vm.recommendation && vm.recommendation.evaluations.length > 0 && !isLocalAiBusy(vm.state) ? (
        <ModelChoice
          evaluations={vm.recommendation.evaluations}
          activeId={active?.id ?? null}
          tooSlowActive={tooSlow}
          onInstall={setInstallCandidate}
        />
      ) : null}
      {vm.state === "ready" && active && vm.status ? (
        <Ready
          model={active}
          status={vm.status}
          testResult={vm.testResult}
          testing={vm.isTesting}
          onBenchmark={vm.benchmark}
          onRemove={setRemoveCandidate}
          onReevaluate={vm.reevaluate}
          tooSlow={tooSlow}
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
        <Metric
          label="Modèle recommandé"
          value={PROFILE_LABELS[model.profile]}
          icon={PROFILE_ICONS[model.profile]}
        />
        <Metric label="Qualité" value={model.profile === "light" ? "Bonne" : "Élevée"} />
        <Metric label="Poids sur le disque" value={formatBytes(model.download_size_bytes)} />
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
  progress: LocalAiViewModel["progress"];
  onCancel: () => void;
}) {
  const titles: Partial<Record<LocalAiState, string>> = {
    downloading: "Téléchargement de l'IA locale",
    verifying: "Vérification du téléchargement",
    installing: "Installation du modèle",
    benchmarking: "Mesure des performances",
  };
  const descriptions: Partial<Record<LocalAiState, string>> = {
    downloading: "Téléchargement en cours… Veuillez patienter.",
    verifying: "Contrôle de l'intégrité du fichier… Veuillez patienter.",
    installing: "Installation et préparation du modèle… Veuillez patienter.",
    benchmarking: "Mesure des performances en cours… Veuillez patienter.",
  };
  const title = titles[state] ?? "Préparation de l'IA locale";
  const description = descriptions[state] ?? "Traitement en cours… Veuillez patienter.";
  const showDownloadProgress = state === "downloading" && progress !== null;
  const percentage = progress?.progress ?? 0;
  return (
    <SettingsCard icon="progress_activity" title={title}>
      <div role="status" aria-live="polite" className="space-y-3">
        <div className="flex items-start gap-3">
          <Icon name="progress_activity" size={20} className="mt-0.5 flex-none animate-spin text-accent" />
          <p className="text-body text-ink-muted">{description}</p>
        </div>
        {showDownloadProgress ? (
          <>
            <div className="h-2 overflow-hidden rounded-full bg-fill">
              <div className="h-full rounded-full bg-accent transition-[width]" style={{ width: `${percentage}%` }} />
            </div>
            <div className="flex flex-wrap justify-between gap-2 font-mono text-note tabular-nums text-ink-muted">
              <span>{percentage} %</span>
              <span>
                {formatBytes(progress.downloaded_bytes)} / {formatBytes(progress.total_bytes)}
                {progress.bytes_per_second > 0 ? ` · ${formatBytes(progress.bytes_per_second)}/s` : ""}
              </span>
            </div>
          </>
        ) : (
          <div className="h-2 overflow-hidden rounded-full bg-fill">
            <div className="h-full w-1/3 animate-pulse rounded-full bg-accent" />
          </div>
        )}
        {state === "benchmarking" ? (
          <p className="text-note text-ink-faint">Test synthétique local, sans aucune donnée personnelle.</p>
        ) : null}
        {!showDownloadProgress && model ? (
          <p className="font-mono text-note text-ink-faint">{formatBytes(model.download_size_bytes)}</p>
        ) : null}
        {state === "downloading" ? (
          <Button className="mt-1" icon="close" onClick={onCancel}>Annuler</Button>
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
  onBenchmark,
  onRemove,
  onReevaluate,
  tooSlow,
  downgrade,
  onDowngrade,
}: {
  model: LocalModelDefinition;
  status: LocalAiStatus;
  testResult: string | null;
  testing: boolean;
  onBenchmark: () => void;
  onRemove: (model: LocalModelDefinition) => void;
  onReevaluate: () => void;
  tooSlow: boolean;
  downgrade: LocalModelDefinition | null;
  onDowngrade: () => void;
}) {
  return (
    <SettingsCard icon="check_circle" title="IA locale prête">
      <div className="grid gap-3 sm:grid-cols-4">
        <Metric
          label="Profil"
          value={PROFILE_LABELS[model.profile]}
          icon={PROFILE_ICONS[model.profile]}
        />
        <Metric
          label="Modèle"
          value={model.display_name.replace(" Instruct", "")}
          family={model.family}
        />
        <Metric label="Poids sur le disque" value={formatBytes(model.download_size_bytes)} />
        <Metric
          label="Vitesse"
          value={status.benchmark ? `${formatDecimal(status.benchmark.tokens_per_second)} tokens/s` : "À mesurer"}
        />
      </div>
      {model.profile === "ultra_light" ? (
        <div className="mt-4 rounded-field border border-line bg-fill px-3 py-3">
          <p className="text-body font-mid text-ink">Mode ultra léger</p>
          <p className="mt-1 text-note text-ink-muted">
            Votre ordinateur utilise un modèle optimisé pour les configurations disposant de moins
            de mémoire. La qualité des générations peut être légèrement inférieure.
          </p>
        </div>
      ) : null}
      <div className="mt-5 flex flex-wrap gap-2">
        <Button icon="delete" disabled={testing} onClick={() => onRemove(model)}>Supprimer le modèle</Button>
        <Button icon="refresh" disabled={testing} onClick={onReevaluate}>Réévaluer ma configuration</Button>
      </div>
      {testResult ? (
        // Le texte renvoyé est la prose du modèle, pas un état : interrogé sur
        // « l'assistance locale », il répondait « Notre équipe d'assistance locale est
        // entièrement opérationnelle… ». Seul son aboutissement fait un message d'état.
        <p role="status" className="mt-3 text-body text-success">
          Votre modèle local est installé et opérationnel.
        </p>
      ) : null}
      {tooSlow ? (
        <div className="mt-4 rounded-field border border-warning bg-warning-tint px-3 py-3">
          <p className="text-body font-mid text-ink">Ce profil est trop lent sur votre ordinateur.</p>
          {downgrade ? (
            <>
              <p className="mt-1 text-note text-ink-muted">
                Le profil {PROFILE_LABELS[downgrade.profile]} devrait être plus fluide. Il ne sera téléchargé qu'après votre confirmation.
              </p>
              <Button className="mt-3" icon="download" onClick={onDowngrade}>
                Passer au profil {PROFILE_LABELS[downgrade.profile]}
              </Button>
            </>
          ) : (
            <p className="mt-1 text-note text-ink-muted">
              {status.benchmark
                ? `À ${formatDecimal(status.benchmark.tokens_per_second)} tokens/s, l'analyse d'un CV demande environ ${estimationAnalyseMinutes(status.benchmark.tokens_per_second)} minutes.`
                : "L'analyse d'un CV sera très longue."}{" "}
              Aucun profil plus léger n'est disponible : pour un usage confortable, choisissez un
              fournisseur IA distant dans les réglages.
            </p>
          )}
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
          <Advanced label="Famille" value={FAMILY_LOGOS[model.family].label} />
          <Advanced label="Quantification" value={model.quantization} />
          <Advanced label="Runtime" value={model.runtime} />
          <Advanced label="Backend" value={status.backend ? BACKEND_LABELS[status.backend] : "CPU"} />
          <Advanced label="Contexte" value={`${model.context_size} tokens`} />
          <Advanced label="RAM estimée" value={formatGoFromMb(model.estimated_ram_mb)} />
        </dl>
        <Button className="mt-3" icon="refresh" disabled={testing} onClick={onBenchmark}>
          Relancer le benchmark
        </Button>
      </details>
    </SettingsCard>
  );
}

/// Liste des profils évalués, avec l'état calculé par le backend et la raison qui l'explique.
///
/// L'écran ne recalcule pas la RAM : `compatibility` et `reason` viennent de
/// `LocalModelSelector`. En revanche un benchmark `too_slow` sur le profil actif
/// prime sur un `optimal` matériel — la machine « tenait » sur le papier, pas à l'usage.
function ModelChoice({
  evaluations,
  activeId,
  tooSlowActive,
  onInstall,
}: {
  evaluations: LocalModelEvaluation[];
  activeId: string | null;
  tooSlowActive: boolean;
  onInstall: (model: LocalModelDefinition) => void;
}) {
  const ordered = [...evaluations].sort(
    (a, b) => a.model.download_size_bytes - b.model.download_size_bytes,
  );
  return (
    <SettingsCard icon="tune" title="Profils disponibles">
      <ul className="flex flex-col gap-2">
        {ordered.map((evaluation) => {
          const { model, reason: hardwareReason } = evaluation;
          const measuredTooSlow = tooSlowActive && model.id === activeId;
          const compatibility = measuredTooSlow ? "not_recommended" : evaluation.compatibility;
          const reason = measuredTooSlow
            ? "Mesuré trop lent sur votre ordinateur."
            : hardwareReason;
          const actif = model.id === activeId;
          // Seuls optimal et supported restent installables : déconseillé charge encore
          // mais sans marge, incompatible ne tient pas en mémoire. Un profil déjà mesuré
          // trop lent est traité comme déconseillé.
          const installable = compatibility === "optimal" || compatibility === "supported";
          const logo = FAMILY_LOGOS[model.family];
          const etat = COMPATIBILITY[compatibility];
          return (
            <li
              key={model.id}
              className={cn(
                "flex flex-wrap items-center gap-x-3 gap-y-2 rounded-field border px-3 py-2.5",
                actif ? "border-accent bg-accent-tint-12" : "border-line bg-fill",
                !installable && "opacity-70",
              )}
            >
              <img
                src={logo.src}
                alt=""
                data-family={model.family}
                width={18}
                height={18}
                className={cn("size-4.5 flex-none", logo.mono && "dark:invert")}
              />
              <div className="min-w-0 flex-1">
                <p className="flex min-w-0 items-center gap-1.5 text-body font-mid text-ink">
                  <Icon
                    name={PROFILE_ICONS[model.profile]}
                    size={16}
                    className="flex-none text-ink-muted"
                  />
                  <span className="truncate">
                    {PROFILE_LABELS[model.profile]} · {model.display_name.replace(" Instruct", "")}
                  </span>
                </p>
                <p className="mt-0.5 text-note text-ink-muted">
                  {formatBytes(model.download_size_bytes)} sur le disque
                  {" · "}
                  RAM reco {formatGoFromMb(model.recommended_ram_mb)}
                  {" · "}
                  VRAM reco {formatGoFromMb(model.recommended_vram_mb ?? model.recommended_ram_mb)}
                  {" · "}
                  {model.recommended_cores} cœurs
                </p>
                <p className="mt-0.5 text-note text-ink-muted">{reason}</p>
              </div>
              <StatusPill tone={etat.tone}>{etat.label}</StatusPill>
              {actif ? (
                <StatusPill tone="success" icon="check_circle">
                  Profil actif
                </StatusPill>
              ) : (
                <Button
                  icon="download"
                  disabled={!installable}
                  onClick={() => onInstall(model)}
                >
                  Installer le profil {PROFILE_LABELS[model.profile]}
                </Button>
              )}
            </li>
          );
        })}
      </ul>
    </SettingsCard>
  );
}

function Metric({
  label,
  value,
  family,
  icon,
}: {
  label: string;
  value: string;
  family?: LocalModelFamily;
  icon?: IconName;
}) {
  const logo = family ? FAMILY_LOGOS[family] : null;
  return (
    <div className="rounded-field bg-fill px-3 py-2.5">
      <p className="text-meta text-ink-faint">{label}</p>
      <p className="mt-1 flex min-w-0 items-center gap-1.5 text-item font-semibold text-ink">
        {icon ? (
          <Icon name={icon} size={16} className="flex-none text-ink-muted" />
        ) : null}
        {logo ? (
          // Décoratif : le nom du modèle est écrit juste à côté, un `alt` le ferait
          // annoncer deux fois.
          <img
            src={logo.src}
            alt=""
            data-family={family}
            width={16}
            height={16}
            className={`size-4 flex-none ${logo.mono ? "dark:invert" : ""}`}
          />
        ) : null}
        <span className="truncate">{value}</span>
      </p>
    </div>
  );
}

function Advanced({ label, value }: { label: string; value: string }) {
  return <div className="flex justify-between gap-3 border-b border-line-soft py-1"><dt className="text-ink-faint">{label}</dt><dd className="font-mono text-right text-ink">{value}</dd></div>;
}

function formatBytes(bytes: number): string {
  if (bytes < 1_000_000_000) return `${(bytes / 1_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 0 })} Mo`;
  return `${(bytes / 1_000_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} Go`;
}

/// Même convention que le sélecteur Rust : Mo internes → Go (base 1024), virgule française.
function formatGoFromMb(mb: number): string {
  const tenths = Math.round((mb / 1024) * 10) / 10;
  if (Number.isInteger(tenths)) {
    return `${tenths} Go`;
  }
  return `${tenths.toLocaleString("fr-FR", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} Go`;
}

function formatDecimal(value: number): string {
  return value.toLocaleString("fr-FR", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
}

/// Durée indicative d'une analyse de CV, arrondie à la minute.
///
/// Fondée sur la taille réellement mesurée des profils générés — de 336 à 1 928 jetons
/// selon les cas de `tests/fixtures/profiles/` — dont on retient un ordre de grandeur
/// médian. Le but est de rendre l'attente prévisible, pas de la chronométrer.
function estimationAnalyseMinutes(tokensPerSecond: number): number {
  const JETONS_TYPIQUES = 1_500;
  if (tokensPerSecond <= 0) return 0;
  return Math.max(1, Math.round(JETONS_TYPIQUES / tokensPerSecond / 60));
}

function findDowngrade(
  active: LocalModelDefinition,
  evaluations: LocalModelEvaluation[],
): LocalModelDefinition | null {
  const target: Partial<Record<LocalModelProfile, LocalModelProfile>> = {
    quality: "balanced",
    balanced: "light",
    light: "ultra_light",
  };
  const profile = target[active.profile];
  if (!profile) return null;
  const candidate = evaluations.find((item) => item.model.profile === profile);
  if (!candidate) return null;
  // Même règle que la liste : pas de repli vers un profil déconseillé ou incompatible.
  if (candidate.compatibility !== "optimal" && candidate.compatibility !== "supported") {
    return null;
  }
  return candidate.model;
}
