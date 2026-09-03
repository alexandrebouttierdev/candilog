import { useState } from "react";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import type { AnalysisMode, LlmForm, Settings, ThemePref } from "@/shared/types/generated/settings";
import {
  Button,
  ErrorBanner,
  FormField,
  InspectorSectionLabel,
  PageHeader,
  SegmentedControl,
  Select,
  Skeleton,
  StatusPill,
  TextInput,
} from "@/shared/ui";
import { settingsService } from "../../services/settingsService";
import { useSettingsViewModel } from "../../viewmodel/useSettingsViewModel";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import {
  completionSoundEnabled,
  setCompletionSoundEnabled,
} from "@/shared/lib/completion-sound";
import {
  endpointDefaut,
  idProvider,
  modelDefaut,
  versProvider,
  type FournisseurOption,
} from "../../model/providers";
import { etatIa, type EtatIa, type TestConnexion } from "../../model/etatIa";
import { ProviderGrid, defFournisseur, logoFournisseur } from "../components/ProviderGrid";
import { SettingsBody } from "../components/SettingsUi";
import { cn } from "@/shared/lib/cn";

const MODES: Array<{ value: AnalysisMode; label: string }> = [
  { value: "auto", label: "Auto" },
  { value: "small", label: "Petit" },
  { value: "standard", label: "Standard" },
  { value: "advanced", label: "Avancé" },
];

const THEMES: Array<{ value: ThemePref; label: string }> = [
  { value: "light", label: "Clair" },
  { value: "dark", label: "Sombre" },
  { value: "system", label: "Système" },
];

const SONS = [
  { value: "on", label: "Activé" },
  { value: "off", label: "Désactivé" },
] as const;

/** Intelligence artificielle : fournisseur, modèle, comportement et apparence. */
export function AiPage() {
  const vm = useSettingsViewModel();
  const setTheme = useUiStore((state) => state.setTheme);
  const [draft, setDraft] = useState<Settings | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [models, setModels] = useState<string[]>([]);
  const [test, setTest] = useState<TestConnexion>("idle");
  // Préférence locale, appliquée immédiatement : elle ne passe pas par le brouillon des
  // réglages puisqu'elle n'est pas enregistrée en base.
  const [son, setSon] = useState<"on" | "off">(() =>
    completionSoundEnabled() ? "on" : "off",
  );
  const [testMessage, setTestMessage] = useState<string | null>(null);
  const form = draft ?? vm.data ?? null;
  const llm = form?.llm;

  const patchLlm = (partiel: Partial<LlmForm>) => {
    setDraft((current) => {
      const base = current ?? vm.data;
      return base ? { ...base, llm: { ...base.llm, ...partiel } } : current;
    });
    setTest("idle");
  };

  const choisirFournisseur = (id: FournisseurOption["id"]) => {
    patchLlm({
      provider: versProvider(id),
      endpoint: endpointDefaut(id),
      model: modelDefaut(id),
    });
  };

  const save = async () => {
    if (!form) return;
    await vm.save(form, apiKeyDraft.trim() || null);
    setDraft(null);
    setApiKeyDraft("");
  };

  const runTest = async () => {
    if (!llm) return;
    setTest("pending");
    setTestMessage(null);
    try {
      await settingsService.testConnection(llm, apiKeyDraft.trim() || null);
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
      const list = await settingsService.listModels(llm, apiKeyDraft.trim() || null);
      setModels(list);
      if (list.length > 0 && !list.includes(llm.model)) {
        patchLlm({ model: list[0] ?? llm.model });
      }
    } catch (error) {
      setTest("error");
      setTestMessage(error instanceof AppError ? error.message : "Modèles inaccessibles.");
    }
  };

  const clearApiKey = async () => {
    await vm.clearApiKey();
    setApiKeyDraft("");
    setDraft((current) =>
      current
        ? {
            ...current,
            llm: { ...current.llm, api_key_configured: false },
          }
        : current,
    );
  };

  const fournisseur = llm ? defFournisseur(llm.provider) : null;
  const logo = fournisseur ? logoFournisseur(fournisseur.id) : null;

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>
          {fournisseur
            ? `${fournisseur.label}${llm?.model ? ` · ${llm.model}` : ""}`
            : "Candilog · données locales"}
        </ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="smart_toy"
        title="Intelligence artificielle"
        subtitle="Le moteur reste sous votre contrôle"
        primary={
          <Button variant="primary" icon="save" disabled={!form || vm.isSaving} onClick={() => void save()}>
            Enregistrer
          </Button>
        }
      />

      {vm.isLoading || !form || !llm || !fournisseur || !logo ? (
        <div className="space-y-4 px-[18px] pt-4" role="status" aria-label="Chargement des réglages">
          <Skeleton className="h-16 w-full" />
          <Skeleton className="h-20 w-full" />
          <Skeleton className="h-40 w-full" />
        </div>
      ) : vm.error && !vm.data ? (
        <div className="px-[18px] pt-4">
          <ErrorBanner
            message={vm.error instanceof AppError ? vm.error.message : "Les réglages n'ont pas pu être chargés."}
            onRetry={vm.recharger}
          />
        </div>
      ) : (
        <SettingsBody>
          <AiHero
            logo={logo}
            label={fournisseur.label}
            model={llm.model}
            etat={etatIa(llm, test)}
            testMessage={test === "error" ? testMessage : null}
            busy={test === "pending"}
            onTest={() => void runTest()}
          />

          <section className="min-w-0">
            <InspectorSectionLabel>Fournisseur</InspectorSectionLabel>
            <ProviderGrid value={llm.provider} onChange={choisirFournisseur} />
          </section>

          <div className="grid gap-x-10 gap-y-6 min-[980px]:grid-cols-2">
            <section className="min-w-0">
              <InspectorSectionLabel>Configuration</InspectorSectionLabel>
              <div className="flex flex-col gap-3.5">
                <div className="grid gap-2 min-[520px]:grid-cols-[minmax(0,1fr)_auto] min-[520px]:items-end">
                  <FormField label="Modèle" required>
                    {(props) =>
                      models.length > 0 ? (
                        <Select {...props} value={llm.model} onChange={(event) => patchLlm({ model: event.target.value })}>
                          {models.map((model) => (
                            <option key={model} value={model}>
                              {model}
                            </option>
                          ))}
                        </Select>
                      ) : (
                        <TextInput
                          {...props}
                          value={llm.model}
                          onChange={(event) => patchLlm({ model: event.target.value })}
                        />
                      )
                    }
                  </FormField>
                  <Button variant="secondary" icon="refresh" onClick={() => void actualiserModels()}>
                    Actualiser
                  </Button>
                </div>
                <FormField label="Endpoint" required={idProvider(llm.provider) === "custom"}>
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
                ) : null}
              </div>
            </section>

            <section className="min-w-0">
              <InspectorSectionLabel>Génération</InspectorSectionLabel>
              <div className="flex flex-col gap-4">
                <div>
                  <p className="mb-1.5 text-note text-ink-subtle">Mode d'analyse</p>
                  <SegmentedControl
                    label="Mode d'analyse"
                    value={llm.mode}
                    onChange={(mode) => patchLlm({ mode })}
                    options={MODES}
                  />
                </div>
                <div>
                  <div className="mb-1.5 flex items-baseline justify-between">
                    <p className="text-note text-ink-subtle">Température</p>
                    <p className="font-mono tabular text-note text-ink">{llm.temperature.toFixed(1)}</p>
                  </div>
                  <input
                    type="range"
                    min={0}
                    max={2}
                    step={0.1}
                    value={llm.temperature}
                    aria-label="Température"
                    onChange={(event) => patchLlm({ temperature: Number(event.target.value) })}
                    className="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-fill accent-accent"
                  />
                </div>

              </div>
            </section>

            <section className="min-w-0 min-[980px]:col-span-2">
              <InspectorSectionLabel>Apparence</InspectorSectionLabel>
              <div className="flex flex-wrap gap-x-10 gap-y-4">
                <div>
                  <p className="mb-1.5 text-note text-ink-subtle">Thème</p>
                  <SegmentedControl
                    label="Thème"
                    value={form.theme}
                    onChange={(theme) => {
                      setDraft({ ...form, theme });
                      setTheme(theme);
                      applyTheme(theme);
                    }}
                    options={THEMES}
                  />
                </div>
                <div>
                  <p className="mb-1.5 text-note text-ink-subtle">Son de fin de traitement</p>
                  <SegmentedControl
                    label="Son de fin de traitement"
                    value={son}
                    onChange={(valeur) => {
                      setSon(valeur);
                      setCompletionSoundEnabled(valeur === "on");
                    }}
                    options={SONS}
                  />
                </div>
              </div>
            </section>
          </div>
        </SettingsBody>
      )}
    </div>
  );
}

