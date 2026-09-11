import { useState, type KeyboardEvent, type ReactNode } from "react";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AiBenchmarkModal, useAiRailStatusStore } from "@/features/ai";
import { AppError } from "@/shared/types/app-error";
import type { AnalysisMode, LlmForm, Settings } from "@/shared/types/generated/settings";
import {
  Button,
  ErrorBanner,
  FormField,
  Icon,
  PageHeader,
  SegmentedControl,
  Skeleton,
  Tag,
  TextInput,
} from "@/shared/ui";
import { openExternal } from "@/shared/services/external-link";
import { useSettingsViewModel } from "../../viewmodel/useSettingsViewModel";
import {
  getProvider,
  OTHER_PROVIDERS,
  idProvider,
  llmFromPreset,
  presetFromLlm,
  type ProviderOption,
} from "../../model/providers";
import { ProviderGrid, providerLogo } from "../components/ProviderGrid";
import { AiHero } from "../components/AiHero";
import { ManagedOllamaPanel } from "../components/ManagedOllamaPanel";
import { RemoteModelPicker } from "../components/RemoteModelPicker";
import { SettingsBody, SettingsCard } from "../components/SettingsUi";
import { cn } from "@/shared/lib/cn";
import { aiStatus, type ConnectionTest } from "../../model/aiStatus";
import { managedOllamaStatus } from "../../model/managedOllamaStatus";
import { useManagedOllamaViewModel } from "../../viewmodel/useManagedOllamaViewModel";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";

const MODES: Array<{ value: AnalysisMode; label: string }> = [
  { value: "auto", label: "Auto" },
  { value: "small", label: "Petit" },
  { value: "standard", label: "Standard" },
  { value: "advanced", label: "Avancé" },
];

type AiTab = "local" | "providers";

const TABS: Array<{ id: AiTab; label: string; badge?: string }> = [
  { id: "local", label: "IA locale", badge: "Gratuit" },
  { id: "providers", label: "IA online/personnalisé" },
];

