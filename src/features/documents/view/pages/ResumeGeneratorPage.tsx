import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useLocation } from "react-router-dom";
import { documentsService } from "../../services/documentsService";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import type { ResumeGeneration } from "@/features/ai/model/types";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { Button, EmptyState, ErrorBanner, FormField, Icon, PageHeader, TextArea, TextInput } from "@/shared/ui";
import { A4Preview, AiProgress, DocumentPanel, ScoreBadge } from "../components/DocumentUi";
import { HeaderBadge, RESUME_KEY, Screen, TexteNonVerifie, detail, exportPdf, generationFromNavigation, message } from "./documentPageSupport";

export function ResumeGeneratorPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const location = useLocation();
  const initiale = generationFromNavigation(location.state);
  const [job_offer, setJobOffer] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [result, setResult] = useState<ResumeGeneration | null>(initiale.result);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState(initiale.name);
  const progress = useAiProgress(operation);
  useCancelAiOnUnmount(operation);

  const run = async () => {
    if (!job_offer.trim()) { setError("Collez le texte de l’offre à cibler."); return; }
    const id = generation_id();
    setOperation(id);
    setError(null);
    try {
      const value = await aiService.generateResume({ generation_id: id, job_offer });
      setResult(value);
      setName(`CV — ${value.job_offer.title || "Version ciblée"}`);
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };
  const save = useMutation({
    mutationFn: () => documentsService.saveResume({ name, content: result }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "CV ajouté à la bibliothèque" });
    },
    // Sans ce gestionnaire, un refus du service Rust laissait l'écran inchangé : le CV
    // généré n'était pas enregistré et rien ne le disait.
    onError: (error) =>
      notify({ tone: "error", title: "Enregistrement impossible", detail: detail(error) }),
  });

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={operation ? <HeaderBadge>IA active</HeaderBadge> : undefined}
        secondary={result ? <Button icon="save" disabled={!name.trim() || save.isPending} onClick={() => save.mutate()}>Enregistrer</Button> : undefined}
        primary={result ? <Button variant="primary" icon="download" onClick={() => void exportPdf(result.resume, notify)}>Exporter le PDF</Button> : undefined}
      />
    }>
      <div className="grid min-h-[660px] gap-4 xl:grid-cols-[350px_minmax(460px,1fr)_320px]">
        <DocumentPanel title="Offre ciblée" icon="target">
          <div className="space-y-4 p-4">
            <FormField label="Texte de l’offre" required help="Le texte est envoyé uniquement au fournisseur configuré.">
              {(props) => <TextArea {...props} rows={18} value={job_offer} placeholder="Collez ici l’intitulé, les missions et les compétences recherchées…" onChange={(e) => setJobOffer(e.target.value)} />}
            </FormField>
            {error ? <ErrorBanner title="Génération impossible" message={error} /> : null}
            {operation ? (
              <><AiProgress progress={progress} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void aiService.cancel(operation)}>Annuler</Button></>
            ) : (
              <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>Générer le CV ciblé</Button>
            )}
          </div>
        </DocumentPanel>
        <DocumentPanel title="Aperçu HTML · A4" icon="article"><A4Preview resume={result?.resume} /></DocumentPanel>
        <DocumentPanel title="Analyse ATS" icon="query_stats">
          <div className="space-y-5 p-4">
            {result ? (
              <>
                <ScoreBadge value={result.profile_score.total} />
                <p className="text-body leading-relaxed text-ink-muted">{result.analysis.recap}</p>
                <TexteNonVerifie />
                <div>
                  <p className="mb-2 text-label font-medium text-ink">Suggestions</p>
                  <ul className="space-y-2">{result.analysis.suggestions.map((s, i) => <li key={i} className="flex gap-2 text-body text-ink-muted"><Icon name="arrow_right" size={15} className="mt-0.5 text-accent" />{s}</li>)}</ul>
                </div>
                <FormField label="Nom de la version" required>{(props) => <TextInput {...props} value={name} onChange={(e) => setName(e.target.value)} />}</FormField>
              </>
            ) : (
              <EmptyState icon="query_stats" title="Analyse en attente" description="Le score et les recommandations suivront la génération." />
            )}
          </div>
        </DocumentPanel>
      </div>
    </Screen>
  );
}
