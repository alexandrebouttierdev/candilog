import { useState } from "react";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import type { ImportedResumeAnalysis } from "@/features/ai/model/types";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { useAiTimer } from "@/features/ai/viewmodel/useAiTimer";
import { formatDuration } from "@/shared/lib/duration";
import { AppError } from "@/shared/types/app-error";
import { Button, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { A4Preview, AiProgress, DocumentPanel, ScoreBadge } from "../components/DocumentUi";
import { ChampOffre, HeaderBadge, labelSection, Screen, TexteNonVerifie, message } from "./documentPageSupport";

export function ResumeAnalysisPage() {
  const [job_offer, setJobOffer] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [result, setResult] = useState<ImportedResumeAnalysis | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useAiProgress(operation);
  useCancelAiOnUnmount(operation);
  const timer = useAiTimer(operation !== null);
  const run = async () => {
    if (!job_offer.trim()) { setError("Collez l’offre ciblée avant de choisir le CV PDF."); return; }
    const id = generation_id();
    setOperation(id);
    setError(null);
    timer.start();
    try {
      const next = await aiService.analyzeResume({ generation_id: id, job_offer });
      if (next !== null) {
        timer.stop();
        setResult(next);
      }
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };
  return (
    <Screen header={
      <PageHeader
        icon="query_stats"
        title="Analyse de CV"
        subtitle="Comparez un PDF à l’offre ciblée"
        badge={
          <>
            {timer.durationMs !== null && operation === null ? (
              <HeaderBadge icon="schedule">Analysé en {formatDuration(timer.durationMs)}</HeaderBadge>
            ) : null}
            <HeaderBadge icon="lock">Lecture locale</HeaderBadge>
          </>
        }
        primary={<Button variant="primary" icon="bolt" disabled={operation !== null} onClick={() => void run()}>Analyser le CV</Button>}
      />
    }>
      <div className="grid gap-4 xl:grid-cols-[400px_minmax(480px,1fr)]">
        <div className="space-y-4">
          <DocumentPanel title="Document à analyser" icon="upload_file">
            <div className="space-y-4 p-4">
              <button type="button" onClick={() => void run()} className="flex w-full flex-col items-center gap-2 rounded-card border border-dashed border-accent-border bg-accent-tint px-5 py-8 text-center">
                <Icon name="upload_file" size={28} className="text-accent" />
                <span className="font-medium text-ink">Choisir et analyser un CV PDF</span>
                <span className="text-meta text-ink-muted">PDF uniquement · 10 Mo maximum</span>
              </button>
              <ChampOffre label="Offre ciblée" required rows={13} value={job_offer} onChange={setJobOffer} />
              {operation ? <AiProgress progress={progress} elapsedMs={timer.elapsedMs} /> : null}
              {error ? <ErrorBanner title="Analyse impossible" message={error} /> : null}
            </div>
          </DocumentPanel>
        </div>
        <div className="space-y-4">
          {result ? (
            <>
              <DocumentPanel title="Résultat" icon="analytics">
                <div className="grid gap-5 p-4 sm:grid-cols-[auto_1fr]">
                  <ScoreBadge value={result.score.total} />
                  <div className="space-y-2">
                    <p className="text-body leading-relaxed text-ink-muted">{result.analysis.recap}</p>
                    <TexteNonVerifie />
                  </div>
                </div>
              </DocumentPanel>
              <DocumentPanel title="Recommandations" icon="tips_and_updates">
                {result.analysis.recommendations.length ? (
                  <ul className="divide-y divide-line">
                    {result.analysis.recommendations.map((recommendation, i) => (
                      <li key={i} className="flex flex-col gap-1.5 px-4 py-3">
                        <span className="text-label font-medium text-accent">
                          {labelSection(recommendation.section)}
                        </span>
                        <p className="text-body text-ink-muted">{recommendation.proposed_text}</p>
                        <span className="flex items-center gap-1.5 text-meta text-ink-faint">
                          <Icon name="info" size={14} />
                          À appliquer dans l’éditeur de CV
                        </span>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <EmptyState icon="tips_and_updates" title="Aucune recommandation" description="Le modèle n’a proposé aucune reformulation pour cette analyse." />
                )}
              </DocumentPanel>
              <DocumentPanel title="Aperçu du CV lu" icon="visibility"><A4Preview resume={result.resume} /></DocumentPanel>
            </>
          ) : (
            <DocumentPanel title="Résultat de l’analyse" icon="analytics">
              <EmptyState icon="query_stats" title="Prêt à analyser" description="Le score ATS, les écarts et les recommandations apparaîtront ici." />
            </DocumentPanel>
          )}
        </div>
      </div>
    </Screen>
  );
}
