import { useState } from "react";
import type { ReactNode } from "react";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { Button, PaneSection, WorkSurface } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { formatDuration } from "@/shared/lib/duration";
import { ManagedPublisherLogo } from "./ProviderGrid";
import {
  formatBytes,
  formatRemaining,
  MACHINE_FIT_LABELS,
  modelMeters,
  stepState,
  throughput,
} from "../../model/localInstall";
import type { InstallStepState } from "../../model/localInstall";
import { useLocalInstall } from "../../viewmodel/useLocalInstall";
import type { ManagedOllamaViewModel } from "../../viewmodel/useManagedOllamaViewModel";

/** Un modèle se choisit s'il n'est pas déjà là et que la machine peut le porter. */
function installable(model: ManagedModelStatus): boolean {
  return !model.installed && model.machine_fit !== "insufficient_memory";
}

function defaultChoice(
  models: readonly ManagedModelStatus[],
): ManagedModelStatus | null {
  return (
    models.find((model) => model.recommended && installable(model)) ??
    models.find(installable) ??
    null
  );
}

/**
 * Installer l'IA locale (`screens/14-ai-install-local.png`, `INTERACTIONS.md` §4.3) :
 * choisir un modèle, suivre le moteur, le modèle et la phrase de test, puis commencer à
 * l'utiliser. Fermer pendant le déroulé n'interrompt rien : le panneau de l'écran IA
 * continue d'afficher la progression, et un toast annonce la fin.
 */
