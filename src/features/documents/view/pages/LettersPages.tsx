import { useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "react-router-dom";
import { documentsService, type CoverLetter } from "../../services/documentsService";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { useAiTimer } from "@/features/ai/viewmodel/useAiTimer";
import { formatAiSummary } from "@/shared/lib/duration";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, FormField, Icon, PageHeader, Pager, Select, TextArea } from "@/shared/ui";
import { AiProgress, DocumentPanel, PreviewAction } from "../components/DocumentUi";
import { LetterContent, LetterEditor } from "../components/LetterEditor";
import { LetterPaper, type LetterPaperField } from "../components/LetterPaper";
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
                  <PreviewAction icon="download" onClick={() => void exportLetterPdf({ name: selected_letter.name, company: selected_letter.company, job_title: selected_letter.job_title, recipient: selected_letter.recipient, recipient_address: selected_letter.recipient_address, job_reference: selected_letter.job_reference, content: selected_letter.content }, notify)}>Exporter le PDF</PreviewAction>
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

/** Aperçu d'une lettre enregistrée : la même feuille que l'éditeur, donc que le PDF. */
function LetterPreview({ letter }: { letter: CoverLetter }) {
  return (
    <div className="flex justify-center bg-page p-[26px]">
      <LetterPaper
        fields={{
          company: letter.company,
          job_title: letter.job_title,
          recipient: letter.recipient,
          recipient_address: letter.recipient_address,
          job_reference: letter.job_reference,
        }}
      >
        <LetterContent content={letter.content} />
      </LetterPaper>
    </div>
  );
}

/** Nombre de consignes réinjectées : au-delà, le brief devient illisible pour le modèle. */
const MAX_CONSIGNES = 8;

type Echange = { auteur: "vous" | "candilog"; texte: string };

/**
 * Suite du travail une fois la première lettre écrite.
 *
 * Le brief n'a plus rien à demander à ce stade : ce qui reste à faire, c'est demander des
 * ajustements. Les consignes s'accumulent — « plus court » puis « plus formel » doivent
 * valoir ensemble — et chaque régénération journalise le temps qu'elle a pris, à l'endroit
 * où l'utilisateur regarde.
 */
function IterationPanel({
  echanges,
  consigne,
  busy,
  error,
  progress,
  onConsigneChange,
  onSubmit,
  onCancel,
  onReopenBrief,
}: {
  echanges: Echange[];
  consigne: string;
  busy: boolean;
  error: string | null;
  progress: ReactNode;
  onConsigneChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
  onReopenBrief: () => void;
}) {
  return (
    <DocumentPanel title="Itérations" icon="forum" className="flex min-h-0 flex-col">
      <div className="flex min-h-0 flex-1 flex-col">
        <ol className="min-h-0 flex-1 space-y-2 overflow-y-auto p-4">
          {echanges.map((echange, index) => (
            <li
              key={`${echange.auteur}-${index}`}
              className={echange.auteur === "vous" ? "flex justify-end" : "flex justify-start"}
            >
              <span
                className={
                  echange.auteur === "vous"
                    ? "max-w-[85%] rounded-card bg-accent-tint px-3 py-2 text-body text-ink"
                    : "inline-flex max-w-[85%] items-center gap-1.5 rounded-card bg-fill px-3 py-2 text-meta text-ink-muted"
                }
              >
                {echange.auteur === "candilog" ? <Icon name="schedule" size={14} className="flex-none" /> : null}
                {echange.texte}
              </span>
            </li>
          ))}
        </ol>
        <div className="space-y-3 border-t border-line p-4">
          {error ? <ErrorBanner title="Rédaction impossible" message={error} /> : null}
          {busy ? (
            <>
              {progress}
              <Button variant="danger" icon="stop" className="w-full" onClick={onCancel}>Arrêter</Button>
            </>
          ) : (
            <>
              <FormField label="Que faut-il changer ?">
                {(props) => (
                  <TextArea
                    {...props}
                    rows={3}
                    value={consigne}
                    placeholder="Ex. « Mets en avant ma dernière expérience » ou « Va droit au but »"
                    onChange={(event) => onConsigneChange(event.target.value)}
                  />
                )}
              </FormField>
              <Button
                variant="primary"
                icon="auto_awesome"
                className="w-full"
                disabled={consigne.trim() === ""}
                onClick={onSubmit}
              >
                Régénérer avec cette consigne
              </Button>
              <Button variant="ghost" icon="target" className="w-full" onClick={onReopenBrief}>
                Revenir au brief
              </Button>
            </>
          )}
        </div>
      </div>
    </DocumentPanel>
  );
}

