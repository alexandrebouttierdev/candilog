import { useMemo, useState, type ReactNode } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "react-router-dom";
import { documentsService } from "../../services/documentsService";
import type { ResumeVersion, CoverLetterExport, CoverLetter } from "../../services/documentsService";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import type { ImportedResumeAnalysis, GeneratedResume, ResumeGeneration } from "@/features/ai/model/types";
import { useAiProgress } from "@/features/ai/viewmodel/useAiProgress";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { ContextBarAccessory, ContextNote, ContextSearch } from "@/app/layout/ContextBar";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, FormField, Icon, PageHeader, Select, TextArea, TextInput } from "@/shared/ui";
import { A4Preview, DocumentPanel, AiProgress, PreviewAction, ScoreBadge } from "../components/DocumentUi";

const RESUME_KEY = ["documents", "cv"] as const;
const COVER_LETTERS_KEY = ["documents", "lettres"] as const;

export function ResumeLibraryPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const list = useQuery({ queryKey: RESUME_KEY, queryFn: documentsService.listResume });
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [recherche, setRecherche] = useState("");
  const selected_id = selected ?? list.data?.[0]?.id ?? null;
  const detail = useQuery({
    queryKey: [...RESUME_KEY, selected_id],
    queryFn: () => documentsService.getResume(selected_id ?? ""),
    enabled: selected_id !== null,
  });
  const remove = useMutation({
    mutationFn: documentsService.deleteResume,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "Version supprimée" });
    },
  });
  const dupliquer = useMutation({
    mutationFn: async () => {
      if (!detail.data) return;
      await documentsService.saveResume({
        name: `${detail.data.name} (copie)`,
        content: detail.data.content,
      });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "Version dupliquée" });
    },
  });
  const generation = detail.data && isGeneration(detail.data.content) ? detail.data.content : null;
  const version = detail.data;
  const versions = useMemo(() => {
    const terme = recherche.trim().toLowerCase();
    const toutes = list.data ?? [];
    return terme ? toutes.filter((resume) => resume.name.toLowerCase().includes(terme)) : toutes;
  }, [list.data, recherche]);

  return (
    <Screen
      padded={false}
      search={{ value: recherche, onChange: setRecherche, placeholder: "Rechercher un document…" }}
      header={
      <PageHeader
        icon="description"
        title="Mes CV"
        subtitle="Vos versions prêtes à l’emploi"
        badge={list.data ? <HeaderBadge>{list.data.length} version{list.data.length > 1 ? "s" : ""}</HeaderBadge> : undefined}
        secondary={<Button icon="upload_file" onClick={() => void navigate("/documents/analyze")}>Importer</Button>}
        primary={<Button variant="primary" icon="auto_awesome" onClick={() => void navigate("/documents/generate-resume")}>Nouveau CV</Button>}
      />
    }>
      <div className="flex min-h-0 flex-1">
          <div className="flex w-[40%] min-w-[280px] flex-col border-r border-line bg-surface">
            <div className="border-b border-line px-5 pt-4 pb-3">
              <div className="mb-[11px] flex items-center justify-between">
                <span className="text-section">Bibliothèque</span>
                <span className="text-label text-ink-faint">{list.data?.length ?? 0} version{(list.data?.length ?? 0) > 1 ? "s" : ""}</span>
              </div>
              <label className="flex h-8 items-center gap-2 rounded-button border border-line bg-page px-2.5">
                <Icon name="search" size={16} className="text-ink-faint" />
                <input
                  type="search"
                  value={recherche}
                  onChange={(e) => setRecherche(e.target.value)}
                  placeholder="Rechercher une version…"
                  className="min-w-0 flex-1 bg-transparent text-body text-ink outline-none placeholder:text-ink-faint"
                />
              </label>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto p-2.5">
              {list.error ? (
                <ErrorBanner message={message(list.error)} onRetry={() => void list.refetch()} />
              ) : list.isLoading ? (
                <p className="p-6 text-center text-ink-muted">Chargement…</p>
              ) : versions.length ? (
                <ul className="space-y-1.5">
                  {versions.map((resume) => (
                    <li key={resume.id}>
                      <button
                        type="button"
                        aria-pressed={selected_id === resume.id}
                        onClick={() => setSelected(resume.id)}
                        className={`flex w-full gap-3 rounded-[10px] border p-3 text-left transition-colors ${selected_id === resume.id ? "border-accent-border bg-accent-tint" : "border-transparent hover:bg-neutral-tint"}`}
                      >
                        <span className="flex h-[50px] w-[38px] flex-none flex-col gap-[3px] rounded-[5px] border border-line bg-page px-[5px] py-1.5">
                          <span className={`h-[3px] w-[70%] rounded-sm ${selected_id === resume.id ? "bg-accent" : "bg-accent/40"}`} />
                          <span className="h-[2px] w-full rounded-sm bg-line" />
                          <span className="h-[2px] w-[85%] rounded-sm bg-line" />
                          <span className="h-[2px] w-[95%] rounded-sm bg-line" />
                          <span className="h-[2px] w-[60%] rounded-sm bg-line" />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="mb-[3px] flex items-center gap-2">
                            <span className="truncate text-item font-semibold">{resume.name}</span>
                            {selected_id === resume.id && generation ? (
                              <AtsChip score={generation.analysis.score} />
                            ) : null}
                          </span>
                          <span className="block text-label text-ink-faint">{date(resume.created_at)}</span>
                        </span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : recherche.trim() ? (
                <EmptyState icon="search" title="Aucun résultat" description="Aucune version ne correspond à cette recherche." />
              ) : (
                <EmptyState icon="description" title="Aucune version" description="Générez puis sauvegardez votre premier CV ciblé." action={<Button icon="auto_awesome" onClick={() => void navigate("/documents/generate-resume")}>GNRer un Resume</Button>} />
              )}
            </div>
          </div>
          <div className="flex min-w-0 flex-1 flex-col bg-page">
            <div className="flex flex-none items-center justify-between gap-3 border-b border-line bg-surface px-[22px] py-3">
              <div className="flex min-w-0 items-center gap-[9px]">
                <Icon name="visibility" size={17} className="text-ink-faint" />
                <p className="truncate text-body font-mid">
                  {version ? `Aperçu · ${version.name}` : "Aperçu"}
                </p>
              </div>
              {version ? (
                <div className="flex items-center gap-1.5">
                  {generation ? (
                    <>
                      <PreviewAction
                        icon="edit"
                        onClick={() =>
                          void navigate("/documents/generate-resume", {
                            state: { generation, name: version.name },
                          })
                        }
                      >
                        Update
                      </PreviewAction>
                      <PreviewAction
                        icon="content_copy"
                        disabled={dupliquer.isPending}
                        onClick={() => dupliquer.mutate()}
                      >
                        Dupliquer
                      </PreviewAction>
                      <PreviewAction
                        icon="download"
                        onClick={() => void exportPdf(generation.resume, version.name, notify)}
                      >
                        Export PDF
                      </PreviewAction>
                    </>
                  ) : null}
                  <PreviewAction tone="danger" icon="delete" onClick={() => setDeleteId(version.id)}>
                    Delete
                  </PreviewAction>
                </div>
              ) : null}
            </div>
            <div className="min-h-0 flex-1 overflow-auto">
              {detail.data ? <ResumeSavedPreview version={detail.data} /> : <EmptyState icon="visibility" title="Sélectionnez une version" description="Son contenu détaillé apparaîtra ici." />}
            </div>
          </div>
        </div>
      <ConfirmDialog open={deleteId !== null} title="Supprimer cette version ?" description="Le CV disparaîtra définitivement de la bibliothèque locale." note="Votre profil et vos autres versions seront conservés." busy={remove.isPending} onCancel={() => setDeleteId(null)} onConfirm={() => { if (deleteId) remove.mutate(deleteId); }} />
    </Screen>
  );
}

function ResumeSavedPreview({ version }: { version: ResumeVersion }) { const generation = isGeneration(version.content) ? version.content : null; return generation ? <A4Preview resume={generation.resume} /> : <A4Preview title={version.name}><div className="flex min-h-[590px] items-center justify-center text-center text-[#7b8493]">Cette ancienne version ne contient pas encore d’aperçu structuré compatible.</div></A4Preview>; }

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
  });

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={operation ? <HeaderBadge>Ai active</HeaderBadge> : undefined}
        secondary={result ? <Button icon="save" disabled={!name.trim() || save.isPending} onClick={() => save.mutate()}>Enregistrer</Button> : undefined}
        primary={result ? <Button variant="primary" icon="download" onClick={() => void exportPdf(result.resume, name || "cv-candilog", notify)}>Exporter le PDF</Button> : undefined}
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
              <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>GNRer le Resume ciblé</Button>
            )}
          </div>
        </DocumentPanel>
        <DocumentPanel title="Aperçu HTML · A4" icon="article"><A4Preview resume={result?.resume} /></DocumentPanel>
        <DocumentPanel title="Analyse ATS" icon="query_stats">
          <div className="space-y-5 p-4">
            {result ? (
              <>
                <ScoreBadge value={result.analysis.score} />
                <p className="text-body leading-relaxed text-ink-muted">{result.analysis.recap}</p>
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

export function LettersLibraryPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [recherche, setRecherche] = useState("");
  const list = useQuery({ queryKey: COVER_LETTERS_KEY, queryFn: documentsService.listCoverLetters });
  const selected_id = selected ?? list.data?.[0]?.id ?? null;
  const selected_letter = list.data?.find((l) => l.id === selected_id) ?? null;
  const cover_letters = useMemo(() => {
    const terme = recherche.trim().toLowerCase();
    const toutes = list.data ?? [];
    return terme
      ? toutes.filter(
          (cover_letter) =>
            cover_letter.name.toLowerCase().includes(terme) ||
            (cover_letter.company ?? "").toLowerCase().includes(terme) ||
            (cover_letter.job_title ?? "").toLowerCase().includes(terme),
        )
      : toutes;
  }, [list.data, recherche]);
  const remove = useMutation({
    mutationFn: documentsService.deleteCoverLetter,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: COVER_LETTERS_KEY });
    },
  });

  const copier = async () => {
    if (!selected_letter) return;
    await navigator.clipboard.writeText(selected_letter.content);
    notify({ tone: "success", title: "Lettre copiée" });
  };

  return (
    <Screen
      padded={false}
      search={{ value: recherche, onChange: setRecherche, placeholder: "Rechercher un document…" }}
      header={
      <PageHeader
        icon="mail"
        title="Mes lettres de motivation"
        subtitle="Bibliothèque"
        badge={list.data ? <HeaderBadge>{list.data.length} lettre{list.data.length > 1 ? "s" : ""}</HeaderBadge> : undefined}
        primary={<Button variant="primary" icon="auto_awesome" onClick={() => void navigate("/documents/write-cover-letter")}>Rédiger une lettre</Button>}
      />
    }>
      <div className="flex min-h-0 flex-1">
          <div className="flex w-[36%] min-w-[260px] flex-col border-r border-line bg-surface">
            <div className="flex items-center justify-between border-b border-line px-5 py-3">
              <span className="text-section">Bibliothèque</span>
              <span className="text-label text-ink-faint">{list.data?.length ?? 0} lettre{(list.data?.length ?? 0) > 1 ? "s" : ""}</span>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto p-2.5">
              {list.error ? (
                <ErrorBanner message={message(list.error)} onRetry={() => void list.refetch()} />
              ) : list.isLoading ? (
                <p className="p-6 text-center text-ink-muted">Chargement…</p>
              ) : cover_letters.length ? (
                <ul className="space-y-1.5">
                  {cover_letters.map((letter) => (
                    <li key={letter.id}>
                      <button
                        type="button"
                        aria-pressed={selected_id === letter.id}
                        onClick={() => setSelected(letter.id)}
                        className={`w-full rounded-[10px] border px-3.5 py-3 text-left transition-colors ${selected_id === letter.id ? "border-accent-border bg-accent-tint" : "border-transparent hover:bg-neutral-tint"}`}
                      >
                        <span className="flex items-center gap-2">
                          <Icon name="mail" size={16} className={selected_id === letter.id ? "text-accent" : "text-ink-faint"} />
                          <span className="min-w-0 flex-1 truncate text-[13px] font-semibold">{letter.company ?? letter.name}</span>
                          <span className="flex-none text-label text-ink-faint">{date(letter.created_at)}</span>
                        </span>
                        <span className="mt-1 block truncate pl-6 text-label text-ink-faint">{letter.job_title ?? "Application"} · {labelTone(letter.tone)}</span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : recherche.trim() ? (
                <EmptyState icon="search" title="Aucun résultat" description="Aucune lettre ne correspond à cette recherche." />
              ) : (
                <EmptyState icon="mail" title="Aucune lettre enregistrée" description="Rédigez une lettre puis enregistrez-la ici." action={<Button icon="auto_awesome" onClick={() => void navigate("/documents/write-cover-letter")}>Rédiger une lettre</Button>} />
              )}
            </div>
          </div>
          <div className="flex min-w-0 flex-1 flex-col bg-page">
            <div className="flex flex-none items-center justify-between gap-3 border-b border-line bg-surface px-[22px] py-3">
              <p className="truncate text-body font-mid">{selected_letter?.name ?? "Lecture"}</p>
              {selected_letter ? (
                <div className="flex items-center gap-1.5">
                  <PreviewAction icon="edit" onClick={() => void navigate("/documents/write-cover-letter", { state: { cover_letter: selected_letter } })}>Modifier</PreviewAction>
                  <PreviewAction icon="content_copy" onClick={() => void copier()}>Copier</PreviewAction>
                  <PreviewAction icon="download" onClick={() => void exportLetterPdf({ name: selected_letter.name, company: selected_letter.company, job_title: selected_letter.job_title, content: selected_letter.content }, notify)}>Exporter le PDF</PreviewAction>
                  <PreviewAction tone="danger" icon="delete" onClick={() => setDeleteId(selected_letter.id)}>Supprimer</PreviewAction>
                </div>
              ) : null}
            </div>
            <div className="min-h-0 flex-1 overflow-auto">
              {selected_letter ? <LetterPreview letter={selected_letter} /> : <EmptyState icon="draft" title="Sélectionnez une lettre" />}
            </div>
          </div>
        </div>
      <ConfirmDialog open={deleteId !== null} title="Supprimer cette lettre ?" description="La lettre sera retirée de la bibliothèque locale." note="Le profil et les autres documents seront conservés." busy={remove.isPending} onCancel={() => setDeleteId(null)} onConfirm={() => { if (deleteId) remove.mutate(deleteId); }} />
    </Screen>
  );
}

function LetterPreview({ letter }: { letter: CoverLetter }) { return <A4Preview title={letter.name}><p className="mt-4 text-[11px] uppercase tracking-[0.12em] text-[#5b6ee1]">Lettre de motivation</p><h2 className="mt-3 text-[23px] font-semibold">{letter.job_title ?? "Candidature"}</h2><p className="mt-1 text-[12px] text-[#6a7280]">{letter.company ?? "Entreprise"}</p><div className="mt-8 whitespace-pre-wrap text-[12px] leading-[1.9]">{letter.content}</div></A4Preview>; }

export function LetterWriterPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const location = useLocation();
  const cover_letter_initiale = coverLetterFromNavigation(location.state);
  const [company, setCompany] = useState(cover_letter_initiale?.company ?? "");
  const [job_title, setJobTitle] = useState(cover_letter_initiale?.job_title ?? "");
  const [tone, setTone] = useState(cover_letter_initiale?.tone || "formal");
  const [length, setLength] = useState(cover_letter_initiale?.length || "medium");
  const [context, setContext] = useState("");
  const [output, setOutput] = useState(cover_letter_initiale?.content ?? "");
  const [operation, setOperation] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useAiProgress(operation);

  const run = async () => {
    const id = generation_id();
    setOperation(id);
    setOutput("");
    setError(null);
    try {
      setOutput(await aiService.generateCoverLetter({ generation_id: id, company: company || null, job_title: job_title || null, tone, length, context: context || null, previous_cover_letter: null, instruction: null }));
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };
  const save = useMutation({
    mutationFn: () => documentsService.saveCoverLetter({ name: `Lettre — ${job_title || company || "Candidature"}`, company: company || null, job_title: job_title || null, tone, length, content: output }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: COVER_LETTERS_KEY });
      notify({ tone: "success", title: "Lettre enregistrée" });
    },
  });

  return (
    <Screen header={
      <PageHeader
        icon="edit_note"
        title="Lettre de motivation"
        subtitle="Rédigez, itérez et enregistrez"
        badge={operation ? <HeaderBadge>IA active</HeaderBadge> : undefined}
        secondary={output ? <Button icon="download" onClick={() => void exportLetterPdf({ name: `Lettre — ${job_title || company || "Candidature"}`, company: company || null, job_title: job_title || null, content: output }, notify)}>Exporter le PDF</Button> : undefined}
        primary={output ? <Button variant="primary" icon="save" disabled={save.isPending} onClick={() => save.mutate()}>Enregistrer</Button> : undefined}
      />
    }>
      <div className="grid min-h-[660px] gap-4 xl:grid-cols-[350px_minmax(480px,1fr)]">
        <DocumentPanel title="Brief de rédaction" icon="target">
          <div className="space-y-4 p-4">
            <Champ label="Entreprise" value={company} onChange={setCompany} />
            <Champ label="Poste ciblé" value={job_title} onChange={setJobTitle} />
            <div className="grid grid-cols-2 gap-3">
              <FormField label="Ton">{(props) => <Select {...props} value={tone} onChange={(e) => setTone(e.target.value)}><option value="formal">Formel</option><option value="casual">Naturel</option><option value="creative">Créatif</option></Select>}</FormField>
              <FormField label="Longueur">{(props) => <Select {...props} value={length} onChange={(e) => setLength(e.target.value)}><option value="short">Courte</option><option value="medium">Moyenne</option><option value="long">Longue</option></Select>}</FormField>
            </div>
            <FormField label="Contexte ou offre">{(props) => <TextArea {...props} rows={10} value={context} onChange={(e) => setContext(e.target.value)} />}</FormField>
            {error ? <ErrorBanner title="Rédaction impossible" message={error} /> : null}
            {operation ? (
              <><AiProgress progress={progress} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void aiService.cancel(operation)}>Arrêter</Button></>
            ) : (
              <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>Rédiger la lettre</Button>
            )}
          </div>
        </DocumentPanel>
        <DocumentPanel title="Document" icon="draft">
          <A4Preview title="Lettre de motivation">
            <p className="mt-4 text-[11px] uppercase tracking-[0.12em] text-[#5b6ee1]">Lettre de motivation</p>
            <h2 className="mt-3 text-[23px] font-semibold">{job_title || "Candidature"}</h2>
            <p className="mt-1 text-[12px] text-[#6a7280]">{company || "Entreprise ciblée"}</p>
            <div className="mt-8 whitespace-pre-wrap text-[12px] leading-[1.9] text-[#303641]">{output || "La lettre apparaîtra ici après la rédaction."}</div>
          </A4Preview>
        </DocumentPanel>
      </div>
    </Screen>
  );
}

