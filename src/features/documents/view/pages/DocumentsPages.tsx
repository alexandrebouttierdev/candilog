import { useMemo, useState, type ReactNode } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "react-router-dom";
import { documentsService } from "../../services/documents.service";
import type { CvVersion, Lettre } from "../../services/documents.service";
import { iaService, generationId } from "@/features/ia/services/ia.service";
import type { AnalyseCvImporte, CvGenere, GenerationCv } from "@/features/ia/model/types";
import { useIaProgress } from "@/features/ia/viewmodel/useIaProgress";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, FormField, Icon, PageHeader, Select, TextArea, TextInput } from "@/shared/ui";
import { A4Preview, DocumentPanel, IaProgress, ScoreBadge } from "../components/DocumentUi";

const CV_KEY = ["documents", "cv"] as const;
const LETTRES_KEY = ["documents", "lettres"] as const;

export function CvLibraryPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const list = useQuery({ queryKey: CV_KEY, queryFn: documentsService.listerCv });
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [recherche, setRecherche] = useState("");
  const selectedId = selected ?? list.data?.[0]?.id ?? null;
  const detail = useQuery({
    queryKey: [...CV_KEY, selectedId],
    queryFn: () => documentsService.obtenirCv(selectedId ?? ""),
    enabled: selectedId !== null,
  });
  const remove = useMutation({
    mutationFn: documentsService.supprimerCv,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: CV_KEY });
      notify({ tone: "success", title: "Version supprimée" });
    },
  });
  const dupliquer = useMutation({
    mutationFn: async () => {
      if (!detail.data) return;
      await documentsService.enregistrerCv({
        nom: `${detail.data.nom} (copie)`,
        contenu: detail.data.contenu,
      });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: CV_KEY });
      notify({ tone: "success", title: "Version dupliquée" });
    },
  });
  const generation = detail.data && isGeneration(detail.data.contenu) ? detail.data.contenu : null;
  const version = detail.data;
  const versions = useMemo(() => {
    const terme = recherche.trim().toLowerCase();
    const toutes = list.data ?? [];
    return terme ? toutes.filter((cv) => cv.nom.toLowerCase().includes(terme)) : toutes;
  }, [list.data, recherche]);

  return (
    <Screen padded={false} header={
      <PageHeader
        icon="description"
        title="Mes CV"
        subtitle="Vos versions prêtes à l’emploi"
        badge={list.data ? <HeaderBadge>{list.data.length} version{list.data.length > 1 ? "s" : ""}</HeaderBadge> : undefined}
        secondary={<Button icon="upload_file" onClick={() => void navigate("/documents/analyser")}>Importer</Button>}
        primary={<Button variant="primary" icon="auto_awesome" onClick={() => void navigate("/documents/generer-cv")}>Nouveau CV</Button>}
      />
    }>
      <div className="flex min-h-0 flex-1">
          <div className="flex w-[40%] min-w-[280px] flex-col border-r border-line bg-surface">
            <div className="border-b border-line px-5 py-3">
              <div className="mb-2.5 flex items-center justify-between">
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
                  {versions.map((cv) => (
                    <li key={cv.id}>
                      <button
                        type="button"
                        aria-pressed={selectedId === cv.id}
                        onClick={() => setSelected(cv.id)}
                        className={`flex w-full gap-3 rounded-[10px] border p-3 text-left transition-colors ${selectedId === cv.id ? "border-accent-border bg-accent-tint" : "border-transparent hover:bg-neutral-tint"}`}
                      >
                        <span className="flex h-[50px] w-[38px] flex-none flex-col gap-[3px] rounded-[5px] border border-line bg-page p-1.5">
                          <span className="h-[3px] w-[70%] rounded-sm bg-accent/40" />
                          <span className="h-[2px] w-full rounded-sm bg-line" />
                          <span className="h-[2px] w-[85%] rounded-sm bg-line" />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="flex items-center gap-2">
                            <span className="truncate text-[13px] font-semibold">{cv.nom}</span>
                          </span>
                          <span className="mt-1 block text-label text-ink-faint">{date(cv.createdAt)}</span>
                        </span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : recherche.trim() ? (
                <EmptyState icon="search" title="Aucun résultat" description="Aucune version ne correspond à cette recherche." />
              ) : (
                <EmptyState icon="description" title="Aucune version" description="Générez puis sauvegardez votre premier CV ciblé." action={<Button icon="auto_awesome" onClick={() => void navigate("/documents/generer-cv")}>Générer un CV</Button>} />
              )}
            </div>
          </div>
          <div className="flex min-w-0 flex-1 flex-col bg-page">
            <div className="flex flex-none items-center justify-between border-b border-line bg-surface px-5 py-3">
              <div className="flex min-w-0 items-center gap-2.5">
                <Icon name="visibility" size={17} className="text-ink-faint" />
                <p className="truncate text-section">{version ? `Aperçu · ${version.nom}` : "Aperçu"}</p>
                {generation ? <AtsChip score={generation.analyse.score} /> : null}
              </div>
              {version ? (
                <div className="flex gap-2">
                  {generation ? (
                    <>
                      <Button icon="edit" onClick={() => void navigate("/documents/generer-cv", { state: { generation, nom: version.nom } })}>Modifier</Button>
                      <Button icon="content_copy" disabled={dupliquer.isPending} onClick={() => dupliquer.mutate()}>Dupliquer</Button>
                      <Button icon="download" onClick={() => void exporterPdf(generation.cv, version.nom, notify)}>Exporter PDF</Button>
                    </>
                  ) : null}
                  <Button variant="danger" icon="delete" onClick={() => setDeleteId(version.id)}>Supprimer</Button>
                </div>
              ) : null}
            </div>
            <div className="min-h-0 flex-1 overflow-auto">
              {detail.data ? <CvSavedPreview version={detail.data} /> : <EmptyState icon="visibility" title="Sélectionnez une version" description="Son contenu détaillé apparaîtra ici." />}
            </div>
          </div>
        </div>
      <ConfirmDialog open={deleteId !== null} title="Supprimer cette version ?" description="Le CV disparaîtra définitivement de la bibliothèque locale." note="Votre profil et vos autres versions seront conservés." busy={remove.isPending} onCancel={() => setDeleteId(null)} onConfirm={() => { if (deleteId) remove.mutate(deleteId); }} />
    </Screen>
  );
}

function CvSavedPreview({ version }: { version: CvVersion }) { const generation = isGeneration(version.contenu) ? version.contenu : null; return generation ? <A4Preview cv={generation.cv} /> : <A4Preview title={version.nom}><div className="flex min-h-[590px] items-center justify-center text-center text-[#7b8493]">Cette ancienne version ne contient pas encore d’aperçu structuré compatible.</div></A4Preview>; }

export function CvGeneratorPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const location = useLocation();
  const initiale = generationDepuisNavigation(location.state);
  const [offre, setOffre] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [result, setResult] = useState<GenerationCv | null>(initiale.result);
  const [error, setError] = useState<string | null>(null);
  const [nom, setNom] = useState(initiale.nom);
  const progress = useIaProgress(operation);

  const run = async () => {
    if (!offre.trim()) { setError("Collez le texte de l’offre à cibler."); return; }
    const id = generationId();
    setOperation(id);
    setError(null);
    try {
      const value = await iaService.genererCv({ generationId: id, offre });
      setResult(value);
      setNom(`CV — ${value.offre.titre || "Version ciblée"}`);
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };
  const save = useMutation({
    mutationFn: () => documentsService.enregistrerCv({ nom, contenu: result }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: CV_KEY });
      notify({ tone: "success", title: "CV ajouté à la bibliothèque" });
    },
  });

  return (
    <Screen header={
      <PageHeader
        icon="auto_awesome"
        title="Générer un CV"
        subtitle="Analysez une offre, générez un CV ciblé, exportez en PDF"
        badge={operation ? <HeaderBadge>IA active</HeaderBadge> : undefined}
        secondary={result ? <Button icon="save" disabled={!nom.trim() || save.isPending} onClick={() => save.mutate()}>Enregistrer</Button> : undefined}
        primary={result ? <Button variant="primary" icon="download" onClick={() => void exporterPdf(result.cv, nom || "cv-candilog", notify)}>Exporter le PDF</Button> : undefined}
      />
    }>
      <div className="grid min-h-[660px] gap-4 xl:grid-cols-[350px_minmax(460px,1fr)_320px]">
        <DocumentPanel title="Offre ciblée" icon="target">
          <div className="space-y-4 p-4">
            <FormField label="Texte de l’offre" required help="Le texte est envoyé uniquement au fournisseur configuré.">
              {(props) => <TextArea {...props} rows={18} value={offre} placeholder="Collez ici l’intitulé, les missions et les compétences recherchées…" onChange={(e) => setOffre(e.target.value)} />}
            </FormField>
            {error ? <ErrorBanner title="Génération impossible" message={error} /> : null}
            {operation ? (
              <><IaProgress progress={progress} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void iaService.annuler(operation)}>Annuler</Button></>
            ) : (
              <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>Générer le CV ciblé</Button>
            )}
          </div>
        </DocumentPanel>
        <DocumentPanel title="Aperçu HTML · A4" icon="article"><A4Preview cv={result?.cv} /></DocumentPanel>
        <DocumentPanel title="Analyse ATS" icon="query_stats">
          <div className="space-y-5 p-4">
            {result ? (
              <>
                <ScoreBadge value={result.analyse.score} />
                <p className="text-body leading-relaxed text-ink-muted">{result.analyse.recap}</p>
                <div>
                  <p className="mb-2 text-label font-medium text-ink">Suggestions</p>
                  <ul className="space-y-2">{result.analyse.suggestions.map((s, i) => <li key={i} className="flex gap-2 text-body text-ink-muted"><Icon name="arrow_right" size={15} className="mt-0.5 text-accent" />{s}</li>)}</ul>
                </div>
                <FormField label="Nom de la version" required>{(props) => <TextInput {...props} value={nom} onChange={(e) => setNom(e.target.value)} />}</FormField>
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
  const list = useQuery({ queryKey: LETTRES_KEY, queryFn: documentsService.listerLettres });
  const selectedId = selected ?? list.data?.[0]?.id ?? null;
  const selectedLetter = list.data?.find((l) => l.id === selectedId) ?? null;
  const remove = useMutation({
    mutationFn: documentsService.supprimerLettre,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: LETTRES_KEY });
    },
  });

  const copier = async () => {
    if (!selectedLetter) return;
    await navigator.clipboard.writeText(selectedLetter.contenu);
    notify({ tone: "success", title: "Lettre copiée" });
  };

  return (
    <Screen padded={false} header={
      <PageHeader
        icon="mail"
        title="Mes lettres de motivation"
        subtitle="Bibliothèque"
        badge={list.data ? <HeaderBadge>{list.data.length} lettre{list.data.length > 1 ? "s" : ""}</HeaderBadge> : undefined}
        primary={<Button variant="primary" icon="auto_awesome" onClick={() => void navigate("/documents/rediger-lettre")}>Rédiger une lettre</Button>}
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
              ) : list.data?.length ? (
                <ul className="space-y-1.5">
                  {list.data.map((letter) => (
                    <li key={letter.id}>
                      <button
                        type="button"
                        aria-pressed={selectedId === letter.id}
                        onClick={() => setSelected(letter.id)}
                        className={`w-full rounded-[10px] border px-3.5 py-3 text-left transition-colors ${selectedId === letter.id ? "border-accent-border bg-accent-tint" : "border-transparent hover:bg-neutral-tint"}`}
                      >
                        <span className="flex items-center gap-2">
                          <Icon name="mail" size={16} className={selectedId === letter.id ? "text-accent" : "text-ink-faint"} />
                          <span className="min-w-0 flex-1 truncate text-[13px] font-semibold">{letter.entreprise ?? letter.nom}</span>
                          <span className="flex-none text-label text-ink-faint">{date(letter.createdAt)}</span>
                        </span>
                        <span className="mt-1 block truncate pl-6 text-label text-ink-faint">{letter.poste ?? "Candidature"} · {libelleTon(letter.ton)}</span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <EmptyState icon="mail" title="Aucune lettre enregistrée" description="Rédigez une lettre puis enregistrez-la ici." action={<Button icon="auto_awesome" onClick={() => void navigate("/documents/rediger-lettre")}>Rédiger une lettre</Button>} />
              )}
            </div>
          </div>
          <div className="flex min-w-0 flex-1 flex-col bg-page">
            <div className="flex flex-none items-center justify-between border-b border-line bg-surface px-5 py-3">
              <p className="truncate text-section">{selectedLetter?.nom ?? "Lecture"}</p>
              {selectedLetter ? (
                <div className="flex gap-2">
                  <Button icon="edit" onClick={() => void navigate("/documents/rediger-lettre", { state: { lettre: selectedLetter } })}>Modifier</Button>
                  <Button icon="content_copy" onClick={() => void copier()}>Copier</Button>
                  <Button variant="danger" icon="delete" onClick={() => setDeleteId(selectedLetter.id)}>Supprimer</Button>
                </div>
              ) : null}
            </div>
            <div className="min-h-0 flex-1 overflow-auto">
              {selectedLetter ? <LetterPreview letter={selectedLetter} /> : <EmptyState icon="draft" title="Sélectionnez une lettre" />}
            </div>
          </div>
        </div>
      <ConfirmDialog open={deleteId !== null} title="Supprimer cette lettre ?" description="La lettre sera retirée de la bibliothèque locale." note="Le profil et les autres documents seront conservés." busy={remove.isPending} onCancel={() => setDeleteId(null)} onConfirm={() => { if (deleteId) remove.mutate(deleteId); }} />
    </Screen>
  );
}

function LetterPreview({ letter }: { letter: Lettre }) { return <A4Preview title={letter.nom}><p className="mt-4 text-[11px] uppercase tracking-[0.12em] text-[#5b6ee1]">Lettre de motivation</p><h2 className="mt-3 text-[23px] font-semibold">{letter.poste ?? "Candidature"}</h2><p className="mt-1 text-[12px] text-[#6a7280]">{letter.entreprise ?? "Entreprise"}</p><div className="mt-8 whitespace-pre-wrap text-[12px] leading-[1.9]">{letter.contenu}</div></A4Preview>; }

export function LetterWriterPage() {
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const location = useLocation();
  const lettreInitiale = lettreDepuisNavigation(location.state);
  const [entreprise, setEntreprise] = useState(lettreInitiale?.entreprise ?? "");
  const [poste, setPoste] = useState(lettreInitiale?.poste ?? "");
  const [ton, setTon] = useState(lettreInitiale?.ton || "formal");
  const [longueur, setLongueur] = useState(lettreInitiale?.longueur || "medium");
  const [contexte, setContexte] = useState("");
  const [output, setOutput] = useState(lettreInitiale?.contenu ?? "");
  const [operation, setOperation] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useIaProgress(operation);

  const run = async () => {
    const id = generationId();
    setOperation(id);
    setOutput("");
    setError(null);
    try {
      setOutput(await iaService.genererLettre({ generationId: id, entreprise: entreprise || null, poste: poste || null, ton, longueur, contexte: contexte || null, lettrePrecedente: null, instruction: null }));
    } catch (e) {
      if (!(e instanceof AppError && e.code === "CANCELLED")) setError(message(e));
    } finally {
      setOperation(null);
    }
  };
  const save = useMutation({
    mutationFn: () => documentsService.enregistrerLettre({ nom: `Lettre — ${poste || entreprise || "Candidature"}`, entreprise: entreprise || null, poste: poste || null, ton, longueur, contenu: output }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: LETTRES_KEY });
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
        primary={output ? <Button variant="primary" icon="save" disabled={save.isPending} onClick={() => save.mutate()}>Enregistrer</Button> : undefined}
      />
    }>
      <div className="grid min-h-[660px] gap-4 xl:grid-cols-[350px_minmax(480px,1fr)]">
        <DocumentPanel title="Brief de rédaction" icon="target">
          <div className="space-y-4 p-4">
            <Champ label="Entreprise" value={entreprise} onChange={setEntreprise} />
            <Champ label="Poste ciblé" value={poste} onChange={setPoste} />
            <div className="grid grid-cols-2 gap-3">
              <FormField label="Ton">{(props) => <Select {...props} value={ton} onChange={(e) => setTon(e.target.value)}><option value="formal">Formel</option><option value="casual">Naturel</option><option value="creative">Créatif</option></Select>}</FormField>
              <FormField label="Longueur">{(props) => <Select {...props} value={longueur} onChange={(e) => setLongueur(e.target.value)}><option value="short">Courte</option><option value="medium">Moyenne</option><option value="long">Longue</option></Select>}</FormField>
            </div>
            <FormField label="Contexte ou offre">{(props) => <TextArea {...props} rows={10} value={contexte} onChange={(e) => setContexte(e.target.value)} />}</FormField>
            {error ? <ErrorBanner title="Rédaction impossible" message={error} /> : null}
            {operation ? (
              <><IaProgress progress={progress} /><Button variant="danger" icon="stop" className="w-full" onClick={() => void iaService.annuler(operation)}>Arrêter</Button></>
            ) : (
              <Button variant="primary" icon="auto_awesome" className="w-full" onClick={() => void run()}>Rédiger la lettre</Button>
            )}
          </div>
        </DocumentPanel>
        <DocumentPanel title="Document" icon="draft">
          <A4Preview title="Lettre de motivation">
            <p className="mt-4 text-[11px] uppercase tracking-[0.12em] text-[#5b6ee1]">Lettre de motivation</p>
            <h2 className="mt-3 text-[23px] font-semibold">{poste || "Candidature"}</h2>
            <p className="mt-1 text-[12px] text-[#6a7280]">{entreprise || "Entreprise ciblée"}</p>
            <div className="mt-8 whitespace-pre-wrap text-[12px] leading-[1.9] text-[#303641]">{output || "La lettre apparaîtra ici après la rédaction."}</div>
          </A4Preview>
        </DocumentPanel>
      </div>
    </Screen>
  );
}