/** Intelligence artificielle : fournisseur, modèle, comportement et apparence. */
export function AiPage() {
  const vm = useSettingsViewModel();
  const [tab, setTab] = useState<AiTab>("local");
  const [draft, setDraft] = useState<Settings | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  /** Brouillons de clé par fournisseur : basculer ne doit pas écraser la saisie en cours. */
  const [apiKeyByProvider, setApiKeyByProvider] = useState<Record<string, string>>({});
  const [models, setModels] = useState<string[]>([]);
  const [test, setTestState] = useState<ConnectionTest>("idle");
  const [testMessage, setTestMessage] = useState<string | null>(null);
  const [benchmarkOpen, setBenchmarkOpen] = useState(false);
  const [benchmarkModelLabel, setBenchmarkModelLabel] = useState("");
  const form = draft ?? vm.data ?? null;
  const llm = form?.llm;
  const providerId = llm ? idProvider(llm.provider) : null;
  const isCandilogLocal = providerId === "candilog_local";
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
  const logo = fournisseur ? providerLogo(fournisseur.id) : null;
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
  const noteModele = isCandilogLocal ? managedModelLabel : (llm?.model ?? "");

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>
          {fournisseur
            ? `${fournisseur.label}${noteModele ? ` · ${noteModele}` : ""}`
            : "Candilog · données locales"}
        </ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="smart_toy"
        title="Intelligence artificielle"
        subtitle="Le moteur reste sous votre contrôle"
        primary={
          tab === "providers" ? (
            <Button
              variant="primary"
              icon={vm.isSaving ? "progress_activity" : "save"}
              disabled={!form || vm.isSaving}
              onClick={() => void save()}
            >
              {vm.isSaving ? "Enregistrement…" : "Enregistrer"}
            </Button>
          ) : null
        }
      />

      {vm.error && !vm.data ? (
        <div className="px-[18px] pt-4">
          <ErrorBanner
            message={vm.error instanceof AppError ? vm.error.message : "Les réglages n'ont pas pu être chargés."}
            onRetry={vm.reload}
          />
        </div>
      ) : vm.isLoading || !form || !llm || !fournisseur ? (
        <div
          className="max-w-[1000px] space-y-4 px-[18px] pt-4"
          role="status"
          aria-label="Chargement des réglages"
        >
          <Skeleton className="h-[82px] w-full rounded-card" />
          <Skeleton className="h-[136px] w-full rounded-card" />
          <Skeleton className="h-60 w-full rounded-card" />
        </div>
      ) : (
        <SettingsBody>
          <div className="flex min-w-0 max-w-[1000px] flex-col gap-4">
            {isCandilogLocal ? (
              <AiHero
                logo={logo}
                label={fournisseur.label}
                model={managedModelLabel}
                status={managedOllamaStatus(managedVm.status, managedVm.error)}
                testMessage={null}
                testLabel="Tester l'IA"
                busy={managedVm.isInstalling}
                testDisabled={!managedActive}
                onTest={() => ouvrirBenchmark(managedModelLabel || "IA locale")}
              />
            ) : (
              <AiHero
                logo={logo}
                label={fournisseur.label}
                model={llm.model}
                status={aiStatus(llm, test)}
                testMessage={test === "error" ? testMessage : null}
                busy={test === "pending"}
                onTest={() => void runTest()}
              />
            )}

            <AiTabs active={tab} onChange={setTab} />

            {tab === "local" ? (
              <ManagedOllamaPanel vm={managedVm} onTestModel={(model) => void testerModeleLocal(model)} />
            ) : null}

            {tab === "providers" ? (
              <div className="flex flex-col gap-4">
                <SettingsCard icon="hub" title="Fournisseur">
                  <ProviderGrid
                    value={llm.provider}
                    onChange={selectProvider}
                    items={OTHER_PROVIDERS}
                  />
                </SettingsCard>

                <div className="grid gap-4 min-[900px]:grid-cols-2 min-[900px]:items-start">
                    <SettingsCard icon="tune" title="Configuration">
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
                                    models.length > 0
                                      ? "Choisissez ci-dessous ou saisissez un identifiant"
                                      : "Identifiant du modèle"
                                  }
                                />
                              )}
                            </FormField>
                            <Button
                              variant="secondary"
                              icon="refresh"
                              onClick={() => void actualiserModels()}
                            >
                              Actualiser
                            </Button>
                          </div>
                          {models.length > 0 ? (
                            <RemoteModelPicker
                              models={models}
                              value={llm.model}
                              onChange={(model) => patchLlm({ model })}
                              providerLabel={fournisseur.label}
                            />
                          ) : (
                            <p className="text-meta leading-relaxed text-ink-faint">
                              Actualisez la liste pour afficher les modèles proposés par le
                              fournisseur.
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
                          <div className="max-w-[380px] rounded-field border border-line bg-fill px-3 py-2.5">
                            <p className="flex items-center gap-1.5 text-label font-mid text-ink">
                              <Icon name="info" size={14} className="flex-none text-ink-faint" />
                              Modèle local : aucune clé, aucune connexion
                            </p>
                            <p className="mt-1 text-meta leading-relaxed text-ink-muted">
                              Besoin d'aide pour choisir un modèle compatible avec votre machine ?{" "}
                              <button
                                type="button"
                                onClick={() => void openExternal("https://www.canirun.ai/")}
                                className="text-accent underline-offset-2 hover:underline"
                              >
                                canirun.ai
                              </button>
                            </p>
                          </div>
                        )}
                      </div>
                    </SettingsCard>

                    <SettingsCard icon="bolt" title="Génération">
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
                        <Temperature
                          value={llm.temperature}
                          onChange={(temperature) => patchLlm({ temperature })}
                        />
                      </div>
                    </SettingsCard>
                  </div>
              </div>
            ) : null}

          </div>
        </SettingsBody>
      )}
      <AiBenchmarkModal
        open={benchmarkOpen}
        onClose={() => setBenchmarkOpen(false)}
        modelLabel={benchmarkModelLabel || managedModelLabel || "IA locale"}
      />
    </div>
  );
}

function AiTabs({ active, onChange }: { active: AiTab; onChange: (tab: AiTab) => void }) {
  const naviguer = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    let prochain = index;
    if (event.key === "ArrowRight") prochain = (index + 1) % TABS.length;
    else if (event.key === "ArrowLeft") prochain = (index - 1 + TABS.length) % TABS.length;
    else if (event.key === "Home") prochain = 0;
    else if (event.key === "End") prochain = TABS.length - 1;
    else return;
    event.preventDefault();
    const suivant = TABS[prochain];
    if (!suivant) return;
    onChange(suivant.id);
    document.getElementById(`ai-tab-${suivant.id}`)?.focus();
  };

  return (
    <div role="tablist" aria-label="Sections Intelligence artificielle" className="flex gap-[3px]">
      {TABS.map((item, index) => {
        const selected = active === item.id;
        return (
          <button
            key={item.id}
            id={`ai-tab-${item.id}`}
            type="button"
            role="tab"
            aria-selected={selected}
            aria-controls={`ai-panel-${item.id}`}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(item.id)}
            onKeyDown={(event) => naviguer(event, index)}
            className={cn(
              "flex h-tab flex-none items-center rounded-button px-3 text-body font-medium",
              selected ? "bg-accent-tint text-accent" : "text-ink-muted hover:bg-neutral-tint",
            )}
          >
            <span className="flex items-center gap-1.5">
              {item.label}
              {item.badge ? <Tag className="py-0">{item.badge}</Tag> : null}
            </span>
          </button>
        );
      })}
    </div>
  );
}

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
