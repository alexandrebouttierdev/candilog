import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useLocation } from "react-router-dom";
import { documentsService } from "../../services/documentsService";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { useAiTimer } from "@/features/ai/viewmodel/useAiTimer";
import { formatDuration } from "@/shared/lib/duration";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { Button, EmptyState, ErrorBanner, FormField, PageHeader, TextInput } from "@/shared/ui";
import { useResumeEditor } from "../../viewmodel/useResumeEditor";
import { A4Preview, AiProgress, DocumentPanel, OverflowStatus, UndoRedoControls } from "../components/DocumentUi";
import { ProfileSkillChoiceDialog } from "../components/ProfileSkillChoiceDialog";
import { ResumeAtsPanel } from "../components/ResumeAtsPanel";
import { ResumePaper } from "../components/ResumePaper";
import { ChampOffre, HeaderBadge, RESUME_KEY, Screen, detail, exportPdf, generationFromNavigation, message } from "./documentPageSupport";

export function ResumeGeneratorPage() {
  const location = useLocation();
  // Mémoïsé : `generationFromNavigation` recrée un objet à chaque appel, et une dépendance
  // d'effet sur cette référence instable relancerait la préparation en boucle.
  const initiale = useMemo(() => generationFromNavigation(location.state), [location.state]);
  const [job_offer, setJobOffer] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState(initiale.name);
  const [workspace, setWorkspace] = useState<ResumeWorkspace | null>(initiale.workspace);
  // Change à chaque nouvelle génération pour remonter l'éditeur (nouvel historique, nouveau
  // brouillon) plutôt que de réutiliser l'état local d'une session d'édition précédente.
  const [generationIndex, setGenerationIndex] = useState(0);
  const progress = useAiProgress(operation);
  useCancelAiOnUnmount(operation);
  const timer = useAiTimer(operation !== null);

  // Une réouverture depuis la bibliothèque transmet encore une ancienne génération plutôt
  // qu'un workspace : elle est préparée une fois ici, avec le même traitement d'échec
  // qu'une génération impossible.
  useEffect(() => {
    if (workspace !== null || initiale.result === null) return;
    let annule = false;
    void documentsService
      .prepareResume(initiale.result)
      .then((prepared) => {
        if (annule) return;
        setWorkspace(prepared);
        setGenerationIndex((index) => index + 1);
      })
      .catch((e) => {
        if (!annule) setError(message(e));
      });
    return () => {
      annule = true;
    };
  }, [initiale.result, workspace]);

  const run = async () => {
    if (!job_offer.trim()) { setError("Collez le texte de l’offre à cibler."); return; }
    const id = generation_id();
    setOperation(id);
    setError(null);
    timer.start();
    try {
      const generation = await aiService.generateResume({ generation_id: id, job_offer });
      const prepared = await documentsService.prepareResume(generation);
      timer.stop();
      setWorkspace(prepared);
      setGenerationIndex((index) => index + 1);
      setName(`CV — ${prepared.job_offer.title || "Version ciblée"}`);
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };

  const briefPanel = (
    <DocumentPanel title="Offre ciblée" icon="target">
      <div className="space-y-4 p-4">
        <ChampOffre
          label="Texte de l’offre"
          required
          help="Le texte est envoyé uniquement au fournisseur configuré."
          rows={18}
          value={job_offer}
          placeholder="Collez ici l’intitulé, les missions et les compétences recherchées…"
          onChange={setJobOffer}
        />
        {error ? <ErrorBanner title="Génération impossible" message={error} /> : null}
        {operation ? (
          <><AiProgress progress={progress} elapsedMs={timer.elapsedMs} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void aiService.cancel(operation)}>Annuler</Button></>
        ) : (
          <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>Générer le CV ciblé</Button>
        )}
      </div>
    </DocumentPanel>
  );

  if (workspace) {
    return (
      <ResumeEditorScreen
        key={generationIndex}
        initial={workspace}
        name={name}
        onNameChange={setName}
        briefPanel={briefPanel}
        durationBadge={operation === null && timer.durationMs !== null ? <HeaderBadge icon="schedule">Généré en {formatDuration(timer.durationMs)}</HeaderBadge> : undefined}
      />
    );
  }

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={operation === null && timer.durationMs !== null ? <HeaderBadge icon="schedule">Généré en {formatDuration(timer.durationMs)}</HeaderBadge> : undefined}
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
  briefPanel,
  durationBadge,
}: {
  initial: ResumeWorkspace;
  name: string;
  onNameChange: (value: string) => void;
  briefPanel: ReactNode;
  durationBadge?: ReactNode;
}) {
  const notify = useUiStore((s) => s.notify);
  const queryClient = useQueryClient();
  const editor = useResumeEditor(initial);
  const [overflow, setOverflow] = useState(false);

  const save = useMutation({
    mutationFn: () => documentsService.saveResume({ name, content: editor.workspace }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "CV ajouté à la bibliothèque" });
    },
    // Sans ce gestionnaire, un refus du service Rust laissait l'écran inchangé : le CV édité
    // n'était pas enregistré et rien ne le disait. L'éditeur et son historique restent
    // affichés tels quels, rien n'est perdu par cet échec.
    onError: (error) =>
      notify({ tone: "error", title: "Enregistrement impossible", detail: detail(error) }),
  });

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={<>{durationBadge}<OverflowStatus overflow={overflow} /></>}
        secondary={
          <>
            <UndoRedoControls canUndo={editor.canUndo} canRedo={editor.canRedo} onUndo={editor.undo} onRedo={editor.redo} />
            <Button icon="save" disabled={!name.trim() || save.isPending} onClick={() => save.mutate()}>Enregistrer</Button>
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
    }>
      <div className="grid min-h-[660px] gap-4 xl:grid-cols-[350px_minmax(460px,1fr)_320px]">
        {briefPanel}
        <DocumentPanel title="Aperçu HTML · A4" icon="article">
          <div className="flex min-h-0 flex-1 justify-center overflow-auto bg-page p-[26px]">
            <ResumePaper workspace={editor.workspace} editable onChange={editor.updateField} onOverflowChange={setOverflow} />
          </div>
        </DocumentPanel>
        <DocumentPanel title="Analyse ATS" icon="query_stats">
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