export function LetterWriterPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const location = useLocation();
  const cover_letter_initiale = coverLetterFromNavigation(location.state);
  const [company, setCompany] = useState(cover_letter_initiale?.company ?? "");
  const [job_title, setJobTitle] = useState(cover_letter_initiale?.job_title ?? "");
  const [recipient, setRecipient] = useState(cover_letter_initiale?.recipient ?? "");
  const [recipient_address, setRecipientAddress] = useState(cover_letter_initiale?.recipient_address ?? "");
  const [job_reference, setJobReference] = useState(cover_letter_initiale?.job_reference ?? "");
  const [tone, setTone] = useState(cover_letter_initiale?.tone || "formal");
  const [length, setLength] = useState(cover_letter_initiale?.length || "medium");
  const [context, setContext] = useState("");
  const [output, setOutput] = useState(cover_letter_initiale?.content ?? "");
  const [operation, setOperation] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [echanges, setEchanges] = useState<Echange[]>([]);
  const [consignes, setConsignes] = useState<string[]>([]);
  const [consigne, setConsigne] = useState("");
  const [briefOuvert, setBriefOuvert] = useState(false);
  const [abandonOuvert, setAbandonOuvert] = useState(false);
  const [overflow, setOverflow] = useState(false);
  const progress = useAiProgress(operation);
  useCancelAiOnUnmount(operation);
  const timer = useAiTimer(operation !== null);
  // Le brief laisse la place aux itérations dès qu'une lettre existe : c'est là que se
  // poursuit le travail, et le rouvrir reste possible pour changer le ton ou l'offre.
  const enIteration = echanges.length > 0 && !briefOuvert;

  const run = async (instruction: string | null) => {
    const id = generation_id();
    const suite = instruction === null ? consignes : [...consignes, instruction].slice(-MAX_CONSIGNES);
    setOperation(id);
    setError(null);
    if (instruction === null) setOutput("");
    if (instruction !== null) {
      setConsignes(suite);
      setConsigne("");
      setEchanges((current) => [...current, { auteur: "vous", texte: instruction }]);
    }
    timer.start();
    try {
      const execution = await aiService.generateCoverLetter({ generation_id: id, company: company || null, job_title: job_title || null, tone, length, context: context || null, previous_cover_letter: null, instruction: suite.length > 0 ? suite.join(" ; ") : null });
      timer.stop();
      setOutput(execution.output);
      setEchanges((current) => [...current, {
        auteur: "candilog",
        texte: formatAiSummary(
          instruction === null ? "Lettre rédigée" : "Lettre régénérée",
          execution.elapsed_ms,
          execution.tokens_used,
        ),
      }]);
      setBriefOuvert(false);
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };
  const letterExport = () => ({
    name: `Lettre — ${job_title || company || "Candidature"}`,
    company: company || null,
    job_title: job_title || null,
    recipient: recipient || null,
    recipient_address: recipient_address || null,
    job_reference: job_reference || null,
    content: output,
  });
  const save = useMutation({
    mutationFn: () => documentsService.saveCoverLetter({ ...letterExport(), tone, length }),
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
    <Screen padded={false} header={
      <PageHeader
        icon="edit_note"
        title="Lettre de motivation"
        subtitle="Rédigez, itérez et enregistrez"
        secondary={output ? (
          <>
            <Button icon="download" disabled={overflow} onClick={() => void exportLetterPdf(letterExport(), notify)}>Exporter le PDF</Button>
            <Button icon="close" onClick={() => setAbandonOuvert(true)}>Annuler</Button>
          </>
        ) : undefined}
        primary={output ? <Button variant="primary" icon="save" disabled={save.isPending || overflow} onClick={() => save.mutate()}>Enregistrer</Button> : undefined}
      />
    }>
      <div className="grid min-h-0 flex-1 gap-4 overflow-hidden p-5 min-[1200px]:p-6 xl:grid-cols-[350px_minmax(480px,1fr)]">
        {enIteration ? (
          <IterationPanel
            echanges={echanges}
            consigne={consigne}
            busy={operation !== null}
            error={error}
            progress={operation ? <AiProgress progress={progress} elapsedMs={timer.elapsedMs} /> : null}
            onConsigneChange={setConsigne}
            onSubmit={() => void run(consigne.trim())}
            onCancel={() => { if (operation) void aiService.cancel(operation); }}
            onReopenBrief={() => setBriefOuvert(true)}
          />
        ) : (
        <DocumentPanel title="Brief de rédaction" icon="target" className="flex min-h-0 flex-col">
          <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
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
              <div className="flex flex-col gap-2">
                <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run(null)}>{echanges.length > 0 ? "Rédiger une nouvelle lettre" : "Rédiger la lettre"}</Button>
                {echanges.length > 0 ? (
                  <Button variant="ghost" icon="forum" className="w-full" onClick={() => setBriefOuvert(false)}>Revenir aux itérations</Button>
                ) : null}
              </div>
            )}
          </div>
        </DocumentPanel>
        )}
        <LetterEditor
          value={output}
          readOnly={operation !== null}
          fields={{
            company,
            job_title,
            recipient,
            recipient_address,
            job_reference,
          }}
          onChange={setOutput}
          onFieldsChange={(field: LetterPaperField, value: string) => {
            if (field === "company") setCompany(value);
            else if (field === "job_title") setJobTitle(value);
            else if (field === "recipient") setRecipient(value);
            else if (field === "recipient_address") setRecipientAddress(value);
            else setJobReference(value);
          }}
          onOverflowChange={setOverflow}
        />
      </div>
      <ConfirmDialog
        open={abandonOuvert}
        title="Abandonner cette lettre ?"
        description="La lettre affichée et les consignes de l'itération seront perdues."
        note="Le brief, lui, est conservé : vous pourrez relancer une rédaction."
        confirmLabel="Abandonner"
        onCancel={() => setAbandonOuvert(false)}
        onConfirm={() => {
          setAbandonOuvert(false);
          setOutput("");
          setEchanges([]);
          setConsignes([]);
          setConsigne("");
          setBriefOuvert(false);
        }}
      />
    </Screen>
  );
}
