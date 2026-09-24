import { useMemo, useState, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { formatElapsed, formatTokens } from "@/shared/lib/duration";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { Button, ErrorBanner, FormField, TextInput } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { useResumeEditor } from "../../viewmodel/useResumeEditor";
import { useResumeGeneratorViewModel } from "../../viewmodel/useResumeGeneratorViewModel";
import { AiProgress, OverflowStatus, UndoRedoControls } from "../components/DocumentUi";
import { GeneratorFrame, PaneSection, StepList } from "../components/GeneratorFrame";
import { OfferSource } from "../components/OfferSource";
import { useStepLog } from "../../viewmodel/useStepLog";
import { ProfileSkillChoiceDialog } from "../components/ProfileSkillChoiceDialog";
import { ResumeAtsPanel } from "../components/ResumeAtsPanel";
import { ResumePaper } from "../components/ResumePaper";
import { useProfilePhoto } from "@/features/profile";
import { AiStopButton } from "@/features/ai";
import { generationFromNavigation } from "./documentPageSupport";

/** Étapes annoncées par le backend pendant la génération d'un CV. */
const CV_STEPS = ["Analyse de l'offre", "Adaptation du CV", "Relecture du français", "Analyse ATS"] as const;

/** Feuille vide aux proportions A4, en attendant le document. */
function EmptySheet({ busy }: { busy: boolean }) {
  return (
    <div className="flex flex-1 justify-center p-[26px]">
      <div
        aria-hidden
        className="flex aspect-[210/297] w-full max-w-[520px] flex-col gap-6 rounded-[2px] bg-paper px-[8%] pt-[7%] shadow-sheet"
      >
        {[0, 1, 2, 3, 4].map((block) => (
          <div key={block} className="flex flex-col gap-2">
            <span className={cn("h-2 w-1/4 rounded-r2 bg-chip", busy && "animate-pulse")} />
            <span className={cn("h-1.5 w-full rounded-r2 bg-chip", busy && "animate-pulse")} />
            <span className={cn("h-1.5 w-4/5 rounded-r2 bg-chip", busy && "animate-pulse")} />
            <span className={cn("h-1.5 w-3/5 rounded-r2 bg-chip", busy && "animate-pulse")} />
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * Générateur de CV (`screens/15-generator-resume.png`) : surcouche plein écran, offre visée à
 * gauche, feuille A4 au centre, étapes puis décisions ATS à droite. Une fois le CV généré,
 * la feuille devient éditable directement (`DECISIONS` D2).
 */
export function ResumeGeneratorPage() {
  const location = useLocation();
  // Mémoïsé : `generationFromNavigation` recrée un objet à chaque appel, et une dépendance
  // d'effet sur cette référence instable relancerait la préparation en boucle.
  const initiale = useMemo(() => generationFromNavigation(location.state), [location.state]);
  const vm = useResumeGeneratorViewModel(initiale);
  const running = vm.operation !== null;
  const steps = useStepLog(CV_STEPS, vm.progress?.step ?? null, running, vm.elapsedMs);

  const offerPane = (
    <>
      <PaneSection title="Offre visée">
        <OfferSource value={vm.jobOffer} onChange={vm.setJobOffer} readClipboard={vm.readClipboard} disabled={running} />
      </PaneSection>
      {vm.error ? <ErrorBanner title="Génération impossible" message={vm.error} /> : null}
    </>
  );

  const generateButton = running ? (
    <AiStopButton stopping={vm.stopping} onStop={() => void vm.stop()} />
  ) : (
    <Button variant="primary" size="bar" shortcut="mod+enter" onClick={() => void vm.generate()}>
      {vm.workspace ? "Générer un nouveau CV" : "Générer"}
    </Button>
  );

  const stepsPane = (
    <PaneSection title={running ? "Étapes" : vm.metrics ? "Terminé" : "Étapes"}>
      <StepList steps={steps} />
      {!running && !vm.workspace ? (
        <p className="mt-5 text-small leading-[1.55] text-tx-4">
          Candilog lit l'offre, choisit les expériences les plus proches et rédige. Vous relisez tout avant
          d'enregistrer — rien n'est envoyé à l'entreprise.
        </p>
      ) : null}
    </PaneSection>
  );

  if (vm.workspace) {
    return (
      <ResumeEditorScreen
        key={vm.generationIndex}
        initial={vm.workspace}
        name={vm.name}
        onNameChange={vm.setName}
        onSave={vm.saveResume}
        isSaving={vm.isSaving}
        onExportPdf={vm.exportPdf}
        left={vm.briefOpen ? offerPane : null}
        onReopenBrief={vm.openBrief}
        onCloseBrief={vm.closeBrief}
        generateButton={vm.briefOpen ? generateButton : null}
        onGenerate={vm.briefOpen ? () => void vm.generate() : undefined}
        stepsPane={stepsPane}
        running={running}
        metrics={vm.operation === null ? vm.metrics : null}
      />
    );
  }

  return (
    <GeneratorFrame
      title="Générer un CV"
      actions={generateButton}
      left={offerPane}
      right={stepsPane}
      status={
        running
          ? `${vm.progress?.step ?? "Préparation"} · ${formatElapsed(vm.elapsedMs)}`
          : vm.jobOffer.trim()
            ? "prêt · l'offre sera lue par le modèle de la tâche « Générer un CV ciblé »"
            : "collez une offre ou choisissez une candidature"
      }
      keys={running ? [] : [{ label: "Générer", shortcut: "mod+enter" }]}
      closeDisabled={running}
      {...(running ? {} : { onSubmit: () => void vm.generate() })}
    >
      {running ? (
        <div className="px-[26px] pt-5">
          <AiProgress progress={vm.progress} elapsedMs={vm.elapsedMs} />
        </div>
      ) : null}
      <EmptySheet busy={running} />
    </GeneratorFrame>
  );
}

/**
 * Le CV une fois généré : feuille A4 éditable au centre, décisions ATS à droite ; l'offre
 * revient à gauche quand on la rouvre pour régénérer. L'éditeur vit ici, jamais dans le
 * composant parent, pour n'être initialisé qu'une fois le workspace connu — un remontage
 * (`key`) sur une nouvelle génération lui donne un historique neuf.
 */
function ResumeEditorScreen({
  initial,
  name,
  onNameChange,
  onSave,
  isSaving,
  onExportPdf,
  left,
  onReopenBrief,
  onCloseBrief,
  generateButton,
  onGenerate,
  stepsPane,
  running,
  metrics,
}: {
  initial: ResumeWorkspace;
  name: string;
  onNameChange: (value: string) => void;
  onSave: (workspace: ResumeWorkspace) => Promise<unknown>;
  isSaving: boolean;
  onExportPdf: (document: ResumeWorkspace["document"]) => Promise<void>;
  left: ReactNode | null;
  onReopenBrief: () => void;
  onCloseBrief: () => void;
  generateButton: ReactNode | null;
  onGenerate: (() => void) | undefined;
  stepsPane: ReactNode;
  running: boolean;
  metrics: { elapsed_ms: number; tokens_used: number | null } | null;
}) {
  const notify = useUiStore((s) => s.notify);
  const editor = useResumeEditor(initial);
  // La photo suit le profil courant, exactement comme à l'export PDF.
  const photo = useProfilePhoto().data ?? null;
  const [overflow, setOverflow] = useState(false);
  const busy = editor.isProofreading || editor.isRecalculating;
  const canSave = Boolean(name.trim()) && !isSaving && !busy;
  const save = () => {
    if (canSave) void onSave(editor.workspace);
  };
  const correctFrench = async () => {
    const result = await editor.correctFrench();
    notify(result === "failed" ? {
      tone: "error",
      title: "Correction impossible",
      detail: "Vérifiez la configuration de votre fournisseur IA puis réessayez.",
    } : {
      tone: "success",
      title: result === "corrected" ? "Orthographe corrigée" : "Aucune correction nécessaire",
      detail: result === "corrected"
        ? "Le sens et les faits du CV ont été conservés."
        : "Le contenu actuel a été relu sans modification.",
    });
  };

  const score = Math.round(editor.workspace.score.total);
  const leftPane = left ?? (
    <>
      <PaneSection title="Offre visée">
        <p className="text-ui font-medium text-tx-2">{initial.job_offer.title || "Offre sans intitulé"}</p>
        {initial.job_offer.location ? <p className="text-sub text-tx-5">{initial.job_offer.location}</p> : null}
        <Button size="compact" className="mt-2.5" onClick={onReopenBrief}>
          Modifier l’offre
        </Button>
      </PaneSection>
      <PaneSection title="Nom de la version">
        <FormField label="Nom de la version" required>
          {(props) => <TextInput {...props} value={name} onChange={(e) => onNameChange(e.target.value)} />}
        </FormField>
      </PaneSection>
    </>
  );

  return (
    <GeneratorFrame
      title="Générer un CV"
      actions={
        <>
          {generateButton}
          {left ? (
            <Button variant="ghost" size="bar" onClick={onCloseBrief}>
              Revenir au CV
            </Button>
          ) : null}
          <Button size="bar" disabled={overflow || busy} onClick={() => void onExportPdf(editor.workspace.document)}>
            Exporter en PDF
          </Button>
          <Button variant="primary" size="bar" shortcut="mod+s" disabled={!canSave} onClick={save}>
            {isSaving ? "Enregistrement…" : "Enregistrer"}
          </Button>
        </>
      }
      left={leftPane}
      right={
        <>
          {running || metrics ? stepsPane : null}
          <ResumeAtsPanel
            workspace={editor.workspace}
            busy={busy}
            onAddProfileItem={editor.addProfileItem}
            onApplyRecommendation={editor.applyContentRecommendation}
            onIgnoreRecommendation={editor.ignoreContentRecommendation}
            onAccept={(id) => void editor.applyProposal(id)}
            onReject={(id) => void editor.rejectProposal(id)}
            onUndo={(id) => void editor.undoProposal(id)}
            lastScoreImpact={editor.lastScoreImpact}
          />
        </>
      }
      status={[
        metrics ? `terminé en ${formatElapsed(metrics.elapsed_ms)}` : null,
        metrics && metrics.tokens_used !== null ? `${formatTokens(metrics.tokens_used)} tokens` : null,
        `score ${score}/100`,
        overflow ? "contenu trop long pour une page" : null,
      ]
        .filter(Boolean)
        .join(" · ")}
      keys={[{ label: "Enregistrer", shortcut: "mod+s" }]}
      closeDisabled={running}
      onSave={save}
      {...(onGenerate ? { onSubmit: onGenerate } : {})}
    >
      <div className="flex flex-none items-center gap-2 border-b border-bd-soft bg-app px-4 py-1.5">
        <UndoRedoControls canUndo={editor.canUndo} canRedo={editor.canRedo} onUndo={editor.undo} onRedo={editor.redo} />
        {overflow ? <OverflowStatus overflow /> : null}
        <Button size="compact" className="ml-auto" disabled={busy} onClick={() => void correctFrench()}>
          {editor.isProofreading ? "Correction en cours…" : "Corriger l'orthographe"}
        </Button>
      </div>
      <div className="flex min-h-0 flex-1 flex-col items-center gap-3 overflow-auto p-[26px]">
        <ResumePaper
          workspace={editor.workspace}
          editable={!editor.isProofreading}
          onChange={editor.updateField}
          onRemoveSkill={editor.removeSkill}
          onRemoveSection={editor.removeSection}
          onOverflowChange={setOverflow}
          photo={photo}
        />
      </div>
      <ProfileSkillChoiceDialog
        pending={editor.pendingProfileSkill}
        error={editor.error}
        onKeepResumeOnly={editor.keepSkillInResumeOnly}
        onAddToProfile={editor.addPendingSkillToProfile}
      />
    </GeneratorFrame>
  );
}
