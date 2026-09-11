import { useEffect, useRef, useState } from "react";
import { cancelAiOperation, runUserBenchmark } from "../../viewmodel/importProfile";
import type { UserBenchmarkResult } from "@/shared/types/generated/ai";
import { Button, Icon, ModalHost, StatusPill } from "@/shared/ui";
import type { Tone } from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";
import {
  formatDuration,
  formatElapsed,
  formatTokens,
  formatTokensPerSecond,
} from "@/shared/lib/duration";
import { AiStopButton } from "./AiStopButton";

const QUALITY_LABELS: Record<UserBenchmarkResult["quality"], string> = {
  weak: "Faible",
  average: "Moyen",
  fair: "Correct",
  good: "Bon",
  very_good: "Très bon",
  excellent: "Excellent",
};

const QUALITY_TONES: Record<UserBenchmarkResult["quality"], Tone> = {
  weak: "danger",
  average: "warning",
  fair: "warning",
  good: "accent",
  very_good: "success",
  excellent: "success",
};

export function AiBenchmarkModal({
  open,
  onClose,
  modelLabel,
}: {
  open: boolean;
  onClose: () => void;
  modelLabel: string;
}) {
  const [session, setSession] = useState(0);
  const [busy, setBusy] = useState(false);

  return (
    <ModalHost
      open={open}
      icon="bolt"
      title="Tester l'IA"
      subtitle={busy ? modelLabel : undefined}
      onClose={onClose}
      cancelLabel="Fermer"
      busy={busy}
    >
      {open ? (
        <BenchmarkSession
          key={session}
          modelLabel={modelLabel}
          onClose={onClose}
          onRetest={() => setSession((value) => value + 1)}
          onBusyChange={setBusy}
        />
      ) : null}
    </ModalHost>
  );
}

function BenchmarkSession({
  modelLabel,
  onClose,
  onRetest,
  onBusyChange,
}: {
  modelLabel: string;
  onClose: () => void;
  onRetest: () => void;
  onBusyChange: (busy: boolean) => void;
}) {
  const [phase, setPhase] = useState<"running" | "done" | "error">("running");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [result, setResult] = useState<UserBenchmarkResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);
  const [stopping, setStopping] = useState(false);
  const onCloseRef = useRef(onClose);
  const generationIdRef = useRef<string | null>(null);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    onBusyChange(phase === "running");
  }, [onBusyChange, phase]);

  useEffect(() => {
    // Nouvel id à chaque lancement. Ne pas dépendre de `onClose` : les parents passent
    // souvent `() => setOpen(false)`, recréé à chaque rendu — ça relançait le test avec
    // le même id, `start` annulait la génération en cours, et le catch fermait la modale.
    const generationId = crypto.randomUUID();
    generationIdRef.current = generationId;
    let active = true;
    const started = Date.now();
    const timer = setInterval(() => {
      if (active) setElapsedMs(Date.now() - started);
    }, 200);
    void runUserBenchmark(generationId)
      .then((payload) => {
        if (!active) return;
        setResult(payload);
        setPhase("done");
      })
      .catch((err: unknown) => {
        if (!active) return;
        // Arrêter ferme déjà la modale ; un cancel de cleanup ne doit pas la refermer.
        if (err instanceof AppError && err.isCancelled) {
          return;
        }
        setError(err instanceof AppError ? err.message : "Le test a échoué.");
        setPhase("error");
      })
      .finally(() => clearInterval(timer));
    return () => {
      active = false;
      clearInterval(timer);
      void cancelAiOperation(generationId);
    };
  }, [attempt]);

  const stop = () => {
    const id = generationIdRef.current;
    setStopping(true);
    if (id) {
      void cancelAiOperation(id).finally(() => onCloseRef.current());
    } else {
      onCloseRef.current();
    }
  };

  return (
    <>
      {phase === "running" ? (
        <RunningPanel
          modelLabel={modelLabel}
          elapsedMs={elapsedMs}
          stopping={stopping}
          onStop={stop}
        />
      ) : null}
      {phase === "error" && error ? (
        <ErrorPanel
          error={error}
          onRetry={() => {
            setPhase("running");
            setError(null);
            setElapsedMs(0);
            setStopping(false);
            setAttempt((value) => value + 1);
          }}
        />
      ) : null}
      {phase === "done" && result ? <ResultPanel result={result} onRetest={onRetest} /> : null}
    </>
  );
}

function RunningPanel({
  modelLabel,
  elapsedMs,
  stopping,
  onStop,
}: {
  modelLabel: string;
  elapsedMs: number;
  stopping: boolean;
  onStop: () => void;
}) {
  return (
    <div className="flex flex-col gap-4 p-4">
      <div role="status" className="rounded-card border border-accent-border bg-accent-tint p-4">
        <div className="flex items-center gap-2">
          <Icon name="progress_activity" size={17} className="animate-spin text-accent" />
          <p className="flex-1 text-label font-medium text-ink">Analyse du CV de référence…</p>
          <span className="tabular text-meta text-accent">{formatElapsed(elapsedMs)}</span>
        </div>
        <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-surface">
          <div className="import-indeterminate h-full w-1/3 rounded-full bg-accent" />
        </div>
      </div>

      <div className="space-y-2">
        <p className="text-body text-ink-muted">
          Candilog envoie un CV de test à{" "}
          <span className="font-medium text-ink">{modelLabel}</span> pour mesurer la qualité
          d&apos;extraction (identité, expériences, compétences) et la vitesse sur cet
          ordinateur.
        </p>
        <p className="text-note text-ink-faint">
          Sur un modèle local, comptez souvent 1 à 3 minutes. Aucune donnée de votre profil
          n&apos;est utilisée ni enregistrée.
        </p>
      </div>

      <ul className="space-y-1.5 rounded-button border border-line bg-surface-alt px-3 py-2.5 text-note text-ink-muted">
        <li className="flex gap-2">
          <Icon name="description" size={14} className="mt-0.5 flex-none text-ink-faint" />
          Lecture du PDF de référence
        </li>
        <li className="flex gap-2">
          <Icon name="visibility" size={14} className="mt-0.5 flex-none text-ink-faint" />
          Extraction Vision ou Texte selon le modèle
        </li>
        <li className="flex gap-2">
          <Icon name="check_circle" size={14} className="mt-0.5 flex-none text-ink-faint" />
          Comparaison au résultat attendu
        </li>
      </ul>

      <AiStopButton stopping={stopping} onStop={onStop} />
    </div>
  );
}

