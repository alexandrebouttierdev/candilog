import { useState } from "react";
import { AppError } from "@/shared/types/app-error";
import type { AnalysisMode, LlmFormulaire, Parametres, ThemePref } from "@/shared/types/generated/parametres";
import {
  Button,
  ConfirmDialog,
  ErrorBanner,
  FormField,
  PageHeader,
  Select,
  Skeleton,
  TextInput,
} from "@/shared/ui";
import { parametresService } from "../../services/parametres.service";
import { useParametresViewModel } from "../../viewmodel/useParametresViewModel";
import { applyTheme, useUiStore } from "@/shared/lib/ui-store";
import {
  endpointDefaut,
  identifiantProvider,
  modeleDefaut,
  versProvider,
  type FournisseurOption,
} from "../../model/providers";
import { ProviderGrid } from "../components/ProviderGrid";
import { SettingsBody, SettingsCard } from "../components/SettingsUi";

const MODES: Array<{ value: AnalysisMode; label: string }> = [
  { value: "auto", label: "Automatique" },
  { value: "small", label: "Petit modèle" },
  { value: "standard", label: "Standard" },
  { value: "advanced", label: "Avancé" },
];

const THEMES: Array<{ value: ThemePref; label: string }> = [
  { value: "light", label: "Clair" },
  { value: "dark", label: "Sombre" },
  { value: "system", label: "Système" },
];

