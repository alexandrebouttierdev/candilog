import { useState, type ReactNode } from "react";
import { AiBenchmarkModal, useAiRailStatusStore } from "@/features/ai";
import { AppError } from "@/shared/types/app-error";
import type { AiTask, AnalysisMode, LlmForm, Settings, TaskRoute } from "@/shared/types/generated/settings";
import {
  Button,
  ErrorBanner,
  FormField,
  Icon,
  SegmentedControl,
  Skeleton,
  TextInput,
} from "@/shared/ui";
import { useChrome } from "@/shared/lib/chrome";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { useUiStore } from "@/shared/lib/ui-store";
import { openExternal } from "@/shared/services/external-link";
import { useSettingsViewModel } from "../../viewmodel/useSettingsViewModel";
import {
  getProvider,
  OLLAMA_PROVIDER,
  OTHER_PROVIDERS,
  PROVIDERS,
  idProvider,
  llmFromPreset,
  presetFromLlm,
  type ProviderOption,
} from "../../model/providers";
import { ManagedOllamaPanel } from "../components/ManagedOllamaPanel";
import { LocalInstallOverlay } from "../components/LocalInstallOverlay";
import { AiTaskRouting } from "../components/AiTaskRouting";
import { AI_TASKS, assignmentOf, mainLabel } from "../../model/taskRouting";
import { RemoteModelPicker } from "../components/RemoteModelPicker";
import { cn } from "@/shared/lib/cn";
import { type ConnectionTest } from "../../model/aiStatus";
import { useManagedOllamaViewModel } from "../../viewmodel/useManagedOllamaViewModel";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";

const MODES: Array<{ value: AnalysisMode; label: string }> = [
  { value: "auto", label: "Auto" },
  { value: "small", label: "Petit" },
  { value: "standard", label: "Standard" },
  { value: "advanced", label: "Avancé" },
];

type AiTab = "local" | "providers";