function ErrorPanel({ error, onRetry }: { error: string; onRetry: () => void }) {
  return (
    <div className="flex flex-col gap-4 p-4">
      <div className="rounded-card border border-danger-border bg-danger-tint px-3 py-3">
        <div className="flex items-start gap-2">
          <Icon name="error" size={17} className="mt-0.5 flex-none text-danger" />
          <div className="min-w-0 space-y-1">
            <p className="text-label font-semibold text-danger">Le test n&apos;a pas abouti</p>
            <p className="text-body text-ink-muted">{error}</p>
          </div>
        </div>
      </div>
      <Button variant="secondary" icon="refresh" onClick={onRetry}>
        Réessayer
      </Button>
    </div>
  );
}

function ResultPanel({
  result,
  onRetest,
}: {
  result: UserBenchmarkResult;
  onRetest: () => void;
}) {
  const methodLabel =
    result.method_used === "vision"
      ? result.fallback_used
        ? "Vision → Texte (repli)"
        : "Vision"
      : "Texte";
  const tokensIn = result.metrics.tokens_input;
  const tokensOut = result.metrics.tokens_output;
  const speed =
    result.metrics.tokens_per_second != null
      ? formatTokensPerSecond(result.metrics.tokens_per_second)
      : null;

  return (
    <div className="flex flex-col gap-4 p-4">
      {result.remote_warning ? (
        <p className="rounded-button border border-warning bg-warning-tint px-3 py-2 text-note text-ink">
          Le CV de benchmark a été envoyé au fournisseur distant configuré.
        </p>
      ) : null}

      <div className="flex items-start justify-between gap-4">
        <div className="space-y-2">
          <p className="text-[1.75rem] font-semibold leading-none tabular-nums text-ink">
            {result.score}
            <span className="text-section font-medium text-ink-faint"> / 100</span>
          </p>
          <StatusPill tone={QUALITY_TONES[result.quality]}>
            {QUALITY_LABELS[result.quality]}
          </StatusPill>
        </div>
        <div className="text-right">
          <p className="text-section font-medium tabular-nums text-ink">
            {formatDuration(result.metrics.total_ms)}
          </p>
          <p className="text-note text-ink-faint">durée totale</p>
        </div>
      </div>

      <p className="text-note text-ink-muted">
        {result.provider_label}
        <span className="text-ink-faint"> · </span>
        {result.model_label}
        <span className="text-ink-faint"> · </span>
        {methodLabel}
      </p>

      <div className="grid grid-cols-2 gap-2">
        <MetricCard label="Temps IA" value={formatDuration(result.metrics.llm_ms)} />
        <MetricCard label="Vitesse" value={speed ?? "Non communiquée"} />
        <MetricCard label="Appels modèle" value={String(result.metrics.llm_calls)} />
        <MetricCard
          label="Hallucinations"
          value={
            result.hallucination_count === 0
              ? "Aucune"
              : String(result.hallucination_count)
          }
        />
        {tokensIn != null ? (
          <MetricCard label="Tokens entrants" value={formatTokens(tokensIn)} />
        ) : null}
        {tokensOut != null ? (
          <MetricCard label="Tokens sortants" value={formatTokens(tokensOut)} />
        ) : null}
      </div>

      <div className="space-y-2 border-t border-line-soft pt-3">
        <p className="text-label font-semibold text-ink">Détail par catégorie</p>
        <ul className="flex flex-col gap-2.5">
          {result.categories.map((cat) => {
            const ratio = cat.max_score > 0 ? Math.min(1, cat.score / cat.max_score) : 0;
            return (
              <li key={cat.label} className="space-y-1">
                <div className="flex justify-between gap-2 text-note">
                  <span className="text-ink-muted">{cat.label}</span>
                  <span className="tabular-nums text-ink">
                    {cat.score} / {cat.max_score}
                  </span>
                </div>
                <div className="h-1 overflow-hidden rounded-full bg-fill">
                  <div
                    className="h-full rounded-full bg-accent"
                    style={{ width: `${ratio * 100}%` }}
                  />
                </div>
              </li>
            );
          })}
        </ul>
      </div>

      <div className="grid grid-cols-3 gap-2 text-note text-ink-faint">
        <span>PDF {formatDuration(result.metrics.pdf_extract_ms)}</span>
        <span>Prétrait. {formatDuration(result.metrics.preprocess_ms)}</span>
        <span>Parse {formatDuration(result.metrics.parse_ms)}</span>
      </div>

      <Button variant="secondary" icon="refresh" onClick={onRetest}>
        Retester
      </Button>
    </div>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-button border border-line bg-surface-alt px-3 py-2">
      <p className="text-note text-ink-faint">{label}</p>
      <p className="mt-0.5 text-label font-semibold tabular-nums text-ink">{value}</p>
    </div>
  );
}
