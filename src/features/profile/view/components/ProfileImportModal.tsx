import { useEffect, useId, useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import type {
  ImportProfilePreview,
  ImportProfileRequest,
  ImportProfileResult,
} from "@/shared/types/generated/profile";
import {
  type AiExecution,
  type CvAnalysisMethod,
  fetchActiveModelCapabilities,
  importProfileFromResume,
  isAiNotConfiguredError,
  useAiOperation,
  useAiRailStatusStore,
} from "@/features/ai";
import { AppError } from "@/shared/types/app-error";
import {
  Button,
  ConfirmDialog,
  ErrorBanner,
  Icon,
  ModalHost,
} from "@/shared/ui";
import { formatAiSummary } from "@/shared/lib/duration";
import {
  countMarked,
  explainImportErrors,
  importProfileRequestSchema,
  previewToFormValues,
  summarizeImport,
  type ImportProfileFormInput,
  type ImportProfileFormValues,
} from "../../model/import-review.schema";
import { useElapsedClock } from "@/shared/hooks/useElapsedClock";
import {
  useProfileImportProgress,
  type ImportJournalEntry,
} from "../../viewmodel/useProfileImportProgress";
import { ImportAnalysisPanel } from "./ImportAnalysisPanel";
import { ImportDonePanel } from "./ImportDonePanel";
import { ImportJournal } from "./ImportJournal";
import { ImportReviewForm } from "./ImportReviewForm";

type Phase = "pick" | "picking" | "analyze" | "review" | "error" | "done";

const METHOD_STORAGE_KEY = "candilog.cv-analysis-method";

/** Import d'un CV : analyse sans écriture, puis revue obligatoire. */
export function ProfileImportModal({
  open,
  busy,
  onClose,
  onApply,
}: {
  open: boolean;
  busy: boolean;
  onClose: () => void;
  onApply: (request: ImportProfileRequest) => Promise<ImportProfileResult>;
}) {
  const formId = useId();
  const { operation, stopping, start, stop, finish, isCurrent } = useAiOperation();
  const [phase, setPhase] = useState<Phase>("pick");
  const [preview, setPreview] = useState<ImportProfilePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [requestedAt, setRequestedAt] = useState<number | null>(null);
  const [metrics, setMetrics] = useState<
    Pick<AiExecution<unknown>, "elapsed_ms" | "tokens_used"> | null
  >(null);
  const [result, setResult] = useState<ImportProfileResult | null>(null);
  const [totalMs, setTotalMs] = useState(0);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pendingRequest, setPendingRequest] =
    useState<ImportProfileRequest | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [method, setMethod] = useState<CvAnalysisMethod>(readStoredMethod);
  const [visionAvailable, setVisionAvailable] = useState(true);
  const [fallbackUsed, setFallbackUsed] = useState(false);
  const [methodUsed, setMethodUsed] = useState<CvAnalysisMethod | null>(null);
  const progress = useProfileImportProgress(stopping ? null : (operation?.id ?? null));

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    void fetchActiveModelCapabilities()
      .then((caps) => {
        if (cancelled) return;
        setVisionAvailable(caps.vision);
        if (!caps.vision) {
          setMethod("text");
        } else if (readStoredMethod() === "vision") {
          setMethod("vision");
        }
      })
      .catch(() => {
        if (cancelled) return;
        // Sans métadonnées, on laisse le choix utilisateur ; le backend basculera si besoin.
        setVisionAvailable(true);
      });
    return () => {
      cancelled = true;
    };
  }, [open]);

  // Le sélecteur de fichier natif est ouvert par la commande Rust : le clic ne déclenche
  // rien d'autre. L'analyse ne commence qu'au premier événement de progression, émis une
  // fois le CV choisi, et c'est cet instant qui date la progression comme la durée.
  const analysisStartedAt = firstEventAt(progress.entries);
  const view: Phase =
    phase === "picking" && analysisStartedAt !== null ? "analyze" : phase;
  const startedAt = analysisStartedAt ?? requestedAt;
  const elapsedMs = useElapsedClock(view === "analyze" && !stopping, startedAt);

  const form = useForm<
    ImportProfileFormInput,
    unknown,
    ImportProfileFormValues
  >({
    resolver: zodResolver(importProfileRequestSchema),
    defaultValues: previewToFormValues(emptyPreview()),
  });
  useWatch({ control: form.control });
  const marked = countMarked(form.getValues());

  const chooseMethod = (next: CvAnalysisMethod) => {
    if (next === "vision" && !visionAvailable) return;
    setMethod(next);
    writeStoredMethod(next);
  };

  const analyze = async () => {
    let id: string;
    try {
      id = start("import");
    } catch (caught) {
      if (isAiNotConfiguredError(caught)) return;
      setError(
        caught instanceof Error
          ? caught.message
          : "L'analyse du CV n'a pas pu démarrer.",
      );
      setPhase("error");
      return;
    }
    setRequestedAt(Date.now());
    setPhase("picking");
    setError(null);
    setFallbackUsed(false);
    setMethodUsed(null);
    try {
      const next = await importProfileFromResume(id, method);
      if (!isCurrent(id)) return;
      if (next === null) {
        setPhase("pick");
        return;
      }
      setPreview(next.output.preview);
      form.reset(previewToFormValues(next.output.preview));
      setMetrics({
        elapsed_ms: next.elapsed_ms,
        tokens_used: next.tokens_used,
      });
      setFallbackUsed(next.output.fallback_used);
      setMethodUsed(next.output.method_used);
      setPhase("review");
    } catch (caught) {
      if (!isCurrent(id)) return;
      if (caught instanceof AppError && caught.code === "CANCELLED") {
        setPhase("pick");
        return;
      }
      setError(
        caught instanceof AppError
          ? caught.message
          : "L'analyse du CV n'a pas pu être terminée.",
      );
      useAiRailStatusStore.getState().setLastOperationFailed(true);
      setPhase("error");
    } finally {
      finish(id);
    }
  };

  const stopAnalysis = async () => {
    setError(null);
    try {
      await stop();
      setRequestedAt(null);
      setPhase("pick");
    } catch (caught) {
      setError(
        caught instanceof AppError
          ? caught.message
          : "L'analyse du CV n'a pas pu être arrêtée.",
      );
    }
  };

  const close = () => {
    if (operation) void stopAnalysis();
    onClose();
  };

  const apply = async (values: ImportProfileFormValues) => {
    setFormError(null);
    const summary = summarizeImport(values, preview ?? undefined);
    if (summary.replaced > 0) {
      setPendingRequest(values);
      setConfirmOpen(true);
      return;
    }
    await commit(values);
  };

  const refuse = () => {
    setFormError(
      explainImportErrors(form.getValues()) ||
        "Certains champs sont incomplets. Corrigez-les dans l'aperçu avant d'importer.",
    );
    document.querySelector<HTMLElement>("[aria-invalid='true']")?.scrollIntoView?.({
      block: "nearest",
    });
  };

  const commit = async (request: ImportProfileRequest) => {
    try {
      const start = startedAt ?? Date.now();
      const applied = await onApply(request);
      setResult(applied);
      setTotalMs(Date.now() - start);
      setConfirmOpen(false);
      setPendingRequest(null);
      setPhase("done");
    } catch (caught) {
      setConfirmOpen(false);
      setFormError(
        caught instanceof AppError
          ? caught.message
          : "L'import n'a pas pu être enregistré.",
      );
    }
  };

  const subtitle =
    view === "review"
      ? metrics
        ? formatAiSummary("Analysé", metrics.elapsed_ms, metrics.tokens_used)
        : "Analyse terminée"
      : view === "done"
        ? "Les éléments choisis ont été enregistrés"
        : view === "picking"
          ? "Choisissez le CV à analyser dans la fenêtre de votre système"
          : "Rien n'est enregistré avant votre validation";

  return (
    <>
      <ModalHost
        open={open}
        icon="upload_file"
        title="Importer depuis un CV"
        subtitle={subtitle}
        cancelLabel={view === "done" ? "Fermer" : "Annuler"}
        submitLabel="Importer les éléments sélectionnés"
        submitIcon="playlist_add_check"
        submitDisabled={view !== "review" || marked === 0}
        busy={busy}
        onClose={close}
        {...(view === "review"
          ? { onSubmit: () => void form.handleSubmit(apply, refuse)(), flush: true }
          : {})}
        width={view === "review" ? "880px" : "720px"}
      >
        {view === "pick" || view === "picking" ? (
          <div className="space-y-4 pt-3">
            <AnalysisMethodPicker
              method={method}
              visionAvailable={visionAvailable}
              disabled={view === "picking"}
              onChange={chooseMethod}
            />
            <PickFile waiting={view === "picking"} onChoose={() => void analyze()} />
          </div>
        ) : null}
        {view === "analyze" ? (
          <div className="space-y-4">
            {error ? <ErrorBanner title="Arrêt impossible" message={error} /> : null}
            <ImportAnalysisPanel
              step={progress.step}
              elapsedMs={elapsedMs}
              entries={progress.entries}
              tokens_used={progress.tokens_used}
              tokens_per_second={progress.tokens_per_second}
              stopping={stopping}
              onStop={() => void stopAnalysis()}
            />
          </div>
        ) : null}
        {view === "error" ? (
          <div className="space-y-4 pt-3">
            <ErrorBanner
              title="Import impossible"
              message={error ?? "L'analyse du CV n'a pas pu être terminée."}
            />
            <div className="flex gap-2">
              <Button variant="primary" onClick={() => void analyze()}>
                Réessayer
              </Button>
              <Button variant="secondary" onClick={close}>
                Fermer
              </Button>
            </div>
            <ImportJournal entries={progress.entries} defaultOpen />
          </div>
        ) : null}
        {view === "review" && preview ? (
          // Chaîne flex intacte : un wrapper `space-y-*` cassait le `min-h-0` / `flex-1`
          // requis par le split, et le corps `flush` (overflow masqué) ne défilait plus.
          <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
            {fallbackUsed || methodUsed ? (
              <div className="flex-none space-y-2 px-[18px] pt-3">
                {fallbackUsed ? (
                  <p className="rounded-button border border-line bg-surface-alt px-3 py-2 text-note text-ink">
                    L'analyse visuelle a échoué. Candilog a poursuivi automatiquement avec
                    l'analyse du texte.
                  </p>
                ) : null}
                {methodUsed ? (
                  <p className="text-meta text-ink-faint">
                    Méthode utilisée : {methodUsed === "vision" ? "Vision" : "Texte"}
                  </p>
                ) : null}
              </div>
            ) : null}
            <ImportReviewForm
              preview={preview}
              entries={progress.entries}
              formId={formId}
              form={form}
              formError={formError}
              onSubmit={(values) => void apply(values)}
            />
          </div>
        ) : null}
        {view === "done" && result ? (
          <ImportDonePanel result={result} totalMs={totalMs} aiMetrics={metrics} />
        ) : null}
      </ModalHost>
      <ConfirmDialog
        open={confirmOpen}
        title="Confirmer l'import"
        description={confirmDescription(pendingRequest, preview)}
        note="Les données non sélectionnées restent inchangées."
        confirmLabel="Confirmer l'import"
        busy={busy}
        onCancel={() => {
          setConfirmOpen(false);
          setPendingRequest(null);
        }}
        onConfirm={() => {
          if (pendingRequest) void commit(pendingRequest);
        }}
      />
    </>
  );
}

