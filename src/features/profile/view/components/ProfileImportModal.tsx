import { useId, useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
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
import { useProfileImportProgress } from "../../viewmodel/useProfileImportProgress";
import { ImportAnalysisPanel } from "./ImportAnalysisPanel";
import { ImportDonePanel } from "./ImportDonePanel";
import { ImportJournal } from "./ImportJournal";
import { ImportReviewForm } from "./ImportReviewForm";

type Phase = "pick" | "analyze" | "review" | "error" | "done";

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
  const [path, setPath] = useState<string | null>(null);
  const [operation, setOperation] = useState<string | null>(null);
  const [phase, setPhase] = useState<Phase>("pick");
  const [preview, setPreview] = useState<ImportProfilePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [analysisMs, setAnalysisMs] = useState(0);
  const [result, setResult] = useState<ImportProfileResult | null>(null);
  const [totalMs, setTotalMs] = useState(0);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pendingRequest, setPendingRequest] =
    useState<ImportProfileRequest | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const progress = useProfileImportProgress(operation);
  const elapsedMs = useElapsedClock(phase === "analyze", startedAt);
  useCancelAiOnUnmount(operation);

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

  const choose = async () => {
    const file = await openFileDialog({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (typeof file !== "string") return;
    setPath(file);
    setPreview(null);
    setError(null);
    setPhase("pick");
  };

  const analyze = async () => {
    if (!path) return;
    const id = generation_id();
    const start = Date.now();
    setOperation(id);
    setStartedAt(start);
    setPhase("analyze");
    setError(null);
    try {
      const next = await aiService.importProfile({ generation_id: id, path });
      setPreview(next);
      form.reset(previewToFormValues(next));
      setAnalysisMs(Date.now() - start);
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
      setAnalysisMs(Date.now() - start);
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
    phase === "review"
      ? `Analyse terminée en ${formatDuration(analysisMs)}`
      : phase === "done"
        ? "Les éléments choisis ont été enregistrés"
        : "L'IA prépare une proposition ; rien n'est enregistré";

  return (
    <>
      <ModalHost
        open={open}
        icon="upload_file"
        title="Importer depuis un CV"
        subtitle={subtitle}
        cancelLabel={phase === "done" ? "Fermer" : "Annuler"}
        submitLabel="Importer les éléments sélectionnés"
        submitIcon="playlist_add_check"
        submitDisabled={phase !== "review" || marked === 0}
        busy={busy}
        onClose={close}
        {...(phase === "review"
          ? { onSubmit: () => void form.handleSubmit(apply, refuse)(), flush: true }
          : {})}
        width={phase === "review" ? "880px" : "720px"}
      >
        {phase === "pick" ? (
          <PickFile path={path} onChoose={() => void choose()} />
        ) : null}
        {phase === "pick" && path ? (
          <div className="mt-4">
            <Button
              variant="primary"
              icon="auto_awesome"
              className="w-full"
              onClick={() => void analyze()}
            >
              Analyser le CV
            </Button>
          </div>
        ) : null}
        {phase === "analyze" ? (
          <ImportAnalysisPanel
            step={progress.step}
            elapsedMs={elapsedMs}
            entries={progress.entries}
          />
        ) : null}
        {phase === "error" ? (
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
        {phase === "review" && preview ? (
          <ImportReviewForm
            preview={preview}
            entries={progress.entries}
            formId={formId}
            form={form}
            formError={formError}
            onSubmit={(values) => void apply(values)}
          />
        ) : null}
        {phase === "done" && result ? (
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
  path,
  onChoose,
}: {
  path: string | null;
  onChoose: () => void;
}) {
  return (
    <div className="pt-3">
      <button
        type="button"
        onClick={onChoose}
        className="flex w-full flex-col items-center gap-2 rounded-card border border-dashed border-line px-6 py-6 text-center"
      >
        <Icon
          name={path ? "picture_as_pdf" : "upload_file"}
          size={22}
          className="text-ink-faint"
        />
        <span className="text-body font-medium text-ink">
          {path ? path.split("/").at(-1) : "Choisir un CV PDF"}
        </span>
        <span className="text-meta text-ink-muted">
          Lecture locale · 10 Mo maximum
        </span>
      </button>
    </div>
  );
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
