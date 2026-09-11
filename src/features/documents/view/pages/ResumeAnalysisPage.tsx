import { formatAiSummary } from "@/shared/lib/duration";
import { Button, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { AiStopButton } from "@/features/ai/view/components/AiStopButton";
import { useResumeAnalysisViewModel } from "../../viewmodel/useResumeAnalysisViewModel";
import { AiProgress, DocumentPanel, ScoreBadge } from "../components/DocumentUi";
import { ChampOffre, HeaderBadge, labelSection, Screen, TexteNonVerifie } from "./documentPageSupport";

export function ResumeAnalysisPage() {
  const vm = useResumeAnalysisViewModel();

  return (
    <Screen
      header={
        <PageHeader
          icon="query_stats"
          title="Analyse de CV"
          subtitle="Comparez un CV à l’offre ciblée"
          badge={
            <>
              {vm.metrics !== null && vm.operation === null ? (
                <HeaderBadge icon="schedule">
                  {formatAiSummary("Analysé", vm.metrics.elapsed_ms, vm.metrics.tokens_used)}
                </HeaderBadge>
              ) : null}
            </>
          }
          secondary={
            vm.canReset ? (
              <Button icon="restart_alt" disabled={vm.operation !== null} onClick={vm.reset}>
                Réinitialiser
              </Button>
            ) : undefined
          }
        />
      }
    >
      <div className="grid gap-4 xl:grid-cols-[400px_minmax(480px,1fr)]">
        <div className="space-y-4">
          <DocumentPanel title="Document à analyser" icon="upload_file">
            <div className="space-y-4 p-4">
              {vm.operation ? (
                <>
                  {vm.stopping ? null : (
                    <AiProgress progress={vm.progress} elapsedMs={vm.elapsedMs} />
                  )}
                  <AiStopButton stopping={vm.stopping} onStop={() => void vm.stop()} />
                </>
              ) : (
                <>
                  <button
                    type="button"
                    aria-label={vm.selectedFile ? "Changer de fichier" : "Choisir un fichier"}
                    disabled={vm.selecting}
                    onClick={() => void vm.selectFile()}
                    className="flex w-full flex-col items-center gap-2 rounded-card border border-dashed border-accent-border bg-accent-tint px-5 py-8 text-center disabled:cursor-default"
                  >
                    <Icon name="upload_file" size={28} className="text-accent" />
                    <span className="font-medium text-ink">
                      {vm.selecting
                        ? "Sélection du fichier…"
                        : vm.selectedFile
                          ? "Changer de fichier"
                          : "Choisir un fichier"}
                    </span>
                    {vm.selectedFile ? (
                      <span className="font-mono text-meta text-accent">{vm.selectedFile.name}</span>
                    ) : null}
                    <span className="text-meta text-ink-muted">PDF uniquement · 10 Mo maximum</span>
                  </button>
                  <ChampOffre
                    label="Offre ciblée"
                    required
                    rows={13}
                    value={vm.jobOffer}
                    onChange={vm.setJobOffer}
                    readClipboard={vm.readClipboard}
                  />
                  {vm.error ? <ErrorBanner title="Analyse impossible" message={vm.error} /> : null}
                  <Button
                    variant="primary"
                    icon="bolt"
                    className="w-full"
                    disabled={vm.selecting || vm.selectedFile === null || !vm.jobOffer.trim()}
                    onClick={() => void vm.run()}
                  >
                    Analyser le CV
                  </Button>
                </>
              )}
            </div>
          </DocumentPanel>
        </div>
        <div className="space-y-4">
          {vm.result ? (
            <>
              <DocumentPanel title="Résultat" icon="analytics">
                <div className="grid gap-5 p-4 sm:grid-cols-[auto_1fr]">
                  <ScoreBadge value={vm.result.score.total} />
                  <div className="space-y-2">
                    <p className="text-body leading-relaxed text-ink-muted">{vm.result.analysis.recap}</p>
                    <TexteNonVerifie />
                  </div>
                </div>
              </DocumentPanel>
              <DocumentPanel title="Recommandations" icon="tips_and_updates">
                {vm.result.analysis.recommendations.length ? (
                  <ul className="divide-y divide-line">
                    {vm.result.analysis.recommendations.map((recommendation, i) => (
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
                  <EmptyState
                    icon="tips_and_updates"
                    title="Aucune recommandation"
                    description="Le modèle n’a proposé aucune reformulation pour cette analyse."
                  />
                )}
              </DocumentPanel>
            </>
          ) : (
            <DocumentPanel title="Résultat de l’analyse" icon="analytics">
              <EmptyState
                icon="query_stats"
                title="Prêt à analyser"
                description="Le score ATS, les écarts et les recommandations apparaîtront ici."
              />
            </DocumentPanel>
          )}
        </div>
      </div>
    </Screen>
  );
}