/** Intelligence artificielle : fournisseur, modèle, comportement et apparence. */
export function AiPage() {
  const vm = useSettingsViewModel();
  const notify = useUiStore((state) => state.notify);
  // Sans choix explicite, l'écran s'ouvre sur le fournisseur principal.
  const [chosenTab, setTab] = useState<AiTab | null>(null);
  const [draft, setDraft] = useState<Settings | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  /** Brouillons de clé par fournisseur : basculer ne doit pas écraser la saisie en cours. */
  const [apiKeyByProvider, setApiKeyByProvider] = useState<Record<string, string>>({});
  const [models, setModels] = useState<string[]>([]);
  const [test, setTestState] = useState<ConnectionTest>("idle");
  const [testMessage, setTestMessage] = useState<string | null>(null);
  const [benchmarkOpen, setBenchmarkOpen] = useState(false);
  const [installOpen, setInstallOpen] = useState(false);
  const [benchmarkModelLabel, setBenchmarkModelLabel] = useState("");
  const form = draft ?? vm.data ?? null;
  const tab: AiTab =
    chosenTab ?? (vm.data && idProvider(vm.data.llm.provider) !== "candilog_local" ? "providers" : "local");
  const llm = form?.llm;
  const managedVm = useManagedOllamaViewModel(() => setDraft(null));

  const setTest = (value: ConnectionTest) => {
    setTestState(value);
    useAiRailStatusStore.getState().setConnectionTest(value);
  };

  const patchLlm = (partiel: Partial<LlmForm>) => {
    setDraft((current) => {
      const base = current ?? vm.data;
      return base ? { ...base, llm: { ...base.llm, ...partiel } } : current;
    });
    setTest("idle");
  };

  const selectProvider = (id: ProviderOption["id"]) => {
    setDraft((current) => {
      const base = current ?? vm.data;
      if (!base) return current;
      const fromId = idProvider(base.llm.provider);
      const presets = {
        ...base.llm_presets,
        [fromId]: presetFromLlm(base.llm),
      };
      return {
        ...base,
        llm_presets: presets,
        llm: llmFromPreset(id, presets[id], {
          temperature: base.llm.temperature,
          mode: base.llm.mode,
        }),
      };
    });
    setApiKeyByProvider((keys) => {
      const fromId = llm ? idProvider(llm.provider) : null;
      const next = fromId ? { ...keys, [fromId]: apiKeyDraft } : { ...keys };
      return next;
    });
    setApiKeyDraft(() => {
      const fromId = llm ? idProvider(llm.provider) : null;
      const nextKeys = fromId
        ? { ...apiKeyByProvider, [fromId]: apiKeyDraft }
        : apiKeyByProvider;
      return nextKeys[id] ?? "";
    });
    setModels([]);
    setTest("idle");
  };

  const save = async () => {
    if (!form) return;
    const id = idProvider(form.llm.provider);
    const toSave: Settings = {
      ...form,
      llm_presets: {
        ...form.llm_presets,
        [id]: presetFromLlm(form.llm),
      },
    };
    await vm.save(toSave, apiKeyDraft.trim() || null);
    setDraft(null);
    setApiKeyDraft("");
    setApiKeyByProvider((keys) => {
      const next = { ...keys };
      delete next[id];
      return next;
    });
  };

  const runTest = async () => {
    if (!llm) return;
    setTest("pending");
    setTestMessage(null);
    try {
      await vm.testConnection(llm, apiKeyDraft.trim() || null);
      setTest("ok");
      setTestMessage("Connexion établie.");
    } catch (error) {
      setTest("error");
      setTestMessage(error instanceof AppError ? error.message : "Connexion impossible.");
    }
  };

  const actualiserModels = async () => {
    if (!llm) return;
    try {
      const list = await vm.listModels(llm, apiKeyDraft.trim() || null);
      setModels(list);
      if (llm.model.trim().length > 0 && list.length > 0 && !list.includes(llm.model)) {
        patchLlm({ model: "" });
      }
    } catch (error) {
      setTest("error");
      setTestMessage(error instanceof AppError ? error.message : "Modèles inaccessibles.");
    }
  };

  const clearApiKey = async () => {
    await vm.clearApiKey();
    setApiKeyDraft("");
    setApiKeyByProvider((keys) => {
      if (!llm) return keys;
      const next = { ...keys };
      delete next[idProvider(llm.provider)];
      return next;
    });
    setDraft((current) => {
      const base = current ?? vm.data;
      if (!base) return current;
      const id = idProvider(base.llm.provider);
      return {
        ...base,
        llm: { ...base.llm, api_key_configured: false },
        llm_presets: {
          ...base.llm_presets,
          [id]: {
            ...presetFromLlm({ ...base.llm, api_key_configured: false }),
          },
        },
      };
    });
  };

  const fournisseur = llm ? getProvider(llm.provider) : null;
  const managedActive = managedVm.status?.active_model ?? null;
  const managedModelLabel = managedActive?.display_name ?? "";

  const ouvrirBenchmark = (label: string) => {
    setBenchmarkModelLabel(label);
    setBenchmarkOpen(true);
  };

  const testerModeleLocal = async (model: ManagedModelStatus) => {
    if (!model.installed) return;
    if (!model.active) {
      await managedVm.activateAsync(model.definition.id);
    }
    ouvrirBenchmark(model.definition.display_name);
  };

  const saved = vm.data ?? null;
  const localModels = managedVm.status?.models ?? [];
  const installedCount = localModels.filter((model) => model.installed).length;
  const selectedId: ProviderOption["id"] = tab === "local" ? "candilog_local" : llm ? idProvider(llm.provider) : "candilog_local";
  const selected = selectedId === "candilog_local" ? PROVIDERS[0]! : getProvider(llm?.provider ?? "candilog_local");
  const principalId = saved ? idProvider(saved.llm.provider) : null;
  const listed = [
    PROVIDERS[0]!,
    ...(principalId === "ollama" || saved?.llm_presets.ollama ? [OLLAMA_PROVIDER] : []),
    ...OTHER_PROVIDERS,
  ];

  const routes = saved?.ai_routes ?? {};
  const assignments = saved ? AI_TASKS.map((task) => assignmentOf(task.value, saved, localModels)) : [];
  const localTasks = assignments.filter((item) => item.locality === "local").length;
  const remoteTasks = assignments.filter((item) => item.locality === "remote").length;
  const offTasks = assignments.filter((item) => item.locality === "off").length;

  const assignTask = async (task: AiTask, route: TaskRoute | null | undefined) => {
    if (!saved) return;
    const next: Settings["ai_routes"] = { ...routes };
    if (route === undefined) delete next[task];
    else next[task] = route;
    await vm.save({ ...saved, ai_routes: next }, null);
    const label = AI_TASKS.find((item) => item.value === task)?.label ?? task;
    const target =
      route === undefined
        ? `${mainLabel(saved, localModels)} (par défaut)`
        : route === null
          ? "désactivée"
          : assignmentOf(task, { ...saved, ai_routes: next }, localModels).label;
    notify({ tone: "success", title: `${label} → ${target}` });
  };

  useChrome({
    crumb: selected.label,
    ...(saved ? { aside: `${mainLabel(saved, localModels)} · principal` } : {}),
    status: saved
      ? `5 tâches · ${localTasks} locale${localTasks > 1 ? "s" : ""} · ${remoteTasks} distante${remoteTasks > 1 ? "s" : ""}${offTasks ? ` · ${offTasks} désactivée${offTasks > 1 ? "s" : ""}` : ""}`
      : "",
    keys: tab === "providers" ? [{ label: "Tester", shortcut: "t" }] : [],
  });
  useShortcut("t", () => void runTest(), { enabled: tab === "providers" && test !== "pending" });

  const choose = (id: ProviderOption["id"]) => {
    if (id === "candilog_local") {
      setTab("local");
      return;
    }
    setTab("providers");
    if (llm && idProvider(llm.provider) !== id) selectProvider(id);
  };

  if (vm.error && !vm.data) {
    return (
      <div className="px-[18px] pt-4">
        <ErrorBanner
          message={vm.error instanceof AppError ? vm.error.message : "Les réglages n'ont pas pu être chargés."}
          onRetry={vm.reload}
        />
      </div>
    );
  }

  if (vm.isLoading || !form || !llm || !fournisseur || !saved) {
    return (
      <div className="max-w-[1000px] space-y-4 px-[18px] pt-4" role="status" aria-label="Chargement des réglages">
        <Skeleton className="h-[82px] w-full rounded-card" />
        <Skeleton className="h-[136px] w-full rounded-card" />
        <Skeleton className="h-60 w-full rounded-card" />
      </div>
    );
  }

  const facts = FACTS[selectedId](selected.label);

  return (
    <div className="flex h-full min-h-0">
      <nav aria-label="Fournisseurs" className="flex w-[190px] flex-none flex-col border-r border-bd-soft">
        <p className="caps px-3.5 pt-3.5 pb-2">Fournisseurs</p>
        <div role="tablist" aria-label="Fournisseurs d'IA" aria-orientation="vertical" className="flex-1 overflow-y-auto px-2">
          {listed.map((provider) => {
            const active = provider.id === selectedId;
            const preset = provider.id === principalId ? saved.llm : saved.llm_presets[provider.id];
            const state =
              provider.id === "candilog_local"
                ? installedCount > 0
                  ? { text: `Local · ${installedCount} modèle${installedCount > 1 ? "s" : ""}`, ready: true }
                  : { text: "Aucun modèle installé", ready: false }
                : provider.id === "ollama"
                  ? { text: preset?.model ? `Local · ${preset.model}` : "Non configuré", ready: Boolean(preset?.model) }
                  : preset?.api_key_configured
                    ? { text: "Clé enregistrée", ready: true }
                    : { text: "Aucune clé", ready: false };
            return (
              <button
                key={provider.id}
                type="button"
                role="tab"
                aria-selected={active}
                onClick={() => choose(provider.id)}
                className={cn(
                  "mb-px flex w-full items-center gap-2.5 rounded-r7 px-2 py-1.5 text-left",
                  active ? "bg-elev shadow-[inset_2px_0_0_var(--ac)]" : "hover:bg-hover",
                )}
              >
                <span className="min-w-0 flex-1">
                  <span className={cn("block truncate text-ui", active ? "font-medium text-tx" : "text-tx-2")}>
                    {provider.label}
                  </span>
                  <span className="block truncate text-tiny text-tx-5">
                    {state.text}
                    {provider.id === principalId ? " · principal" : ""}
                  </span>
                </span>
                <span aria-hidden className={cn("size-1.5 flex-none rounded-full", state.ready ? "bg-st-g" : "bg-tx-7")} />
              </button>
            );
          })}
        </div>
      </nav>

      <div className="min-w-0 flex-1 overflow-y-auto px-[22px] pt-[18px] pb-8">
        <div className="max-w-[760px]">
          <div className="flex items-start gap-3">
            <div className="min-w-0 flex-1">
              <h1 className="serif-title text-[21px] leading-tight text-tx">{selected.label}</h1>
              <p className="mt-1.5 max-w-[560px] text-small leading-[1.55] text-tx-4">{facts.description}</p>
            </div>
            {tab === "providers" ? (
              <span className="flex flex-none gap-1.5">
                <Button size="compact" disabled={test === "pending"} onClick={() => void runTest()}>
                  {test === "pending" ? "Test en cours…" : "Tester la connexion"}
                </Button>
                <Button variant="primary" size="compact" disabled={vm.isSaving} onClick={() => void save()}>
                  {vm.isSaving ? "Enregistrement…" : "Enregistrer"}
                </Button>
              </span>
            ) : null}
          </div>

          <dl className="mt-4 flex flex-wrap gap-x-9 gap-y-3">
            {facts.meters.map((meter) => (
              <div key={meter.label}>
                <div aria-hidden className="mb-1.5 flex gap-[3px]">
                  {[0, 1, 2].map((index) => (
                    <span key={index} className={cn("h-[3px] w-[13px] rounded-r2", index < meter.level ? meter.tone : "bg-chip")} />
                  ))}
                </div>
                <dt className="text-small text-tx-2">{meter.label}</dt>
                <dd className="text-tiny text-tx-5">{meter.value}</dd>
              </div>
            ))}
          </dl>

          <div className="mt-5 border-t border-bd-soft pt-4">
            {tab === "local" ? (
              <ManagedOllamaPanel
                vm={managedVm}
                onInstall={() => setInstallOpen(true)}
                onTestModel={(model) => void testerModeleLocal(model)}
              />
            ) : (
              <div className="flex flex-col gap-5">
                {principalId !== selectedId ? (
                  <p className="rounded-r8 bg-group px-3 py-2 text-small leading-[1.5] text-tx-3">
                    Enregistrer fait de {selected.label} le fournisseur principal : les tâches sans modèle attitré
                    l'utiliseront. Les tâches routées ci-dessous gardent leur modèle.
                  </p>
                ) : null}
                <section aria-label="Configuration">
                  <h2 className="caps mb-2.5">Configuration</h2>
                  {testMessage ? (
                    <p
                      role="status"
                      className={cn("mb-2.5 text-small leading-relaxed", test === "error" ? "text-st-c" : "text-tint-g-tx")}
                    >
                      {testMessage}
                    </p>
                  ) : null}
                  <div className="flex flex-col gap-3.5">
                    <div className="flex flex-col gap-2.5">
                      <div className="flex max-w-[380px] items-end gap-2">
                        <FormField label="Modèle" required className="flex-1">
                          {(props) => (
                            <TextInput
                              {...props}
                              value={llm.model}
                              onChange={(event) => patchLlm({ model: event.target.value })}
                              placeholder={
                                models.length > 0 ? "Choisissez ci-dessous ou saisissez un identifiant" : "Identifiant du modèle"
                              }
                            />
                          )}
                        </FormField>
                        <Button variant="secondary" icon="refresh" onClick={() => void actualiserModels()}>
                          Actualiser
                        </Button>
                        <Button
                          variant="secondary"
                          icon="bolt"
                          disabled={!llm.model.trim()}
                          onClick={() => ouvrirBenchmark(llm.model.trim())}
                        >
                          Tester
                        </Button>
                      </div>
                      {models.length > 0 ? (
                        <RemoteModelPicker
                          models={models}
                          value={llm.model}
                          onChange={(model) => patchLlm({ model })}
                          providerLabel={fournisseur.label}
                          providerId={idProvider(llm.provider)}
                        />
                      ) : (
                        <p className="text-sub leading-relaxed text-tx-5">
                          Actualisez la liste pour afficher les modèles proposés par le fournisseur.
                        </p>
                      )}
                    </div>
                    <FormField
                      label="Endpoint"
                      required={idProvider(llm.provider) === "custom"}
                      className="max-w-[380px]"
                      help={
                        idProvider(llm.provider) === "custom"
                          ? "Ollama local ou LM Studio : http://localhost:11434 (HTTP, sans /v1)."
                          : undefined
                      }
                    >
                      {(props) => (
                        <TextInput
                          {...props}
                          value={llm.endpoint ?? ""}
                          onChange={(event) => patchLlm({ endpoint: event.target.value || null })}
                        />
                      )}
                    </FormField>
                    {idProvider(llm.provider) !== "ollama" ? (
                      <FormField
                        label="Clé API"
                        required={idProvider(llm.provider) !== "custom"}
                        className="max-w-[380px]"
                        help={
                          llm.api_key_configured
                            ? "Une clé est configurée dans le coffre système. Saisissez-en une nouvelle uniquement pour la remplacer."
                            : "Stockée dans le coffre système, jamais renvoyée à l'interface ni écrite dans la base."
                        }
                      >
                        {(props) => (
                          <div className="flex items-center gap-2">
                            <TextInput
                              {...props}
                              type="password"
                              autoComplete="new-password"
                              value={apiKeyDraft}
                              placeholder={llm.api_key_configured ? "Clé configurée" : "Saisir la clé API"}
                              onChange={(event) => {
                                setApiKeyDraft(event.target.value);
                                setTest("idle");
                              }}
                            />
                            {llm.api_key_configured ? (
                              <Button
                                variant="ghost"
                                icon="delete"
                                disabled={vm.isClearingApiKey}
                                onClick={() => void clearApiKey()}
                              >
                                Supprimer
                              </Button>
                            ) : null}
                          </div>
                        )}
                      </FormField>
                    ) : (
                      <div className="max-w-[380px] rounded-r8 bg-group px-3 py-2.5">
                        <p className="flex items-center gap-1.5 text-small font-medium text-tx-2">
                          <Icon name="info" size={14} className="flex-none text-tx-5" />
                          Modèle local : aucune clé, aucune connexion
                        </p>
                        <p className="mt-1 text-sub leading-relaxed text-tx-4">
                          Besoin d'aide pour choisir un modèle compatible avec votre machine ?{" "}
                          <button
                            type="button"
                            onClick={() => void openExternal("https://www.canirun.ai/")}
                            className="text-ac-tx underline-offset-2 hover:underline"
                          >
                            canirun.ai
                          </button>
                        </p>
                      </div>
                    )}
                  </div>
                </section>

                <section aria-label="Génération">
                  <h2 className="caps mb-2.5">Génération</h2>
                  <div className="flex flex-col gap-4">
                    <div>
                      <ControlLabel>Mode d'analyse</ControlLabel>
                      <div className="flex">
                        <SegmentedControl
                          label="Mode d'analyse"
                          value={llm.mode}
                          onChange={(mode) => patchLlm({ mode })}
                          options={MODES}
                        />
                      </div>
                    </div>
                    <Temperature value={llm.temperature} onChange={(temperature) => patchLlm({ temperature })} />
                  </div>
                </section>
              </div>
            )}
          </div>

          <AiTaskRouting
            settings={saved}
            models={localModels}
            busy={vm.isSaving}
            onChange={(task, route) => void assignTask(task, route).catch(() => undefined)}
          />
        </div>
      </div>

      {installOpen ? <LocalInstallOverlay vm={managedVm} onClose={() => setInstallOpen(false)} /> : null}

      <AiBenchmarkModal
        open={benchmarkOpen}
        onClose={() => setBenchmarkOpen(false)}
        modelLabel={benchmarkModelLabel || managedModelLabel || "IA locale"}
      />
    </div>
  );
}

