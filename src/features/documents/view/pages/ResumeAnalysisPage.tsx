import { useState } from "react";
import { formatAiSummary, formatElapsed } from "@/shared/lib/duration";
import { Button, EmptyState, ErrorBanner, StatusGlyph } from "@/shared/ui";
import type { GlyphTone } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { MatchScore } from "@/features/ai";
import type { RequirementEvaluation, RequirementMatchKind } from "@/shared/types/generated/ai";
import { useResumeAnalysisViewModel } from "../../viewmodel/useResumeAnalysisViewModel";
import { AiProgress } from "../components/DocumentUi";
import { GeneratorFrame, PaneSection } from "../components/GeneratorFrame";
import { OfferSource } from "../components/OfferSource";
import { StopGenerationDialog } from "../components/StopGenerationDialog";
import { labelSection, TexteNonVerifie } from "./documentPageSupport";

/** Verdict d'une exigence, tel que la liste l'affiche. */
const VERDICT: Record<RequirementMatchKind, { label: string; glyph: GlyphTone; chip: string }> = {
  exact: { label: "couverte", glyph: "g", chip: "bg-tint-g-bg text-tint-g-tx" },
  equivalent: { label: "couverte", glyph: "g", chip: "bg-tint-g-bg text-tint-g-tx" },
  transferable: { label: "partielle", glyph: "a", chip: "bg-tint-ac-bg text-tint-ac-tx" },
  partial: { label: "partielle", glyph: "a", chip: "bg-tint-ac-bg text-tint-ac-tx" },
  missing: { label: "absente", glyph: "c", chip: "bg-tint-c-bg text-tint-c-tx" },
  unknown: { label: "non évaluée", glyph: "n", chip: "bg-chip text-tx-4" },
};

function covered(evaluation: RequirementEvaluation): boolean {
  return evaluation.match_kind === "exact" || evaluation.match_kind === "equivalent";
}

/** Verdict d'ensemble sous le score : prêt, ou à corriger avant envoi. */
function scoreVerdict(total: number): { label: string; chip: string; text: string; bar: string } {
  if (total >= 80) return { label: "prêt à envoyer", chip: "bg-tint-g-bg text-tint-g-tx", text: "text-tint-g-tx", bar: "bg-st-g" };
  if (total >= 60) return { label: "à renforcer", chip: "bg-tint-ac-bg text-tint-ac-tx", text: "text-ac-tx", bar: "bg-ac" };
  return { label: "à corriger avant envoi", chip: "bg-tint-c-bg text-tint-c-tx", text: "text-tint-c-tx", bar: "bg-st-c" };
}

/**
 * Analyse d'un CV face à une offre (`screens/09-resume-analysis.png`) : surcouche plein
 * écran. À gauche, ce qui est comparé et le score calculé par Candilog ; au centre,
 * chaque exigence de l'offre avec la preuve trouvée dans le CV — ou l'absence de preuve.
 */
