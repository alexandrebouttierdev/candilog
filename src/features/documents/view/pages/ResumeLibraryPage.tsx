import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { documentsService, type ResumeVersion } from "../../services/documentsService";
import { isResumeWorkspace } from "../../model/resumeWorkspace";
import { useUiStore } from "@/shared/lib/ui-store";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, Icon, PageHeader, Pager } from "@/shared/ui";
import { A4Preview, PreviewAction } from "../components/DocumentUi";
import { ResumePaper } from "../components/ResumePaper";
import { PAGE_SIZE } from "@/shared/types/page";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { AtsChip, HeaderBadge, RESUME_KEY, Screen, date, detail as detailErreur, exportPdf, isLegacyGeneration, message } from "./documentPageSupport";

export function ResumeLibraryPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const notify = useUiStore((s) => s.notify);
  const [page, setPage] = useState(1);
  const [recherche, setRecherche] = useState("");
  const searchQuery = useDebounce(recherche);
  const list = useQuery({
    queryKey: [...RESUME_KEY, "page", { page, search: searchQuery }],
    queryFn: () => documentsService.listResumePage({ page, page_size: PAGE_SIZE, search: searchQuery }),
  });
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const versions = list.data?.items ?? [];
  const selected_id = versions.some((resume) => resume.id === selected)
    ? selected
    : (versions[0]?.id ?? null);
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
    onError: (error) => {
      setDeleteId(null);
      notify({ tone: "error", title: "Suppression impossible", detail: detailErreur(error) });
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
    onError: (error) =>
      notify({ tone: "error", title: "Duplication impossible", detail: detailErreur(error) }),
  });
  const version = detail.data;
  const workspace = version && isResumeWorkspace(version.content) ? version.content : null;
  const generation = version && isLegacyGeneration(version.content) ? version.content : null;
  const atsScore = workspace?.score.total ?? generation?.profile_score.total;

  const exporterHistorique = async () => {
    if (!generation) return;
    try {
      const prepared = await documentsService.prepareResume(generation);
      await exportPdf(prepared.document, notify);
    } catch (error) {
      notify({
        tone: "error",
        title: "Export PDF impossible",
        detail: detailErreur(error),
      });
    }
  };

  return (
    <Screen
      padded={false}
      search={{ value: recherche, onChange: (value) => { setRecherche(value); setPage(1); }, placeholder: "Rechercher un document…" }}
      header={
      <PageHeader
        icon="description"
        title="Mes CV"
        subtitle="Vos versions prêtes à l’emploi"
        badge={list.data ? <HeaderBadge>{list.data.total} version{list.data.total > 1 ? "s" : ""}</HeaderBadge> : undefined}
        secondary={<Button icon="upload_file" onClick={() => void navigate("/documents/analyze")}>Importer</Button>}
        primary={<Button variant="primary" icon="auto_awesome" onClick={() => void navigate("/documents/generate-resume")}>Nouveau CV</Button>}
      />
    }>
      <div className="flex min-h-0 flex-1">
          <div className="flex w-[40%] min-w-[280px] flex-col border-r border-line bg-surface">
            <div className="border-b border-line px-5 pt-4 pb-3">
              <div className="mb-[11px] flex items-center justify-between">
                <span className="text-section">Bibliothèque</span>
                <span className="text-label text-ink-faint">{list.data?.total ?? 0} version{(list.data?.total ?? 0) > 1 ? "s" : ""}</span>
              </div>
              <label className="flex h-8 items-center gap-2 rounded-button border border-line bg-page px-2.5">
                <Icon name="search" size={16} className="text-ink-faint" />
                <input
                  type="search"
                  value={recherche}
                  onChange={(e) => { setRecherche(e.target.value); setPage(1); }}
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
                        className={`flex w-full gap-3 rounded-tile border p-3 text-left transition-colors ${selected_id === resume.id ? "border-accent-border bg-accent-tint" : "border-transparent hover:bg-neutral-tint"}`}
                      >
                        <span className="flex h-[50px] w-[38px] flex-none flex-col gap-[3px] rounded-tag border border-line bg-page px-[5px] py-1.5">
                          <span className={`h-[3px] w-[70%] rounded-sm ${selected_id === resume.id ? "bg-accent" : "bg-accent/40"}`} />
                          <span className="h-[2px] w-full rounded-sm bg-line" />
                          <span className="h-[2px] w-[85%] rounded-sm bg-line" />
                          <span className="h-[2px] w-[95%] rounded-sm bg-line" />
                          <span className="h-[2px] w-[60%] rounded-sm bg-line" />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="mb-[3px] flex items-center gap-2">
                            <span className="truncate text-item font-semibold">{resume.name}</span>
                            {selected_id === resume.id && atsScore !== undefined ? (
                              <AtsChip score={atsScore} />
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
                <EmptyState icon="description" title="Aucune version" description="Générez puis sauvegardez votre premier CV ciblé." action={<Button icon="auto_awesome" onClick={() => void navigate("/documents/generate-resume")}>Générer un CV</Button>} />
              )}
            </div>
            <Pager page={page} page_size={PAGE_SIZE} total={list.data?.total ?? 0} label="versions" dense onPageChange={setPage} />
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
                  {workspace ? (
                    <>
                      <PreviewAction
                        icon="edit"
                        onClick={() =>
                          void navigate("/documents/generate-resume", {
                            state: { workspace, name: version.name },
                          })
                        }
                      >
                        Modifier
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
                        onClick={() => void exportPdf(workspace.document, notify)}
                      >
                        Exporter PDF
                      </PreviewAction>
                    </>
                  ) : null}
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
                        Modifier
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
                        onClick={() => void exporterHistorique()}
                      >
                        Exporter PDF
                      </PreviewAction>
                    </>
                  ) : null}
                  <PreviewAction tone="danger" icon="delete" onClick={() => setDeleteId(version.id)}>
                    Supprimer
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

function ResumeSavedPreview({ version }: { version: ResumeVersion }) {
  if (isResumeWorkspace(version.content)) {
    return (
      <div className="flex min-h-0 flex-1 justify-center overflow-auto bg-page p-[26px]">
        <ResumePaper workspace={version.content} editable={false} onChange={() => {}} />
      </div>
    );
  }
  const generation = isLegacyGeneration(version.content) ? version.content : null;
  return generation ? (
    <A4Preview resume={generation.resume} />
  ) : (
    <A4Preview title={version.name}>
      <div className="flex min-h-[590px] items-center justify-center text-center text-paper-muted">
        Cette ancienne version ne contient pas encore d’aperçu structuré compatible.
      </div>
    </A4Preview>
  );
}
