import { useState, type ReactNode } from "react";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { AppError } from "@/shared/types/app-error";
import type { AnalysisMode, LlmForm, Settings, ThemePref } from "@/shared/types/generated/settings";
import {
  Button,
  ErrorBanner,
  FormField,
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
import { SettingsBody, SettingsCard } from "../components/SettingsUi";
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

      {vm.error && !vm.data ? (
        <div className="px-[18px] pt-4">
          <ErrorBanner
            message={vm.error instanceof AppError ? vm.error.message : "Les réglages n'ont pas pu être chargés."}
            onRetry={vm.recharger}
          />
        </div>
      ) : vm.isLoading || !form || !llm || !fournisseur || !logo ? (
        <div
          className="max-w-[1000px] space-y-4 px-[18px] pt-4"
          role="status"
          aria-label="Chargement des réglages"
        >
          {/* Même gabarit que l'écran chargé : bandeau, fournisseurs, puis les deux
              colonnes de réglages — sinon la mise en page saute à l'arrivée des données. */}
          <Skeleton className="h-[82px] w-full rounded-card" />
          <Skeleton className="h-[136px] w-full rounded-card" />
          <Skeleton className="h-60 w-full rounded-card" />
        </div>
      ) : (
        <SettingsBody>
          {/* Colonne bornée : au-delà, un champ « Endpoint » s'étirait sur 600 px et la
              grille de fournisseurs devenait un alignement de logos perdus. */}
          <div className="flex min-w-0 max-w-[1000px] flex-col gap-4">
            <AiHero
              logo={logo}
              label={fournisseur.label}
              model={llm.model}
              etat={etatIa(llm, test)}
              testMessage={test === "error" ? testMessage : null}
              busy={test === "pending"}
              onTest={() => void runTest()}
            />

            <SettingsCard icon="hub" title="Fournisseur">
              <ProviderGrid value={llm.provider} onChange={choisirFournisseur} />
            </SettingsCard>

            <div className="grid gap-4 min-[900px]:grid-cols-2 min-[900px]:items-start">
              <SettingsCard icon="tune" title="Configuration">
                <div className="flex flex-col gap-3.5">
                  <div className="flex max-w-[380px] items-end gap-2">
                    <FormField label="Modèle" required className="flex-1">
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
                  <FormField
                    label="Endpoint"
                    required={idProvider(llm.provider) === "custom"}
                    className="max-w-[380px]"
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
                  ) : null}
                </div>
              </SettingsCard>

              <div className="flex min-w-0 flex-col gap-4">
                <SettingsCard icon="bolt" title="Génération">
                  <div className="flex flex-col gap-4">
                    <div>
                      <ControlLabel>Mode d'analyse</ControlLabel>
                      {/* Conteneur flex : sans lui, le groupe s'étirerait sur toute la
                          colonne et traînerait une piste vide à droite des options. */}
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
                </SettingsCard>

                <SettingsCard icon="palette" title="Apparence">
                  <div className="flex flex-wrap gap-x-8 gap-y-4">
                    <div>
                      <ControlLabel>Thème</ControlLabel>
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
                      <ControlLabel>Son de fin de traitement</ControlLabel>
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
                </SettingsCard>
              </div>
            </div>
          </div>
        </SettingsBody>
      )}
    </div>
  );
}

/** Libellé de contrôle, aligné sur celui de `FormField` : même écran, même graisse. */
function ControlLabel({ children }: { children: ReactNode }) {
  return <p className="mb-1.5 text-label font-mid text-ink-muted">{children}</p>;
}

/**
 * Température : piste remplie jusqu'à la valeur, valeur en mono, bornes nommées.
 *
 * Le curseur natif nu ne disait ni où l'on était sur l'échelle, ni ce que « 0 » ou « 2 »
 * signifiaient. La piste est peinte par un dégradé calculé, faute de pseudo-élément de
 * remplissage standard.
 */
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
    <section className="flex flex-wrap items-center gap-x-4 gap-y-3 rounded-card border border-line bg-surface px-[18px] py-4">
      <span className="flex size-12 flex-none items-center justify-center rounded-tile bg-fill">
        <img
          src={logo.src}
          alt=""
          width={28}
          height={28}
          className={cn("size-7", logo.mono && "dark:invert")}
        />
      </span>
      <div className="min-w-[200px] flex-1">
        {/* Pas d'intitulé « Fournisseur » ici : la section juste en dessous porte déjà ce
            libellé, et le répéter ajoutait un niveau de titre pour rien. */}
        <p className="text-title text-ink">{label}</p>
        <p className="mt-0.5 truncate font-mono tabular text-note text-ink-faint">
          {model || "Aucun modèle"}
        </p>
      </div>
      <div className="flex flex-none items-center gap-2.5">
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
