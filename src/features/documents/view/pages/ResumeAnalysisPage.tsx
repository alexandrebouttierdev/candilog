import { useState } from "react";
import { aiService } from "@/features/ai/services/aiService";
import type { AiExecution, ImportedResumeAnalysis, SelectedResumeFile } from "@/features/ai/model/types";
import { AiStopButton } from "@/features/ai/view/components/AiStopButton";
import { useAiOperation } from "@/features/ai/viewmodel/useAiOperation";
import { useAiProgress } from "@/features/ai/viewmodel/useAiProgress";
import { useAiTimer } from "@/features/ai/viewmodel/useAiTimer";
import { formatAiSummary } from "@/shared/lib/duration";
import { AppError } from "@/shared/types/app-error";
import { Button, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { A4Preview, AiProgress, DocumentPanel, ScoreBadge } from "../components/DocumentUi";
import { ChampOffre, HeaderBadge, labelSection, Screen, TexteNonVerifie, message } from "./documentPageSupport";

export function ResumeAnalysisPage() {
  const [job_offer, setJobOffer] = useState("");
  const [selectedFile, setSelectedFile] = useState<SelectedResumeFile | null>(null);
  const [selecting, setSelecting] = useState(false);
  const { operation, stopping, start, stop, finish, isCurrent } = useAiOperation();
  const [result, setResult] = useState<ImportedResumeAnalysis | null>(null);
  const [metrics, setMetrics] = useState<Pick<AiExecution<unknown>, "elapsed_ms" | "tokens_used"> | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useAiProgress(stopping ? null : (operation?.id ?? null));
  const timer = useAiTimer(operation !== null && !stopping);

  const selectFile = async () => {
    setSelecting(true);
    setError(null);
    try {
      const selected = await aiService.selectResumeFile();
      if (selected !== null) {
        setSelectedFile(selected);
        setResult(null);
        setMetrics(null);
      }
    } catch (caught) {
      setError(message(caught));
    } finally {
      setSelecting(false);
    }
  };

  const run = async () => {
    if (!selectedFile) { setError("Choisissez le CV PDF à analyser."); return; }
    if (!job_offer.trim()) { setError("Collez l’offre ciblée avant de lancer l’analyse."); return; }
    let id: string;
    try {
      id = start("analyse");
    } catch (caught) {
      setError(message(caught));
      return;
    }
    setError(null);
    timer.start();
    try {
      const execution = await aiService.analyzeResume({
        generation_id: id,
        job_offer,
        file_path: selectedFile.path,
      });
      if (!isCurrent(id)) return;
      timer.stop();
      setResult(execution.output);
      setMetrics(execution);
    } catch (e) {
      if (isCurrent(id) && !(e instanceof AppError && e.code === "CANCELLED")) {
        setError(message(e));
      }
    } finally {
      finish(id);
    }
  };
  return (
    <Screen header={
      <PageHeader
        icon="query_stats"
        title="Analyse de CV"
        subtitle="Comparez un CV à l’offre ciblée"
        badge={
          <>
            {metrics !== null && operation === null ? (
              <HeaderBadge icon="schedule">
                {formatAiSummary("Analysé", metrics.elapsed_ms, metrics.tokens_used)}
              </HeaderBadge>
            ) : null}
          </>
        }
      />
    }>
      <div className="grid gap-4 xl:grid-cols-[400px_minmax(480px,1fr)]">
        <div className="space-y-4">
          <DocumentPanel title="Document à analyser" icon="upload_file">
            <div className="space-y-4 p-4">
              {operation ? (
                <>
                  {stopping ? null : <AiProgress progress={progress} elapsedMs={timer.elapsedMs} />}
                  <AiStopButton
                    stopping={stopping}
                    onStop={() => void stop().catch((caught: unknown) => setError(message(caught)))}
                  />
                </>
              ) : (
                <>
                  <button type="button" aria-label={selectedFile ? "Changer de fichier" : "Choisir un fichier"} disabled={selecting} onClick={() => void selectFile()} className="flex w-full flex-col items-center gap-2 rounded-card border border-dashed border-accent-border bg-accent-tint px-5 py-8 text-center disabled:cursor-default">
                    <Icon name="upload_file" size={28} className="text-accent" />
                    <span className="font-medium text-ink">
                      {selecting ? "Sélection du fichier…" : selectedFile ? "Changer de fichier" : "Choisir un fichier"}
                    </span>
                    {selectedFile ? (
                      <span className="font-mono text-meta text-accent">{selectedFile.name}</span>
                    ) : null}
                    <span className="text-meta text-ink-muted">PDF uniquement · 10 Mo maximum</span>
                  </button>
                  <ChampOffre label="Offre ciblée" required rows={13} value={job_offer} onChange={setJobOffer} />
                  {error ? <ErrorBanner title="Analyse impossible" message={error} /> : null}
                  <Button variant="primary" icon="bolt" className="w-full" disabled={selecting || selectedFile === null || !job_offer.trim()} onClick={() => void run()}>Analyser le CV</Button>
                </>
              )}
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
