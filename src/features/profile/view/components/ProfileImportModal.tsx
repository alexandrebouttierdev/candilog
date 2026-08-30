import { useId, useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import type {
  ImportProfilePreview,
  ImportProfileRequest,
  ImportProfileResult,
} from "@/shared/types/generated/profile";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import { useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { AppError } from "@/shared/types/app-error";
import {
  Button,
  ConfirmDialog,
  ErrorBanner,
  Icon,
  ModalHost,
} from "@/shared/ui";
import { formatDuration } from "../../model/formatElapsed";
import {
  countMarked,
  explainImportErrors,
  importProfileRequestSchema,
  previewToFormValues,
  summarizeImport,
  type ImportProfileFormInput,
  type ImportProfileFormValues,
} from "../../model/import-review.schema";
import { useElapsedClock } from "../../viewmodel/useElapsedClock";
import {
  useProfileImportProgress,
  type ImportJournalEntry,
} from "../../viewmodel/useProfileImportProgress";
import { ImportAnalysisPanel } from "./ImportAnalysisPanel";
import { ImportDonePanel } from "./ImportDonePanel";
import { ImportJournal } from "./ImportJournal";
import { ImportReviewForm } from "./ImportReviewForm";

type Phase = "pick" | "picking" | "analyze" | "review" | "error" | "done";

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
  const [operation, setOperation] = useState<string | null>(null);
  const [phase, setPhase] = useState<Phase>("pick");
  const [preview, setPreview] = useState<ImportProfilePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [requestedAt, setRequestedAt] = useState<number | null>(null);
  const [finishedAt, setFinishedAt] = useState<number | null>(null);
  const [result, setResult] = useState<ImportProfileResult | null>(null);
  const [totalMs, setTotalMs] = useState(0);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pendingRequest, setPendingRequest] =
    useState<ImportProfileRequest | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const progress = useProfileImportProgress(operation);
  useCancelAiOnUnmount(operation);

  // Le sélecteur de fichier natif est ouvert par la commande Rust : le clic ne déclenche
  // rien d'autre. L'analyse ne commence qu'au premier événement de progression, émis une
  // fois le CV choisi, et c'est cet instant qui date la progression comme la durée.
  const analysisStartedAt = firstEventAt(progress.entries);
  const view: Phase =
    phase === "picking" && analysisStartedAt !== null ? "analyze" : phase;
  const startedAt = analysisStartedAt ?? requestedAt;
  const elapsedMs = useElapsedClock(view === "analyze", startedAt);
  const analysisMs =
    finishedAt !== null && startedAt !== null ? finishedAt - startedAt : 0;

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

  const analyze = async () => {
    const id = generation_id();
    setOperation(id);
    setRequestedAt(Date.now());
    setFinishedAt(null);
    setPhase("picking");
    setError(null);
    try {
      const next = await aiService.importProfile({ generation_id: id });
      if (next === null) {
        setPhase("pick");
        return;
      }
      setPreview(next);
      form.reset(previewToFormValues(next));
      setFinishedAt(Date.now());
      setPhase("review");
    } catch (caught) {
      if (caught instanceof AppError && caught.code === "CANCELLED") {
        setPhase("pick");
        return;
      }
      setError(
        caught instanceof AppError
          ? caught.message
          : "L'analyse du CV n'a pas pu être terminée.",
      );
      setFinishedAt(Date.now());
      setPhase("error");
    } finally {
      setOperation(null);
    }
  };

  const close = () => {
    if (operation) void aiService.cancel(operation);
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
      ? `Analyse terminée en ${formatDuration(analysisMs)}`
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
          <PickFile waiting={view === "picking"} onChoose={() => void analyze()} />
        ) : null}
        {view === "analyze" ? (
          <ImportAnalysisPanel
            step={progress.step}
            elapsedMs={elapsedMs}
            entries={progress.entries}
          />
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
          <ImportReviewForm
            preview={preview}
            entries={progress.entries}
            formId={formId}
            form={form}
            formError={formError}
            onSubmit={(values) => void apply(values)}
          />
        ) : null}
        {view === "done" && result ? (
          <ImportDonePanel result={result} totalMs={totalMs} />
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

function PickFile({
  waiting,
  onChoose,
}: {
  waiting: boolean;
  onChoose: () => void;
}) {
  return (
    <div className="pt-3">
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
    </div>
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
    counts: {
      identity: 0,
      experiences: 0,
      skills: 0,
      education: 0,
      languages: 0,
      projects: 0,
      certifications: 0,
    },
  };
}