function AnalysisMethodPicker({
  method,
  visionAvailable,
  disabled,
  onChange,
}: {
  method: CvAnalysisMethod;
  visionAvailable: boolean;
  disabled: boolean;
  onChange: (method: CvAnalysisMethod) => void;
}) {
  return (
    <fieldset className="space-y-2" disabled={disabled}>
      <legend className="text-label font-semibold text-ink">Méthode d'analyse</legend>
      <label className="flex cursor-pointer gap-3 rounded-button border border-line px-3 py-2.5 has-[:disabled]:cursor-default">
        <input
          type="radio"
          name="cv-analysis-method"
          className="mt-1"
          checked={method === "vision"}
          disabled={!visionAvailable}
          onChange={() => onChange("vision")}
        />
        <span className="min-w-0">
          <span className="block text-body font-medium text-ink">Vision — recommandé</span>
          <span className="block text-meta text-ink-muted">
            Analyse directement la mise en page du CV avec le modèle multimodal.
          </span>
          {!visionAvailable ? (
            <span className="mt-1 block text-meta text-ink-faint">
              Ce modèle ne prend pas en charge l'analyse visuelle. Le mode Texte sera utilisé.
            </span>
          ) : null}
        </span>
      </label>
      <label className="flex cursor-pointer gap-3 rounded-button border border-line px-3 py-2.5">
        <input
          type="radio"
          name="cv-analysis-method"
          className="mt-1"
          checked={method === "text"}
          onChange={() => onChange("text")}
        />
        <span className="min-w-0">
          <span className="block text-body font-medium text-ink">Texte</span>
          <span className="block text-meta text-ink-muted">
            Extrait d'abord le contenu du PDF puis l'analyse avec le modèle.
          </span>
        </span>
      </label>
    </fieldset>
  );
}