export function ResumeAnalysisPage() {
  const vm = useResumeAnalysisViewModel();
  const running = vm.operation !== null;
  const [askStop, setAskStop] = useState(false);
  const result = vm.result;
  const canRun = !vm.selecting && vm.selectedFile !== null && vm.jobOffer.trim() !== "" && !running;
  // Les exigences manquantes que le score liste sans évaluation détaillée restent visibles :
  // une absence non citée se lirait comme une exigence couverte.
  const evaluations: RequirementEvaluation[] = result
    ? [
        ...result.score.evaluations,
        ...result.score.missing
          .filter((missing) => !result.score.evaluations.some((item) => item.requirement === missing))
          .map(
            (requirement): RequirementEvaluation => ({
              requirement,
              category: "other",
              importance: "important",
              match_kind: "missing",
              score: null,
              evidence: null,
            }),
          ),
      ]
    : [];
  const coveredCount = evaluations.filter(covered).length;

  return (
    <GeneratorFrame
      title="Analyse face à l’offre"
      actions={
        running ? (
          <Button size="bar" shortcut="mod+." disabled={vm.stopping} onClick={() => setAskStop(true)}>
            {vm.stopping ? "Arrêt…" : "Arrêter"}
          </Button>
        ) : (
          <>
            {vm.canReset ? (
              <Button variant="ghost" size="bar" onClick={vm.reset}>
                Réinitialiser
              </Button>
            ) : null}
            <Button variant="primary" size="bar" shortcut="mod+enter" disabled={!canRun} onClick={() => void vm.run()}>
              Analyser le CV
            </Button>
          </>
        )
      }
      left={
        <>
          <PaneSection title="Ce qui est comparé">
            <button
              type="button"
              aria-label={vm.selectedFile ? "Changer de fichier" : "Choisir un fichier"}
              disabled={vm.selecting || running}
              onClick={() => void vm.selectFile()}
              className={cn(
                "flex w-full items-center gap-2.5 rounded-r8 border px-2.5 py-2 text-left",
                vm.selectedFile ? "border-bd-soft bg-panel" : "border-dashed border-bd-menu hover:border-ac",
              )}
            >
              <span aria-hidden className="flex h-5 w-4 flex-none flex-col gap-[2px] rounded-[2px] border border-bd-menu bg-panel px-[3px] pt-[4px]">
                <span className="h-px w-full bg-tx-6" />
                <span className="h-px w-4/5 bg-tx-6" />
              </span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-ui text-tx-2">
                  {vm.selecting ? "Sélection du fichier…" : (vm.selectedFile?.name ?? "Choisir un CV (PDF)")}
                </span>
                <span className="block text-tiny text-tx-5">
                  {vm.selectedFile ? "Changer de fichier" : "PDF uniquement · 10 Mo maximum"}
                </span>
              </span>
            </button>
            <p className="my-1.5 pl-2.5 font-mono text-caps text-tx-6">face à</p>
            <OfferSource
              value={vm.jobOffer}
              onChange={vm.setJobOffer}
              readClipboard={vm.readClipboard}
              disabled={running}
              textLabel="Offre ciblée"
            />
          </PaneSection>
          {result ? <ScorePane score={result.score} /> : null}
          {vm.error ? <ErrorBanner title="Analyse impossible" message={vm.error} /> : null}
        </>
      }
      status={
        running
          ? `${vm.progress?.step ?? "Préparation"} · ${formatElapsed(vm.elapsedMs)}`
          : result && vm.metrics
            ? `${formatAiSummary("analysé", vm.metrics.elapsed_ms, vm.metrics.tokens_used)} · lecture ${result.method_used === "vision" ? "visuelle" : "du texte"}${result.fallback_used ? " (repli)" : ""}`
            : "choisissez un CV et une offre"
      }
      keys={running ? [{ label: "Arrêter", shortcut: "mod+." }] : [{ label: "Analyser", shortcut: "mod+enter" }]}
      {...(running ? { onStop: () => setAskStop(true) } : {})}
      closeDisabled={running}
      {...(canRun ? { onSubmit: () => void vm.run() } : {})}
    >
      <div className="bg-panel flex-1 overflow-y-auto">
        <div className="max-w-[760px] px-[26px] pt-5 pb-8">
          {running && !vm.stopping ? <AiProgress progress={vm.progress} elapsedMs={vm.elapsedMs} /> : null}
          {result ? (
            <>
              <h1 className="serif-title text-[21px] leading-tight text-tx">
                {evaluations.length > 0
                  ? `${coveredCount} exigence${coveredCount > 1 ? "s" : ""} couverte${coveredCount > 1 ? "s" : ""} sur ${evaluations.length}`
                  : "Analyse terminée"}
              </h1>
              {vm.metrics ? (
                <p className="mt-1 font-mono text-caps text-tx-5">
                  {formatAiSummary("Analysé", vm.metrics.elapsed_ms, vm.metrics.tokens_used)}
                </p>
              ) : null}
              <p className="mt-1.5 text-small leading-[1.55] text-tx-4">
                Candilog a lu l’annonce ligne par ligne et cherché dans votre CV ce qui y répond. Chaque exigence est
                citée avec la preuve trouvée — ou l’absence de preuve.
              </p>

              {evaluations.length > 0 ? (
                <section aria-label="Exigences de l’offre" className="mt-5">
                  <div className="mb-2 flex items-center gap-3">
                    <h2 className="caps">Exigences de l’offre</h2>
                    <span className="font-mono text-caps text-tx-6">{evaluations.length}</span>
                    <span className="ml-auto flex items-center gap-3 text-tiny text-tx-5">
                      {(["g", "a", "c"] as const).map((tone) => (
                        <span key={tone} className="flex items-center gap-1">
                          <StatusGlyph tone={tone} small />
                          {tone === "g" ? "couverte" : tone === "a" ? "partielle" : "absente"}
                        </span>
                      ))}
                    </span>
                  </div>
                  <ul className="flex flex-col">
                    {evaluations.map((evaluation, index) => {
                      const verdict = VERDICT[evaluation.match_kind];
                      return (
                        <li key={`${evaluation.requirement}-${index}`} className="border-b border-bd-soft py-2.5 last:border-b-0">
                          <div className="flex items-center gap-2.5">
                            <StatusGlyph tone={verdict.glyph} />
                            <span className="min-w-0 flex-1 text-row text-tx">{evaluation.requirement}</span>
                            <span className={cn("inline-flex h-5 flex-none items-center rounded-r5 px-1.5 text-tiny", verdict.chip)}>
                              {verdict.label}
                            </span>
                          </div>
                          <p className="mt-1 flex gap-2 pl-[21px] text-small leading-[1.5]">
                            <span className="flex-none font-mono text-caps text-tx-6">
                              {evaluation.evidence ? "CV" : "introuvable"}
                            </span>
                            <span className={evaluation.evidence ? "text-tx-3" : "text-tx-5 italic"}>
                              {evaluation.evidence ?? "Aucune preuve trouvée dans le CV."}
                            </span>
                          </p>
                        </li>
                      );
                    })}
                  </ul>
                </section>
              ) : null}

              {result.analysis.recommendations.length > 0 ? (
                <section aria-label="Recommandations" className="mt-6">
                  <h2 className="caps mb-2">Recommandations</h2>
                  <ul className="flex flex-col gap-2.5">
                    {result.analysis.recommendations.map((recommendation, index) => (
                      <li key={index} className="rounded-r8 bg-group px-3 py-2.5">
                        <p className="text-small font-medium text-ac-tx">
                          {recommendation.target_requirement || labelSection(recommendation.section)}
                        </p>
                        {recommendation.reason ? <p className="mt-1 text-small text-tx-3">{recommendation.reason}</p> : null}
                        <p className="mt-1 text-small text-tx-2">{recommendation.proposed_text}</p>
                        {recommendation.source_evidence.length ? (
                          <p className="mt-1 text-tiny text-tx-5">
                            Preuve dans le CV : {recommendation.source_evidence.join(" · ")}
                          </p>
                        ) : null}
                      </li>
                    ))}
                  </ul>
                </section>
              ) : null}

              {result.analysis.recap ? (
                <section aria-label="Commentaire du modèle" className="mt-6">
                  <h2 className="caps mb-2">Commentaire</h2>
                  <p className="text-small leading-[1.55] text-tx-3">{result.analysis.recap}</p>
                  <div className="mt-2">
                    <TexteNonVerifie />
                  </div>
                </section>
              ) : null}
            </>
          ) : running ? null : (
            <div className="pt-[10vh]">
              <EmptyState
                icon="query_stats"
                title="Prêt à analyser"
                description="Choisissez votre CV en PDF et l’offre visée : chaque exigence sera confrontée au CV, preuve à l’appui."
              />
            </div>
          )}
        </div>
      </div>
      <StopGenerationDialog
        open={askStop && running && !vm.stopping}
        step={vm.progress?.step ?? null}
        elapsedMs={vm.elapsedMs}
        onKeepGoing={() => setAskStop(false)}
        onStop={() => {
          setAskStop(false);
          void vm.stop();
        }}
      />
    </GeneratorFrame>
  );
}