/** Faits affichés sous le nom d'un fournisseur : aucune promesse de qualité, des faits. */
interface ProviderFacts {
  readonly description: string;
  readonly meters: ReadonlyArray<{ label: string; value: string; level: number; tone: string }>;
}

const GOOD = "bg-st-g";
const WARN = "bg-st-a";

function remoteFacts(label: string): ProviderFacts {
  return {
    description: `Les données de chaque tâche confiée à ${label} lui sont envoyées pour être traitées. La rédaction est généralement plus rapide et plus fine qu'en local ; l'usage est facturé par ${label} selon votre compte.`,
    meters: [
      { label: "Confidentialité", value: `Envoyé à ${label}`, level: 1, tone: WARN },
      { label: "Coût", value: "Facturé à l'usage", level: 1, tone: WARN },
      { label: "Hors connexion", value: "Impossible", level: 0, tone: WARN },
    ],
  };
}

const FACTS: Record<ProviderOption["id"], (label: string) => ProviderFacts> = {
  candilog_local: () => ({
    description:
      "Les modèles tournent sur votre machine. Vos CV, vos lettres et vos candidatures ne sortent jamais de l'ordinateur. C'est gratuit et utilisable hors connexion ; en contrepartie, c'est plus lent et la qualité dépend de la taille du modèle installé.",
    meters: [
      { label: "Confidentialité", value: "Rien ne sort", level: 3, tone: GOOD },
      { label: "Coût", value: "Gratuit", level: 3, tone: GOOD },
      { label: "Hors connexion", value: "Oui", level: 3, tone: GOOD },
    ],
  }),
  ollama: () => ({
    description:
      "Votre propre installation d'Ollama, sur cet ordinateur ou sur votre réseau. Les données restent chez vous, sans service tiers.",
    meters: [
      { label: "Confidentialité", value: "Reste chez vous", level: 3, tone: GOOD },
      { label: "Coût", value: "Gratuit", level: 3, tone: GOOD },
      { label: "Hors connexion", value: "Selon l'installation", level: 2, tone: GOOD },
    ],
  }),
  claude: remoteFacts,
  openai: remoteFacts,
  gemini: remoteFacts,
  mistral: remoteFacts,
  deepseek: remoteFacts,
  custom: () => ({
    description:
      "Un service compatible OpenAI — LM Studio, un serveur interne… Les données vont à l'adresse que vous indiquez : locale ou distante, c'est elle qui décide.",
    meters: [
      { label: "Confidentialité", value: "Selon l'adresse", level: 2, tone: WARN },
      { label: "Coût", value: "Selon le service", level: 2, tone: WARN },
      { label: "Hors connexion", value: "Selon l'adresse", level: 1, tone: WARN },
    ],
  }),
};