export function ResumeAnalysisPage() {
  const [path, setPath] = useState<string | null>(null);
  const [job_offer, setJobOffer] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [result, setResult] = useState<ImportedResumeAnalysis | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useAiProgress(operation);
  const choose = async () => {
    const file = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof file === "string") setPath(file);
  };
  const run = async () => {
    if (!path || !job_offer.trim()) { setError("Sélectionnez un PDF et collez l’offre ciblée."); return; }
    const id = generation_id();
    setOperation(id);
    setError(null);
    try {
      setResult(await aiService.analyzeResume({ generation_id: id, path: path, job_offer }));
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
        primary={<Button variant="primary" icon="bolt" disabled={operation !== null} onClick={() => void run()}>Analyze le Resume</Button>}
      />
    }>
      <div className="grid gap-4 xl:grid-cols-[400px_minmax(480px,1fr)]">
        <div className="space-y-4">
          <DocumentPanel title="Document à analyser" icon="upload_file">
            <div className="space-y-4 p-4">
              <button type="button" onClick={() => void choose()} className="flex w-full flex-col items-center gap-2 rounded-card border border-dashed border-accent-border bg-accent-tint px-5 py-8 text-center">
                <Icon name={path ? "picture_as_pdf" : "upload_file"} size={28} className="text-accent" />
                <span className="font-medium text-ink">{path ? path.split("/").at(-1) : "Choisir un CV PDF"}</span>
                <span className="text-meta text-ink-muted">PDF uniquement · 10 Mo maximum</span>
              </button>
              <FormField label="Offre ciblée" required>{(props) => <TextArea {...props} rows={13} value={job_offer} onChange={(e) => setJobOffer(e.target.value)} />}</FormField>
              {operation ? <AiProgress progress={progress} /> : null}
              {error ? <ErrorBanner title="Analyse impossible" message={error} /> : null}
            </div>
          </DocumentPanel>
        </div>
        <div className="space-y-4">
          {result ? (
            <>
              <DocumentPanel title="Résultat" icon="analytics">
                <div className="grid gap-5 p-4 sm:grid-cols-[auto_1fr]"><ScoreBadge value={result.analysis.score} /><p className="text-body leading-relaxed text-ink-muted">{result.analysis.recap}</p></div>
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

function Screen({
  header,
  children,
  padded = true,
  search,
}: {
  header: ReactNode;
  children: ReactNode;
  padded?: boolean;
  search?: { value: string; onChange: (value: string) => void; placeholder: string };
}) {
  return (
    <div className="flex h-full flex-col">
      {search ? (
        <ContextBarAccessory>
          <ContextSearch
            value={search.value}
            placeholder={search.placeholder}
            onChange={search.onChange}
            width={230}
          />
        </ContextBarAccessory>
      ) : (
        <ContextBarAccessory>
          <ContextNote>Documents locaux · gNRation Ai</ContextNote>
        </ContextBarAccessory>
      )}
      {header}
      <div className={padded ? "min-h-0 flex-1 overflow-y-auto p-5 min-[1200px]:p-6" : "flex min-h-0 flex-1 flex-col overflow-hidden"}>
        {children}
      </div>
    </div>
  );
}

function HeaderBadge({ children, icon = "auto_awesome" }: { children: ReactNode; icon?: string }) {
  return (
    <span className="inline-flex items-center gap-[5px] rounded-pill bg-accent-tint px-2.5 py-[5px] text-label font-mid text-accent">
      <Icon name={icon} size={15} />
      {children}
    </span>
  );
}

function AtsChip({ score }: { score: number }) {
  const tone = score >= 80 ? "bg-success-tint text-success" : score >= 65 ? "bg-warning-tint text-warning" : "bg-neutral-tint text-ink-muted";
  return <span className={`rounded-[5px] px-1.5 py-0.5 text-[10.5px] font-semibold ${tone}`}>ATS {score}</span>;
}

function Champ({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return <FormField label={label}>{(props) => <TextInput {...props} value={value} onChange={(e) => onChange(e.target.value)} />}</FormField>;
}
function message(error: unknown): string { return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite."; }
function date(value: string): string { const d = new Date(value); return Number.isNaN(d.getTime()) ? value : new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "short", year: "numeric" }).format(d); }
function isGeneration(value: unknown): value is ResumeGeneration { return typeof value === "object" && value !== null && "resume" in value && "analysis" in value; }
function generationFromNavigation(state: unknown): { result: ResumeGeneration | null; name: string } {
  if (typeof state !== "object" || state === null || !("generation" in state)) {
    return { result: null, name: "" };
  }
  const payload = state as { generation?: ResumeGeneration; name?: string };
  if (!payload.generation) return { result: null, name: "" };
  return {
    result: payload.generation,
    name: payload.name ?? `CV — ${payload.generation.job_offer.title || "Version ciblée"}`,
  };
}
function coverLetterFromNavigation(state: unknown): CoverLetter | null {
  if (typeof state !== "object" || state === null || !("cover_letter" in state)) return null;
  const payload = state as { cover_letter?: CoverLetter };
  return payload.cover_letter ?? null;
}
function labelTone(tone: string): string {
  if (tone === "casual") return "Naturel";
  if (tone === "creative") return "Créatif";
  return "Formel";
}

async function exportPdf(
  resume: GeneratedResume,
  name: string,
  notify: (toast: Omit<ToastMessage, "id">) => void,
) {
  const base = name.trim().replace(/[\\/:*?"<>|]+/g, "-") || "resume-candilog";
  const path = await save({
    title: "Exporter le CV en PDF",
    defaultPath: `${base}.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (path === null) return;
  try {
    await documentsService.exportPdf(resume, path);
    notify({ tone: "success", title: "CV exporté" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}

async function exportLetterPdf(
  cover_letter: CoverLetterExport,
  notify: (toast: Omit<ToastMessage, "id">) => void,
) {
  const base = cover_letter.name.trim().replace(/[\\/:*?"<>|]+/g, "-") || "lettre-candilog";
  const path = await save({
    title: "Exporter la lettre en PDF",
    defaultPath: `${base}.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (path === null) return;
  try {
    await documentsService.exportCoverLetterPdf(cover_letter, path);
    notify({ tone: "success", title: "Lettre exportée" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}