/** Score et détail, colonne de gauche une fois l'analyse faite. */
function ScorePane({ score }: { score: MatchScore }) {
  const total = Math.round(score.total);
  const verdict = scoreVerdict(total);
  return (
    <>
      <section aria-label="Score" className="mb-4 border-t border-bd-soft pt-3.5">
        <p className="flex items-baseline gap-1.5">
          <span className={cn("serif-title text-[28px] leading-none", verdict.text)}>{total}</span>
          <span className="text-sub text-tx-5">/ 100</span>
          <span className={cn("ml-auto inline-flex h-5 items-center rounded-r5 px-1.5 text-tiny", verdict.chip)}>{verdict.label}</span>
        </p>
        <div aria-hidden className="mt-2 flex gap-0.5">
          {Array.from({ length: 20 }, (_, index) => (
            <span key={index} className={cn("h-[3px] flex-1 rounded-r2", index < Math.round(total / 5) ? verdict.bar : "bg-chip")} />
          ))}
        </div>
        {score.critical_requirements_penalty > 0 ? (
          <p className="mt-2 text-tiny text-tint-c-tx">
            Exigence réglementaire obligatoire absente : −{score.critical_requirements_penalty} points.
          </p>
        ) : null}
      </section>
      {score.breakdown.length > 0 ? (
        <PaneSection title="Détail du score" className="border-t border-bd-soft pt-3">
          <dl aria-label="Détail du score ATS" className="flex flex-col gap-2.5">
            {score.breakdown.map((item) => (
              <div key={item.category}>
                <div className="flex items-baseline justify-between gap-2">
                  <dt className="truncate text-small text-tx-2">{item.label}</dt>
                  <dd className="flex-none font-mono text-caps text-tx-4">{item.score} / 100</dd>
                </div>
                <div aria-hidden className="mt-1 h-[3px] rounded-r2 bg-chip">
                  <div
                    className={cn("h-full rounded-r2", item.score >= 80 ? "bg-st-g" : "bg-ac")}
                    style={{ width: `${Math.max(0, Math.min(100, item.score))}%` }}
                  />
                </div>
              </div>
            ))}
          </dl>
        </PaneSection>
      ) : null}
    </>
  );
}