/** Intelligence artificielle : fournisseur, modèle, comportement et apparence. */
export function IaPage() {
  const vm = useParametresViewModel();
  const setTheme = useUiStore((state) => state.setTheme);
  const [draft, setDraft] = useState<Parametres | null>(null);
  const [modeles, setModeles] = useState<string[]>([]);
  const [test, setTest] = useState<"idle" | "pending" | "ok" | "error">("idle");
  const [testMessage, setTestMessage] = useState<string | null>(null);
  const [cacheOpen, setCacheOpen] = useState(false);
  const [cacheBusy, setCacheBusy] = useState(false);
  const form = draft ?? vm.data ?? null;
  const llm = form?.llm;

  const patchLlm = (partiel: Partial<LlmFormulaire>) => {
    setDraft((courant) => {
      const base = courant ?? vm.data;
      return base ? { ...base, llm: { ...base.llm, ...partiel } } : courant;
    });
    setTest("idle");
  };

  const choisirFournisseur = (id: FournisseurOption["id"]) => {
    patchLlm({
      provider: versProvider(id),
      endpoint: endpointDefaut(id),
      model: modeleDefaut(id),
    });
  };

  const enregistrer = async () => {
    if (!form) return;
    await vm.enregistrer(form);
    setDraft(null);
  };

  const tester = async () => {
    if (!llm) return;
    setTest("pending");
    setTestMessage(null);
    try {
      await parametresService.testerConnexion(llm);
      setTest("ok");
      setTestMessage("Connexion établie.");
    } catch (error) {
      setTest("error");
      setTestMessage(error instanceof AppError ? error.message : "Connexion impossible.");
    }
  };

  const actualiserModeles = async () => {
    if (!llm) return;
    try {
      const liste = await parametresService.listerModeles(llm);
      setModeles(liste);
      if (liste.length > 0 && !liste.includes(llm.model)) {
        patchLlm({ model: liste[0] ?? llm.model });
      }
    } catch (error) {
      setTest("error");
      setTestMessage(error instanceof AppError ? error.message : "Modèles inaccessibles.");
    }
  };

  const viderCache = async () => {
    setCacheBusy(true);
    try {
      await parametresService.viderCacheIa();
      setCacheOpen(false);
    } finally {
      setCacheBusy(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="smart_toy"
        title="Intelligence artificielle"
        subtitle="Fournisseur, modèle et comportement"
        primary={
          <Button variant="primary" icon="save" disabled={!form || vm.isSaving} onClick={() => void enregistrer()}>
            Enregistrer
          </Button>
        }
      />

      {vm.isLoading || !form || !llm ? (
        <div className="space-y-3 p-6" role="status" aria-label="Chargement des réglages">
          <Skeleton className="h-24 w-full rounded-card" />
          <Skeleton className="h-48 w-full rounded-card" />
        </div>
      ) : vm.error && !vm.data ? (
        <div className="p-6">
          <ErrorBanner
            message={vm.error instanceof AppError ? vm.error.message : "Les réglages n’ont pas pu être chargés."}
            onRetry={vm.recharger}
          />
        </div>
      ) : (
        <SettingsBody>
          <SettingsCard icon="hub" title="Fournisseur">
            <ProviderGrid value={llm.provider} onChange={choisirFournisseur} />
            <div className="mt-5 space-y-4 rounded-card bg-surface-alt p-4">
              <div className="grid gap-4 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
                <FormField label="Modèle" required>
                  {(props) =>
                    modeles.length > 0 ? (
                      <Select {...props} value={llm.model} onChange={(event) => patchLlm({ model: event.target.value })}>
                        {modeles.map((modele) => (
                          <option key={modele} value={modele}>
                            {modele}
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
                <Button variant="secondary" icon="refresh" onClick={() => void actualiserModeles()}>
                  Actualiser
                </Button>
              </div>
              <FormField label="Endpoint" required={identifiantProvider(llm.provider) === "custom"}>
                {(props) => (
                  <TextInput
                    {...props}
                    value={llm.endpoint ?? ""}
                    onChange={(event) => patchLlm({ endpoint: event.target.value || null })}
                  />
                )}
              </FormField>
              {identifiantProvider(llm.provider) !== "ollama" ? (
                <FormField
                  label="Clé API"
                  required={identifiantProvider(llm.provider) !== "custom"}
                  help="Stockée dans le coffre système, jamais en clair dans la base."
                >
                  {(props) => (
                    <TextInput
                      {...props}
                      type="password"
                      autoComplete="off"
                      value={llm.apiKey ?? ""}
                      onChange={(event) => patchLlm({ apiKey: event.target.value || null })}
                    />
                  )}
                </FormField>
              ) : null}
              <div className="grid gap-4 sm:grid-cols-2">
                <FormField label="Mode d'analyse">
                  {(props) => (
                    <Select
                      {...props}
                      value={llm.mode}
                      onChange={(event) => patchLlm({ mode: event.target.value as AnalysisMode })}
                    >
                      {MODES.map((mode) => (
                        <option key={mode.value} value={mode.value}>
                          {mode.label}
                        </option>
                      ))}
                    </Select>
                  )}
                </FormField>
                <FormField label={`Température (${llm.temperature.toFixed(1)})`}>
                  {(props) => (
                    <input
                      {...props}
                      type="range"
                      min={0}
                      max={2}
                      step={0.1}
                      value={llm.temperature}
                      onChange={(event) => patchLlm({ temperature: Number(event.target.value) })}
                      className="h-11 w-full accent-[var(--accent)]"
                    />
                  )}
                </FormField>
              </div>
            </div>
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <Button variant="ghost" icon="wifi" disabled={test === "pending"} onClick={() => void tester()}>
                Tester la connexion
              </Button>
              <Button variant="ghost" icon="delete" onClick={() => setCacheOpen(true)}>
                Vider le cache IA
              </Button>
              {testMessage ? (
                <p className={test === "ok" ? "text-meta text-success" : "text-meta text-danger"} role="status">
                  {testMessage}
                </p>
              ) : null}
            </div>
          </SettingsCard>

          <SettingsCard icon="contrast" title="Apparence">
            <div role="radiogroup" aria-label="Thème" className="grid grid-cols-3 gap-2">
              {THEMES.map((theme) => {
                const selected = form.theme === theme.value;
                return (
                  <button
                    key={theme.value}
                    type="button"
                    role="radio"
                    aria-checked={selected}
                    onClick={() => {
                      setDraft({ ...form, theme: theme.value });
                      setTheme(theme.value);
                      applyTheme(theme.value);
                    }}
                    className={[
                      "min-h-11 rounded-card border px-3 py-2 text-label font-medium",
                      selected ? "border-accent bg-accent-tint text-ink" : "border-line bg-surface text-ink-muted hover:bg-neutral-tint",
                    ].join(" ")}
                  >
                    {theme.label}
                  </button>
                );
              })}
            </div>
          </SettingsCard>
        </SettingsBody>
      )}

      <ConfirmDialog
        open={cacheOpen}
        title="Vider le cache IA ?"
        description="Les réponses déjà calculées seront oubliées. Les prochaines analyses rappelleront le fournisseur."
        note="Vos candidatures, CV et réglages restent intacts."
        confirmLabel="Vider le cache"
        busy={cacheBusy}
        onCancel={() => setCacheOpen(false)}
        onConfirm={() => void viderCache()}
      />
    </div>
  );
}
