import { useState } from "react";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import type { ImportedResumeAnalysis } from "@/features/ai/model/types";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { AppError } from "@/shared/types/app-error";
import { Button, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { A4Preview, AiProgress, DocumentPanel, ScoreBadge } from "../components/DocumentUi";
import { ChampOffre, HeaderBadge, Screen, TexteNonVerifie, message } from "./documentPageSupport";

export function ResumeAnalysisPage() {
  const [job_offer, setJobOffer] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [result, setResult] = useState<ImportedResumeAnalysis | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useAiProgress(operation);
  useCancelAiOnUnmount(operation);
  const run = async () => {
    if (!job_offer.trim()) { setError("Collez l’offre ciblée avant de choisir le CV PDF."); return; }
    const id = generation_id();
    setOperation(id);
    setError(null);
    try {
      const next = await aiService.analyzeResume({ generation_id: id, job_offer });
      if (next !== null) setResult(next);
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
        badge={<HeaderBadge icon="lock">Lecture locale</HeaderBadge>}
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
              {operation ? <AiProgress progress={progress} /> : null}
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
                <ul className="divide-y divide-line">{result.analysis.suggestions.map((s, i) => <li key={i} className="flex gap-3 px-4 py-3 text-body text-ink-muted"><span className="tabular text-accent">{i + 1}</span>{s}</li>)}</ul>
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