export function LocalInstallOverlay({
  vm,
  onClose,
}: {
  vm: ManagedOllamaViewModel;
  onClose: () => void;
}) {
  const flow = useLocalInstall(vm);
  const models = vm.status?.models ?? [];
  const [chosenId, setChosenId] = useState(
    () => defaultChoice(models)?.definition.id ?? null,
  );
  const chosen =
    models.find((model) => model.definition.id === chosenId) ??
    defaultChoice(models);
  const size = chosen
    ? formatBytes(chosen.definition.approximate_download_bytes)
    : "";

  const install = () => {
    if (chosen && installable(chosen)) flow.start(chosen.definition);
  };

  const progress = vm.progress;
  const rate =
    flow.current === "model" && progress
      ? throughput(
          progress.downloaded_bytes,
          progress.total_bytes,
          flow.stepElapsedMs,
        )
      : null;
  const engineBytes =
    progress?.kind === "runtime" && progress.total_bytes > 0
      ? progress.total_bytes
      : null;

  const status =
    flow.phase === "done"
      ? `${flow.installed?.ollama_tag ?? ""} · actif${flow.probe ? ` · test ${flow.probe.latency_ms} ms` : ""}`
      : flow.phase === "run"
        ? flow.current === "verify"
          ? "vérification · phrase de test"
          : `${flow.current === "model" ? "téléchargement du modèle" : "moteur"}${progress && progress.total_bytes > 0 ? ` · ${progress.progress} %` : ""}`
        : chosen && installable(chosen)
          ? `prêt à installer · ${size} à télécharger`
          : "aucun modèle à installer";

  return (
    <WorkSurface
      crumbRoot="Intelligence artificielle"
      title="Installer l’IA locale"
      onClose={onClose}
      status={status}
      {...(flow.phase === "run"
        ? {}
        : { onSubmit: flow.phase === "done" ? onClose : install })}
      actions={
        flow.phase === "done" ? (
          <Button
            variant="primary"
            size="bar"
            shortcut="mod+enter"
            onClick={onClose}
          >
            Commencer à l’utiliser
          </Button>
        ) : flow.phase === "run" ? (
          <Button
            size="bar"
            disabled={flow.current === "verify"}
            onClick={flow.cancel}
          >
            Annuler l’installation
          </Button>
        ) : (
          <Button
            variant="primary"
            size="bar"
            shortcut="mod+enter"
            disabled={!chosen || !installable(chosen)}
            onClick={install}
          >
            Installer{size ? ` · ${size}` : ""}
          </Button>
        )
      }
      left={
        <>
          <PaneSection title="Ce que fait Candilog">
            <ol className="flex flex-col gap-2.5">
              <Step
                state={
                  flow.phase === "done"
                    ? "done"
                    : stepState("engine", flow.current)
                }
                label="Installer le moteur"
                detail="Ollama, dans le dossier de Candilog"
                aside={
                  durationLabel(flow.durationOf("engine")) ??
                  (engineBytes ? formatBytes(engineBytes) : null)
                }
              />
              <Step
                state={
                  flow.phase === "done"
                    ? "done"
                    : stepState("model", flow.current)
                }
                label="Télécharger le modèle"
                detail={`${chosen?.definition.display_name ?? "Le modèle"} depuis le registre Ollama`}
                aside={size || null}
              />
              <Step
                state={
                  flow.phase === "done" && flow.probe
                    ? "done"
                    : stepState("verify", flow.current)
                }
                label="Vérifier"
                detail="une phrase de test, pour être sûr"
                aside={flow.probe ? `${flow.probe.latency_ms} ms` : null}
              />
            </ol>
          </PaneSection>
          <p className="mb-4 rounded-r8 bg-tint-g-bg px-3 py-2.5 text-small leading-[1.5] text-tint-g-tx">
            Le moteur et les modèles s’installent dans le dossier de Candilog,
            pas dans votre système. Aucune commande à taper.
          </p>
          {chosen ? (
            <dl className="grid grid-cols-[76px_1fr] gap-x-2 gap-y-2 text-small">
              <Fact label="Modèle">{chosen.definition.display_name}</Fact>
              <Fact label="Éditeur">{chosen.definition.publisher_label}</Fact>
              <Fact label="Taille">
                <span className="font-mono">{size}</span>
              </Fact>
              <Fact label="Requis">
                {chosen.definition.recommended_ram_gb} Go de mémoire
              </Fact>
              <Fact label="Ici">{MACHINE_FIT_LABELS[chosen.machine_fit]}</Fact>
              {chosen.last_benchmark ? (
                <Fact label="Dernier test">
                  {formatDuration(chosen.last_benchmark.total_ms)}
                </Fact>
              ) : null}
            </dl>
          ) : null}
        </>
      }
    >
      {/* Pas de feuille ici : le fond de bureau des générateurs céderait la place à l'écran. */}
      <div className="flex-1 bg-app">
        <div className="w-full max-w-[640px] px-8 pt-7 pb-8">
          {flow.phase === "done" && flow.installed ? (
            <>
              <h1 className="serif-title text-[21px] leading-tight text-tx">
                Installation terminée
              </h1>
              <p className="mt-1.5 text-small leading-[1.55] text-tx-4">
                {flow.probe
                  ? "Le modèle répond. Candilog l’utilisera pour toutes les tâches qui suivent le fournisseur principal."
                  : "Le modèle est installé, mais la phrase de test n’a pas abouti."}
              </p>
              <div
                role="status"
                className="mt-4 flex items-start gap-2.5 rounded-r9 bg-tint-g-bg px-3.5 py-3 text-tint-g-tx"
              >
                <span
                  aria-hidden
                  className="mt-[5px] size-[9px] flex-none rounded-full bg-st-g"
                />
                <span>
                  <span className="block text-ui font-medium">
                    {flow.installed.display_name} est prêt
                  </span>
                  <span className="mt-0.5 block text-small opacity-80">
                    Installé dans le dossier de Candilog ·{" "}
                    {formatBytes(flow.installed.approximate_download_bytes)} sur
                    le disque
                  </span>
                </span>
              </div>
              <dl className="mt-4 grid grid-cols-[150px_1fr] gap-x-3 gap-y-2.5 text-small">
                <Fact label="Phrase de test">
                  {flow.verifying ? (
                    <span className="text-tx-4">Vérification…</span>
                  ) : flow.probe ? (
                    <span className="text-tint-g-tx">
                      répondue en {flow.probe.latency_ms} ms
                    </span>
                  ) : (
                    <span className="flex flex-col items-start gap-1">
                      <span className="text-st-c">{flow.probeError}</span>
                      <Button variant="link" onClick={flow.verify}>
                        Refaire le test
                      </Button>
                    </span>
                  )}
                </Fact>
                <Fact label="Utilisé pour">les tâches sans modèle attitré</Fact>
                <Fact label="Confidentialité">
                  aucune donnée ne quitte l’ordinateur
                </Fact>
              </dl>
              <p className="mt-5 text-small text-tx-5">
                Vous pourrez installer un autre modèle à tout moment depuis
                Intelligence artificielle, et choisir lequel sert à quelle
                tâche.
              </p>
            </>
          ) : (
            <>
              <h1 className="serif-title text-[21px] leading-tight text-tx">
                {flow.phase === "run"
                  ? "Installation en cours"
                  : "Choisissez un modèle à installer"}
              </h1>
              <p className="mt-1.5 text-small leading-[1.55] text-tx-4">
                {flow.phase === "run"
                  ? "Le moteur et le modèle se téléchargent depuis leurs sources officielles. Vous pouvez annuler à tout moment."
                  : "Candilog installe et gère le moteur lui-même : aucune commande à taper, rien à configurer. Les modèles tournent entièrement sur votre ordinateur."}
              </p>

              <div
                role="radiogroup"
                aria-label="Modèle à installer"
                className="mt-4 flex flex-col gap-2"
              >
                {models.map((model) => (
                  <ModelChoice
                    key={model.definition.id}
                    model={model}
                    checked={chosen?.definition.id === model.definition.id}
                    disabled={flow.phase === "run" || !installable(model)}
                    onChoose={() => setChosenId(model.definition.id)}
                  />
                ))}
              </div>

              {flow.phase === "run" ? (
                <div
                  role="status"
                  aria-live="polite"
                  className="mt-4 rounded-r9 bg-group px-3.5 py-3"
                >
                  <p className="text-small text-tx-2">
                    {flow.current === "verify"
                      ? "Envoi de la phrase de test…"
                      : (progress?.label ?? "Préparation…")}
                  </p>
                  <div
                    aria-hidden
                    className="mt-2 h-1 overflow-hidden rounded-r2 bg-chip"
                  >
                    {progress &&
                    progress.total_bytes > 0 &&
                    flow.current !== "verify" ? (
                      <div
                        className="h-full rounded-r2 bg-ac transition-[width]"
                        style={{ width: `${progress.progress}%` }}
                      />
                    ) : (
                      <div className="import-indeterminate h-full w-1/3 rounded-r2 bg-ac" />
                    )}
                  </div>
                  {progress &&
                  progress.total_bytes > 0 &&
                  flow.current !== "verify" ? (
                    <p className="mt-1.5 font-mono text-caps text-tx-5">
                      {formatBytes(progress.downloaded_bytes)} /{" "}
                      {formatBytes(progress.total_bytes)}
                      {rate
                        ? ` · ${formatBytes(rate.bytesPerSecond)}/s · reste ${formatRemaining(rate.remainingSeconds)}`
                        : ""}
                    </p>
                  ) : null}
                </div>
              ) : (
                <p className="mt-3 text-small text-tx-5">
                  Vous pourrez installer les autres modèles plus tard, et
                  choisir lequel sert à quelle tâche.
                </p>
              )}

              {flow.outcome ? (
                <div
                  role="alert"
                  className="mt-4 rounded-r9 bg-tint-c-bg px-3.5 py-3 text-tint-c-tx"
                >
                  <p className="flex items-start gap-2 text-small">
                    <span
                      aria-hidden
                      className="mt-[5px] size-[7px] flex-none rounded-full bg-st-c"
                    />
                    {flow.outcome.kind === "cancelled"
                      ? "Installation annulée. Vous pouvez la reprendre quand vous voulez."
                      : flow.outcome.message}
                  </p>
                  <Button
                    className="mt-2.5"
                    variant="primary"
                    size="bar"
                    disabled={!chosen || !installable(chosen)}
                    onClick={install}
                  >
                    {flow.outcome.kind === "cancelled"
                      ? "Reprendre l’installation"
                      : "Réessayer"}
                  </Button>
                </div>
              ) : null}
            </>
          )}
        </div>
      </div>
    </WorkSurface>
  );
}