export function CvAnalysisPage() {
  const [path, setPath] = useState<string | null>(null);
  const [offre, setOffre] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [result, setResult] = useState<AnalyseCvImporte | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useIaProgress(operation);
  const choose = async () => {
    const file = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof file === "string") setPath(file);
  };
  const run = async () => {
    if (!path || !offre.trim()) { setError("Sélectionnez un PDF et collez l’offre ciblée."); return; }
    const id = generationId();
    setOperation(id);
    setError(null);
    try {
      setResult(await iaService.analyserCv({ generationId: id, chemin: path, offre }));
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
        primary={<Button variant="primary" icon="bolt" disabled={operation !== null} onClick={() => void run()}>Analyser le CV</Button>}
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
              <FormField label="Offre ciblée" required>{(props) => <TextArea {...props} rows={13} value={offre} onChange={(e) => setOffre(e.target.value)} />}</FormField>
              {operation ? <IaProgress progress={progress} /> : null}
              {error ? <ErrorBanner title="Analyse impossible" message={error} /> : null}
            </div>
          </DocumentPanel>
        </div>
        <div className="space-y-4">
          {result ? (
            <>
              <DocumentPanel title="Résultat" icon="analytics">
                <div className="grid gap-5 p-4 sm:grid-cols-[auto_1fr]"><ScoreBadge value={result.analyse.score} /><p className="text-body leading-relaxed text-ink-muted">{result.analyse.recap}</p></div>
              </DocumentPanel>
              <DocumentPanel title="Recommandations" icon="tips_and_updates">
                <ul className="divide-y divide-line">{result.analyse.suggestions.map((s, i) => <li key={i} className="flex gap-3 px-4 py-3 text-body text-ink-muted"><span className="tabular text-accent">{i + 1}</span>{s}</li>)}</ul>
              </DocumentPanel>
              <DocumentPanel title="Aperçu du CV lu" icon="visibility"><A4Preview cv={result.cv} /></DocumentPanel>
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

function Screen({ header, children, padded = true }: { header: ReactNode; children: ReactNode; padded?: boolean }) {
  return (
    <div className="flex h-full flex-col">
      {header}
      <div className={padded ? "min-h-0 flex-1 overflow-y-auto p-5 min-[1200px]:p-6" : "flex min-h-0 flex-1 flex-col overflow-hidden"}>
        {children}
      </div>
    </div>
  );
}

function HeaderBadge({ children, icon = "auto_awesome" }: { children: ReactNode; icon?: string }) {
  return (
    <span className="inline-flex items-center gap-1 rounded-pill bg-accent-tint px-2.5 py-1 text-label font-medium text-accent">
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
function isGeneration(value: unknown): value is GenerationCv { return typeof value === "object" && value !== null && "cv" in value && "analyse" in value; }
function generationDepuisNavigation(state: unknown): { result: GenerationCv | null; nom: string } {
  if (typeof state !== "object" || state === null || !("generation" in state)) {
    return { result: null, nom: "" };
  }
  const payload = state as { generation?: GenerationCv; nom?: string };
  if (!payload.generation) return { result: null, nom: "" };
  return {
    result: payload.generation,
    nom: payload.nom ?? `CV — ${payload.generation.offre.titre || "Version ciblée"}`,
  };
}
function lettreDepuisNavigation(state: unknown): Lettre | null {
  if (typeof state !== "object" || state === null || !("lettre" in state)) return null;
  const payload = state as { lettre?: Lettre };
  return payload.lettre ?? null;
}
function libelleTon(ton: string): string {
  if (ton === "casual") return "Naturel";
  if (ton === "creative") return "Créatif";
  return "Formel";
}

async function exporterPdf(
  cv: CvGenere,
  nom: string,
  notify: (toast: Omit<ToastMessage, "id">) => void,
) {
  const base = nom.trim().replace(/[\\/:*?"<>|]+/g, "-") || "cv-candilog";
  const chemin = await save({
    title: "Exporter le CV en PDF",
    defaultPath: `${base}.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (chemin === null) return;
  try {
    await documentsService.exporterPdf(cv, chemin);
    notify({ tone: "success", title: "CV exporté" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}