function PickFile({
  waiting,
  onChoose,
}: {
  waiting: boolean;
  onChoose: () => void;
}) {
  return (
    <button
      type="button"
      disabled={waiting}
      onClick={onChoose}
      className="flex w-full flex-col items-center gap-2 rounded-card border border-dashed border-line px-6 py-6 text-center disabled:cursor-default"
    >
      <Icon
        name="upload_file"
        size={22}
        className="text-ink-faint"
      />
      <span className="text-body font-medium text-ink">
        {waiting ? "Sélection du fichier…" : "Choisir et analyser un CV PDF"}
      </span>
      <span className="text-meta text-ink-muted">
        {waiting
          ? "La fenêtre de sélection de votre système est ouverte : l'analyse démarrera une fois le CV choisi."
          : "Lecture locale · 10 Mo maximum"}
      </span>
    </button>
  );
}

/** Instant du premier événement d'analyse, seul repère fiable du début du traitement. */
function firstEventAt(entries: ImportJournalEntry[]): number | null {
  const first = entries[0];
  if (!first) return null;
  const at = Date.parse(first.at);
  return Number.isNaN(at) ? null : at;
}

function confirmDescription(
  request: ImportProfileRequest | null,
  preview: ImportProfilePreview | null,
) {
  if (!request) return "Des données existantes seront remplacées.";
  const summary = summarizeImport(request, preview ?? undefined);
  return `${summary.added} élément${summary.added > 1 ? "s" : ""} seront ajoutés. ${summary.replaced} élément${summary.replaced > 1 ? "s" : ""} existants seront remplacés. ${summary.skipped} élément${summary.skipped > 1 ? "s" : ""} seront ignorés.`;
}

function emptyPreview(): ImportProfilePreview {
  return {
    identity: [],
    experiences: [],
    skills: [],
    education: [],
    languages: [],
    projects: [],
    certifications: [],
    interests: [],
    counts: {
      identity: 0,
      experiences: 0,
      skills: 0,
      education: 0,
      languages: 0,
      projects: 0,
      certifications: 0,
      interests: 0,
    },
  };
}

function readStoredMethod(): CvAnalysisMethod {
  try {
    const raw = localStorage.getItem(METHOD_STORAGE_KEY);
    if (raw === "text" || raw === "vision") return raw;
  } catch {
    // localStorage indisponible (tests) : Vision par défaut.
  }
  return "vision";
}

function writeStoredMethod(method: CvAnalysisMethod) {
  try {
    localStorage.setItem(METHOD_STORAGE_KEY, method);
  } catch {
    // Ignoré : la préférence reste en mémoire pour la session.
  }
}
