import { useLayoutEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "react-router-dom";
import { documentsService, type CoverLetter } from "../../services/documentsService";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { useAiTimer } from "@/features/ai/viewmodel/useAiTimer";
import { formatDuration } from "@/shared/lib/duration";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, FormField, Icon, PageHeader, Pager, Select } from "@/shared/ui";
import { A4Preview, AiProgress, DocumentPanel, PreviewAction } from "../components/DocumentUi";
import { PAGE_SIZE } from "@/shared/types/page";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { Champ, ChampOffre, COVER_LETTERS_KEY, HeaderBadge, Screen, coverLetterFromNavigation, date, detail, exportLetterPdf, labelTone, message } from "./documentPageSupport";

export function LettersLibraryPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [recherche, setRecherche] = useState("");
  const searchQuery = useDebounce(recherche);
  const list = useQuery({
    queryKey: [...COVER_LETTERS_KEY, "page", { page, search: searchQuery }],
    queryFn: () => documentsService.listCoverLettersPage({ page, page_size: PAGE_SIZE, search: searchQuery }),
  });
  const cover_letters = list.data?.items ?? [];
  const selected_id = cover_letters.some((letter) => letter.id === selected)
    ? selected
    : (cover_letters[0]?.id ?? null);
  const selected_letter = cover_letters.find((letter) => letter.id === selected_id) ?? null;
  const remove = useMutation({
    mutationFn: documentsService.deleteCoverLetter,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: COVER_LETTERS_KEY });
    },
    onError: (error) => {
      setDeleteId(null);
      notify({ tone: "error", title: "Suppression impossible", detail: detail(error) });
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
      search={{ value: recherche, onChange: (value) => { setRecherche(value); setPage(1); }, placeholder: "Rechercher un document…" }}
      header={
      <PageHeader
        icon="mail"
        title="Mes lettres de motivation"
        subtitle="Bibliothèque"
        badge={list.data ? <HeaderBadge>{list.data.total} lettre{list.data.total > 1 ? "s" : ""}</HeaderBadge> : undefined}
        primary={<Button variant="primary" icon="auto_awesome" onClick={() => void navigate("/documents/write-cover-letter")}>Rédiger une lettre</Button>}
      />
    }>
      <div className="flex min-h-0 flex-1">
          <div className="flex w-[36%] min-w-[260px] flex-col border-r border-line bg-surface">
            <div className="flex items-center justify-between border-b border-line px-5 py-3">
              <span className="text-section">Bibliothèque</span>
              <span className="text-label text-ink-faint">{list.data?.total ?? 0} lettre{(list.data?.total ?? 0) > 1 ? "s" : ""}</span>
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
                        className={`w-full rounded-tile border px-3.5 py-3 text-left transition-colors ${selected_id === letter.id ? "border-accent-border bg-accent-tint" : "border-transparent hover:bg-neutral-tint"}`}
                      >
                        <span className="flex items-center gap-2">
                          <Icon name="mail" size={16} className={selected_id === letter.id ? "text-accent" : "text-ink-faint"} />
                          <span className="min-w-0 flex-1 truncate text-[13px] font-semibold">{letter.company ?? letter.name}</span>
                          <span className="flex-none text-label text-ink-faint">{date(letter.created_at)}</span>
                        </span>
                        <span className="mt-1 block truncate pl-6 text-label text-ink-faint">{letter.job_title ?? "Candidature"} · {labelTone(letter.tone)}</span>
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
            <Pager page={page} page_size={PAGE_SIZE} total={list.data?.total ?? 0} label="lettres" dense onPageChange={setPage} />
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

function LetterPreview({ letter }: { letter: CoverLetter }) { return <A4Preview title={letter.name}><p className="mt-4 text-[11px] uppercase tracking-[0.12em] text-paper-muted">Lettre de motivation</p><h2 className="mt-3 text-[23px] font-semibold">{letter.job_title ?? "Candidature"}</h2><p className="mt-1 text-[12px] text-paper-muted">{letter.company ?? "Entreprise"}</p><div className="mt-8 whitespace-pre-wrap text-[12px] leading-[1.9]">{letter.content}</div></A4Preview>; }

/**
 * Corps de la lettre, édité **sur la feuille** : ce qui est lu à l'écran est exactement ce
 * qui sera enregistré et exporté en PDF. La zone est une saisie transparente calée sur la
 * typographie du papier plutôt qu'un éditeur riche : la lettre est stockée en texte brut,
 * et faire croire à du gras qui disparaîtrait à l'export serait pire que pas d'éditeur.
 */
function LetterBody({
  value,
  readOnly,
  onChange,
}: {
  value: string;
  readOnly: boolean;
  onChange: (value: string) => void;
}) {
  const zone = useRef<HTMLTextAreaElement | null>(null);

  // La feuille grandit avec le texte : une barre de défilement interne à la page A4
  // couperait la lettre en deux et ne correspondrait à aucun rendu imprimé.
  useLayoutEffect(() => {
    const element = zone.current;
    if (!element) return;
    element.style.height = "auto";
    element.style.height = `${element.scrollHeight}px`;
  }, [value]);

  return (
    <textarea
      ref={zone}
      value={value}
      readOnly={readOnly}
      aria-label="Contenu de la lettre"
      placeholder="La lettre apparaîtra ici après la rédaction. Vous pouvez aussi l'écrire ou la retoucher directement sur cette page."
      onChange={(event) => onChange(event.target.value)}
      className="mt-8 min-h-[420px] w-full resize-none border-0 bg-transparent p-0 text-[12px] leading-[1.9] text-paper-ink outline-none placeholder:text-paper-muted focus:outline-none"
    />
  );
}

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
  useCancelAiOnUnmount(operation);
  const timer = useAiTimer(operation !== null);

  const run = async () => {
    const id = generation_id();
    setOperation(id);
    setOutput("");
    setError(null);
    timer.start();
    try {
      setOutput(await aiService.generateCoverLetter({ generation_id: id, company: company || null, job_title: job_title || null, tone, length, context: context || null, previous_cover_letter: null, instruction: null }));
      timer.stop();
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
    // Sans ce gestionnaire, un refus du service Rust laissait l'écran inchangé : la lettre
    // n'était pas enregistrée et rien ne le disait.
    onError: (error) =>
      notify({ tone: "error", title: "Enregistrement impossible", detail: detail(error) }),
  });

  return (
    <Screen header={
      <PageHeader
        icon="edit_note"
        title="Lettre de motivation"
        subtitle="Rédigez, itérez et enregistrez"
        badge={operation ? <HeaderBadge>IA active</HeaderBadge> : timer.durationMs !== null ? <HeaderBadge icon="schedule">Rédigée en {formatDuration(timer.durationMs)}</HeaderBadge> : undefined}
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
            <ChampOffre label="Contexte ou offre" rows={10} value={context} onChange={setContext} />
            {error ? <ErrorBanner title="Rédaction impossible" message={error} /> : null}
            {operation ? (
              <><AiProgress progress={progress} elapsedMs={timer.elapsedMs} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void aiService.cancel(operation)}>Arrêter</Button></>
            ) : (
              <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>Rédiger la lettre</Button>
            )}
          </div>
        </DocumentPanel>
        <DocumentPanel title="Document" icon="draft">
          <A4Preview title="Lettre de motivation">
            <p className="mt-4 text-[11px] uppercase tracking-[0.12em] text-paper-muted">Lettre de motivation</p>
            <h2 className="mt-3 text-[23px] font-semibold">{job_title || "Candidature"}</h2>
            <p className="mt-1 text-[12px] text-paper-muted">{company || "Entreprise ciblée"}</p>
            <LetterBody value={output} readOnly={operation !== null} onChange={setOutput} />
          </A4Preview>
        </DocumentPanel>
      </div>
    </Screen>
  );
}