function durationLabel(ms: number | null): string | null {
  return ms === null ? null : `${Math.max(1, Math.round(ms / 1000))} s`;
}

function Step({
  state,
  label,
  detail,
  aside,
}: {
  state: InstallStepState;
  label: string;
  detail: string;
  aside: string | null;
}) {
  return (
    <li className="flex gap-2.5">
      <span
        aria-hidden
        className={cn(
          "mt-[4px] size-[9px] flex-none rounded-full",
          state === "done" && "bg-st-g",
          state === "running" &&
            "animate-pulse border-[1.5px] border-ac bg-tint-ac-bg",
          state === "pending" && "border-[1.5px] border-tx-6",
        )}
      />
      <span className="min-w-0 flex-1">
        <span className="flex items-baseline gap-2">
          <span
            className={cn(
              "text-small",
              state === "pending" ? "text-tx-3" : "text-tx",
            )}
          >
            {label}
          </span>
          {aside ? (
            <span className="ml-auto font-mono text-caps text-tx-5">
              {aside}
            </span>
          ) : null}
        </span>
        <span className="mt-0.5 block text-tiny text-tx-5">{detail}</span>
      </span>
      <span className="sr-only">
        {state === "done"
          ? "terminée"
          : state === "running"
            ? "en cours"
            : "à venir"}
      </span>
    </li>
  );
}