/**
 * En-tête de l'écran : fournisseur actif, modèle, état, action de test.
 *
 * Un seul bloc, sans carte ni bordure : l'information la plus utile — « est-ce que ça
 * marche ? » — doit se lire d'un coup d'œil, pas se déduire de trois panneaux imbriqués.
 */
function AiHero({
  logo,
  label,
  model,
  etat,
  testMessage,
  busy,
  onTest,
}: {
  logo: { src: string; mono: boolean };
  label: string;
  model: string;
  etat: EtatIa;
  /** Message du dernier échec, affiché sous l'état ; `null` sinon. */
  testMessage: string | null;
  busy: boolean;
  onTest: () => void;
}) {
  return (
    <section className="flex flex-wrap items-start gap-4 border-b border-line-soft pb-5">
      <span className="flex size-12 flex-none items-center justify-center rounded-control bg-fill">
        <img
          src={logo.src}
          alt=""
          width={28}
          height={28}
          className={cn("size-7", logo.mono && "dark:invert")}
        />
      </span>
      <div className="min-w-[220px] flex-1">
        {/* Pas d'intitulé « Fournisseur » ici : la section juste en dessous porte déjà ce
            libellé, et le répéter ajoutait un niveau de titre pour rien. */}
        <p className="text-title text-ink">{label}</p>
        <p className="mt-1 truncate font-mono tabular text-note text-ink-faint">
          {model || "Aucun modèle"}
        </p>
      </div>
      <div className="flex flex-none flex-col items-end gap-2 pt-0.5">
        <StatusPill tone={etat.tone}>{etat.label}</StatusPill>
        <Button icon="wifi" disabled={busy} onClick={onTest}>
          Tester la connexion
        </Button>
      </div>
      {testMessage ?? etat.hint ? (
        <p
          role="status"
          className={cn(
            "w-full text-note leading-relaxed",
            testMessage ? "text-danger" : "text-ink-faint",
          )}
        >
          {testMessage ?? etat.hint}
        </p>
      ) : null}
    </section>
  );
}
