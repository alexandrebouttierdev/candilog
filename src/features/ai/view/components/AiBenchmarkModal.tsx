import { useEffect, useState } from "react";
import { cancelAiOperation, runUserBenchmark } from "../../viewmodel/importProfile";
import type { UserBenchmarkResult } from "@/shared/types/generated/ai";
import { ModalHost } from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";

const QUALITY_LABELS: Record<UserBenchmarkResult["quality"], string> = {
  weak: "Faible",
  average: "Moyen",
  fair: "Correct",
  good: "Bon",
  very_good: "Très bon",
  excellent: "Excellent",
};

function formatSeconds(ms: number): string {
  return `${(ms / 1000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} s`;
}

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

  return (
    <ModalHost
      open={open}
      icon="bolt"
      title="Tester l'IA"
      onClose={onClose}
      cancelLabel="Fermer"
      busy={false}
    >
      {open ? (
        <BenchmarkSession
          key={session}
          modelLabel={modelLabel}
          onClose={onClose}
          onRetest={() => setSession((value) => value + 1)}
        />
      ) : null}
    </ModalHost>
  );
}

function BenchmarkSession({
  modelLabel,
  onClose,
  onRetest,
}: {
  modelLabel: string;
  onClose: () => void;
  onRetest: () => void;
}) {
  const [phase, setPhase] = useState<"running" | "done" | "error">("running");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [result, setResult] = useState<UserBenchmarkResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [generationId] = useState(() => crypto.randomUUID());
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    const started = Date.now();
    const timer = setInterval(() => setElapsedMs(Date.now() - started), 200);
    void runUserBenchmark(generationId)
      .then((payload) => {
        setResult(payload);
        setPhase("done");
      })
      .catch((err: unknown) => {
        if (err instanceof AppError && err.code === "cancelled") {
          onClose();
          return;
        }
        setError(err instanceof AppError ? err.message : "Le test a échoué.");
        setPhase("error");
      })
      .finally(() => clearInterval(timer));
    return () => clearInterval(timer);
  }, [attempt, generationId, onClose]);

  const stop = () => {
    void cancelAiOperation(generationId).finally(onClose);
  };

  return (
    <>
      {phase === "running" && (
        <div className="flex flex-col gap-3 p-4">
          <p className="text-body text-ink-muted">Analyse du CV de référence ({modelLabel})…</p>
          <p className="text-note text-ink-faint">
            Temps écoulé : {formatSeconds(elapsedMs)}
          </p>
          <button
            type="button"
            onClick={stop}
            className="self-start text-label font-semibold text-accent"
          >
            Arrêter
          </button>
        </div>
      )}
      {phase === "error" && error ? (
        <div className="flex flex-col gap-3 p-4">
          <p className="text-body text-danger">{error}</p>
          <button
            type="button"
            onClick={() => {
              setPhase("running");
              setError(null);
              setElapsedMs(0);
              setAttempt((value) => value + 1);
            }}
            className="self-start text-label font-semibold text-accent"
          >
            Réessayer
          </button>
        </div>
      ) : null}
      {phase === "done" && result ? (
        <div className="flex flex-col gap-4 p-4">
          {result.remote_warning ? (
            <p className="rounded-button border border-line bg-surface-alt px-3 py-2 text-note text-ink">
              Le CV de benchmark sera envoyé au fournisseur configuré.
            </p>
          ) : null}
          <div className="flex items-end justify-between gap-4">
            <div>
              <p className="text-[1.75rem] font-semibold leading-tight text-ink">{result.score} / 100</p>
              <p className="text-section text-ink-muted">{QUALITY_LABELS[result.quality]}</p>
            </div>
            <div className="text-right">
              <p className="text-section font-medium text-ink">{formatSeconds(result.metrics.total_ms)}</p>
              <p className="text-note text-ink-faint">sur cet ordinateur</p>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-2 text-note">
            <span className="text-ink-muted">Temps IA</span>
            <span className="text-right tabular-nums text-ink">{formatSeconds(result.metrics.llm_ms)}</span>
            <span className="text-ink-muted">Vitesse</span>
            <span className="text-right tabular-nums text-ink">
              {result.metrics.tokens_per_second != null
                ? `${result.metrics.tokens_per_second.toLocaleString("fr-FR", { maximumFractionDigits: 1 })} tokens/s`
                : "N/A"}
            </span>
          </div>
          <ul className="flex flex-col gap-1 border-t border-line-soft pt-3 text-note">
            {result.categories.map((cat) => (
              <li key={cat.label} className="flex justify-between gap-2 text-ink-muted">
                <span>{cat.label}</span>
                <span className="tabular-nums text-ink">
                  {cat.score} / {cat.max_score}
                </span>
              </li>
            ))}
          </ul>
          <button type="button" onClick={onRetest} className="self-start text-label font-semibold text-accent">
            Retester
          </button>
        </div>
      ) : null}
    </>
  );
}
