import { useMemo, useState, type ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { formatDuration } from "@/shared/lib/duration";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { Button, EmptyState, ErrorBanner, FormField, PageHeader, TextInput } from "@/shared/ui";
import { useResumeEditor } from "../../viewmodel/useResumeEditor";
import { useResumeGeneratorViewModel } from "../../viewmodel/useResumeGeneratorViewModel";
import { A4Preview, AiProgress, DocumentPanel, OverflowStatus, UndoRedoControls } from "../components/DocumentUi";
import { ProfileSkillChoiceDialog } from "../components/ProfileSkillChoiceDialog";
import { ResumeAtsPanel } from "../components/ResumeAtsPanel";
import { ResumePaper } from "../components/ResumePaper";
import { ChampOffre, HeaderBadge, Screen, exportPdf, generationFromNavigation } from "./documentPageSupport";

export function ResumeGeneratorPage() {
  const location = useLocation();
  // Mémoïsé : `generationFromNavigation` recrée un objet à chaque appel, et une dépendance
  // d'effet sur cette référence instable relancerait la préparation en boucle.
  const initiale = useMemo(() => generationFromNavigation(location.state), [location.state]);
  const vm = useResumeGeneratorViewModel(initiale);

  const briefPanel = (
    <DocumentPanel title="Offre ciblée" icon="target" className="flex min-h-0 flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
        <ChampOffre
          label="Texte de l’offre"
          required
          help="Le texte est envoyé uniquement au fournisseur configuré."
          rows={18}
          value={vm.jobOffer}
          placeholder="Collez ici l’intitulé, les missions et les compétences recherchées…"
          onChange={vm.setJobOffer}
        />
        {vm.error ? <ErrorBanner title="Génération impossible" message={vm.error} /> : null}
        {vm.operation ? (
          <><AiProgress progress={vm.progress} elapsedMs={vm.elapsedMs} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void vm.cancel()}>Annuler</Button></>
        ) : (
          <div className="flex flex-col gap-2">
            <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void vm.generate()}>{vm.workspace ? "Générer un nouveau CV" : "Générer le CV ciblé"}</Button>
            {vm.workspace ? (
              <Button variant="ghost" icon="article" className="w-full" onClick={vm.closeBrief}>Revenir au CV</Button>
            ) : null}
          </div>
        )}
      </div>
    </DocumentPanel>
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
        briefPanel={vm.briefOpen ? briefPanel : null}
        onReopenBrief={vm.openBrief}
        durationBadge={vm.operation === null && vm.durationMs !== null ? <HeaderBadge icon="schedule">Généré en {formatDuration(vm.durationMs)}</HeaderBadge> : undefined}
      />
    );
  }

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={vm.operation === null && vm.durationMs !== null ? <HeaderBadge icon="schedule">Généré en {formatDuration(vm.durationMs)}</HeaderBadge> : undefined}
      />
    }>
      <div className="grid min-h-[660px] gap-4 xl:grid-cols-[350px_minmax(460px,1fr)_320px]">
        {briefPanel}
        <DocumentPanel title="Aperçu HTML · A4" icon="article"><A4Preview /></DocumentPanel>
        <DocumentPanel title="Analyse ATS" icon="query_stats">
          <div className="space-y-5 p-4">
            <EmptyState
              icon="query_stats"
              title="Analyse en attente"
              description="Le score et les recommandations suivront la génération."
            />
          </div>
        </DocumentPanel>
      </div>
    </Screen>
  );
}

/**
 * Le CV une fois généré : offre à gauche (inchangée), papier A4 éditable au centre, décisions
 * ATS à droite. L'éditeur vit ici, jamais dans le composant parent, pour n'être initialisé
 * qu'une fois le workspace connu — un remontage (`key`) sur une nouvelle génération lui donne
 * un historique neuf plutôt que de réutiliser celui d'une session précédente.
 */
function ResumeEditorScreen({
  initial,
  name,
  onNameChange,
  onSave,
  isSaving,
  briefPanel,
  onReopenBrief,
  durationBadge,
}: {
  initial: ResumeWorkspace;
  name: string;
  onNameChange: (value: string) => void;
  onSave: (workspace: ResumeWorkspace) => Promise<unknown>;
  isSaving: boolean;
  briefPanel: ReactNode | null;
  onReopenBrief: () => void;
  durationBadge?: ReactNode;
}) {
  const notify = useUiStore((s) => s.notify);
  const editor = useResumeEditor(initial);
  const [overflow, setOverflow] = useState(false);

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={<>{durationBadge}<OverflowStatus overflow={overflow} /></>}
        secondary={
          <>
            {briefPanel ? null : <Button icon="target" onClick={onReopenBrief}>Modifier l’offre</Button>}
            <UndoRedoControls canUndo={editor.canUndo} canRedo={editor.canRedo} onUndo={editor.undo} onRedo={editor.redo} />
            <Button icon="save" disabled={!name.trim() || isSaving} onClick={() => void onSave(editor.workspace)}>Enregistrer</Button>
          </>
        }
        primary={
          <Button
            variant="primary"
            icon="download"
            disabled={overflow}
            onClick={() => void exportPdf(editor.workspace.document, notify)}
          >
            Exporter le PDF
          </Button>
        }
      />
    } padded={false}>
      <div className={`grid min-h-0 flex-1 gap-4 overflow-hidden p-5 min-[1200px]:p-6 ${briefPanel ? "xl:grid-cols-[350px_minmax(460px,1fr)_320px]" : "xl:grid-cols-[minmax(460px,1fr)_320px]"}`}>
        {briefPanel}
        <DocumentPanel title="Aperçu HTML · A4" icon="article" className="flex min-h-0 flex-col">
          <div className="flex min-h-0 flex-1 flex-col items-center gap-3 overflow-auto bg-page p-[26px]">
            <ResumePaper workspace={editor.workspace} editable onChange={editor.updateField} onOverflowChange={setOverflow} />
          </div>
        </DocumentPanel>
        <DocumentPanel title="Analyse ATS" icon="query_stats" className="flex min-h-0 flex-col overflow-y-auto">
          <ResumeAtsPanel
            workspace={editor.workspace}
            busy={editor.isRecalculating}
            onAccept={(id) => void editor.applyProposal(id)}
            onReject={(id) => void editor.rejectProposal(id)}
            onUndo={(id) => void editor.undoProposal(id)}
          />
          <div className="px-4 pb-4">
            <FormField label="Nom de la version" required>
              {(props) => <TextInput {...props} value={name} onChange={(e) => onNameChange(e.target.value)} />}
            </FormField>
          </div>
        </DocumentPanel>
      </div>
      <ProfileSkillChoiceDialog
        pending={editor.pendingProfileSkill}
        error={editor.error}
        onKeepResumeOnly={editor.keepSkillInResumeOnly}
        onAddToProfile={editor.addPendingSkillToProfile}
      />
    </Screen>
  );
}