function Fact({ label, children }: { label: string; children: ReactNode }) {
  return (
    <>
      <dt className="text-tx-5">{label}</dt>
      <dd className="min-w-0 text-tx-2">{children}</dd>
    </>
  );
}

function Meter({ label, level }: { label: string; level: number }) {
  return (
    <span className="flex items-center gap-1.5">
      <span className="text-tiny text-tx-5">{label}</span>
      <span aria-hidden className="flex gap-[3px]">
        {[0, 1, 2].map((index) => (
          <span
            key={index}
            className={cn(
              "h-[3px] w-[9px] rounded-r2",
              index < level ? "bg-ac" : "bg-chip",
            )}
          />
        ))}
      </span>
      <span className="sr-only">{level} sur 3</span>
    </span>
  );
}

function ModelChoice({
  model,
  checked,
  disabled,
  onChoose,
}: {
  model: ManagedModelStatus;
  checked: boolean;
  disabled: boolean;
  onChoose: () => void;
}) {
  const meters = modelMeters(model.definition);
  const note = model.installed
    ? "installé"
    : model.machine_fit === "insufficient_memory"
      ? "mémoire insuffisante"
      : null;
  return (
    <button
      type="button"
      role="radio"
      aria-checked={checked}
      disabled={disabled && !checked}
      onClick={onChoose}
      className={cn(
        "flex w-full gap-3 rounded-r9 border px-3.5 py-3 text-left",
        checked
          ? "border-ac bg-tint-ac-bg"
          : "border-transparent bg-panel hover:bg-hover",
        disabled && !checked && "opacity-60",
      )}
    >
      <span
        aria-hidden
        className="flex size-7 flex-none items-center justify-center rounded-r7 bg-chip"
      >
        <ManagedPublisherLogo
          publisher={model.definition.publisher}
          className="size-4"
        />
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex items-center gap-2">
          <span className="text-ui font-medium text-tx">
            {model.definition.display_name}
          </span>
          {model.recommended && !model.installed ? (
            <span className="rounded-r5 bg-tint-g-bg px-1.5 text-tiny text-tint-g-tx">
              recommandé
            </span>
          ) : null}
          {note ? (
            <span className="rounded-r5 bg-chip px-1.5 text-tiny text-tx-4">
              {note}
            </span>
          ) : null}
          <span className="ml-auto font-mono text-caps text-tx-5">
            {formatBytes(model.definition.approximate_download_bytes)}
          </span>
          <span
            aria-hidden
            className={cn(
              "flex size-[13px] flex-none items-center justify-center rounded-full border-[1.5px]",
              checked ? "border-ac" : "border-tx-6",
            )}
          >
            {checked ? (
              <span className="size-[7px] rounded-full bg-ac" />
            ) : null}
          </span>
        </span>
        {model.definition.description ? (
          <span className="mt-1 block text-small leading-[1.5] text-tx-3">
            {model.definition.description}
          </span>
        ) : null}
        <span className="mt-2 flex flex-wrap gap-x-4 gap-y-1">
          <Meter label="vitesse" level={meters.speed} />
          <Meter label="qualité" level={meters.quality} />
          <Meter label="mémoire" level={meters.memory} />
        </span>
      </span>
    </button>
  );
}