function ControlLabel({ children }: { children: ReactNode }) {
  return <p className="mb-1.5 text-label font-mid text-ink-muted">{children}</p>;
}

function Temperature({ value, onChange }: { value: number; onChange: (value: number) => void }) {
  const part = Math.round((value / 2) * 100);

  return (
    <div className="max-w-[380px]">
      <div className="mb-1.5 flex items-baseline justify-between">
        <p className="text-label font-mid text-ink-muted">Température</p>
        <span className="rounded-chip bg-fill px-1.5 py-0.5 font-mono tabular text-meta text-ink">
          {value.toFixed(1)}
        </span>
      </div>
      <input
        type="range"
        min={0}
        max={2}
        step={0.1}
        value={value}
        aria-label="Température"
        onChange={(event) => onChange(Number(event.target.value))}
        style={{
          background: `linear-gradient(to right, var(--color-accent) 0 ${part}%, var(--color-fill) ${part}% 100%)`,
        }}
        className={cn(
          "h-1.5 w-full cursor-pointer appearance-none rounded-full",
          "focus-visible:outline-1 focus-visible:outline-accent-focus",
          "[&::-webkit-slider-thumb]:size-[14px] [&::-webkit-slider-thumb]:appearance-none",
          "[&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border-2",
          "[&::-webkit-slider-thumb]:border-surface [&::-webkit-slider-thumb]:bg-accent",
          "[&::-moz-range-thumb]:size-[14px] [&::-moz-range-thumb]:rounded-full",
          "[&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-surface",
          "[&::-moz-range-thumb]:bg-accent",
        )}
      />
      <div className="mt-1.5 flex justify-between text-meta text-ink-faint">
        <span>Précise</span>
        <span>Créative</span>
      </div>
    </div>
  );
}
