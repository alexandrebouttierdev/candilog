import { formatAiSummary } from "@/shared/lib/duration";
import { Button, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { AiStopButton, type MatchScore } from "@/features/ai";
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
                      <span className="max-w-full truncate rounded-button bg-surface px-2.5 py-1 font-mono text-meta text-accent">{vm.selectedFile.name}</span>
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
                  <ScoreBreakdown score={vm.result.score} />
                </div>
              </DocumentPanel>
              <DocumentPanel title="Recommandations" icon="tips_and_updates">
                {vm.result.analysis.recommendations.length || vm.result.score.missing.length ? (
                  <ul className="divide-y divide-line">
                    {vm.result.analysis.recommendations.map((recommendation, i) => (
                      <li key={i} className="flex flex-col gap-1.5 px-4 py-3">
                        <span className="text-label font-medium text-accent">
                          {recommendation.target_requirement || labelSection(recommendation.section)}
                        </span>
                        {recommendation.reason ? (
                          <p className="text-body text-ink-muted">{recommendation.reason}</p>
                        ) : null}
                        <p className="text-body text-ink-muted">{recommendation.proposed_text}</p>
                        {recommendation.source_evidence.length ? (
                          <p className="text-meta text-ink-faint">
                            Preuve dans le CV : {recommendation.source_evidence.join(" · ")}
                          </p>
                        ) : null}
                        <span className="flex items-center gap-1.5 text-meta text-ink-faint">
                          <Icon name="info" size={14} />
                          À appliquer dans l’éditeur de CV
                        </span>
                      </li>
                    ))}
                    {vm.result.score.missing.map((requirement) => (
                      <li key={`missing-${requirement}`} className="flex flex-col gap-1.5 px-4 py-3">
                        <span className="text-label font-medium text-warning">
                          Exigence absente : {requirement}
                        </span>
                        <p className="text-body text-ink-muted">
                          Cette exigence apparaît dans l’offre, mais aucune preuve n’a été trouvée
                          dans le CV. Ne l’ajoutez que si vous la possédez réellement.
                        </p>
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

function ScoreBreakdown({ score }: { score: MatchScore }) {
  if (score.breakdown.length === 0) return null;
  const totalWeight = score.breakdown.reduce((total, item) => total + item.weight, 0);

  return (
    <div className="space-y-2 border-t border-line pt-4 sm:col-span-2" aria-label="Détail du score ATS">
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-label font-semibold text-ink">Détail du score</h3>
        <span className="text-meta text-ink-faint">Pondération adaptée à l’offre</span>
      </div>
      <dl className="grid gap-x-5 gap-y-2 sm:grid-cols-2">
        {score.breakdown.map((item) => {
          const weight = totalWeight > 0 ? Math.round((item.weight / totalWeight) * 100) : 0;
          return (
            <div key={item.category} className="flex items-center justify-between gap-3">
              <dt className="min-w-0 truncate text-label text-ink-muted">{item.label}</dt>
              <dd className="flex-none tabular text-label font-medium text-ink">
                {item.score} / 100
                <span className="ml-1.5 font-normal text-ink-faint">· poids {weight} %</span>
              </dd>
            </div>
          );
        })}
      </dl>
      {score.critical_requirements_penalty > 0 ? (
        <p className="flex items-center gap-1.5 text-meta text-danger-text">
          <Icon name="warning" size={14} />
          Exigence réglementaire obligatoire absente : −{score.critical_requirements_penalty} points.
        </p>
      ) : null}
    </div>
  );
}
