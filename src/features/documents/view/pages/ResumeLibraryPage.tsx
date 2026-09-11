import { useNavigate } from "react-router-dom";
import type { ResumeVersion } from "@/shared/types/generated/documents";
import { normalizeResumeWorkspace } from "../../model/resumeWorkspace";
import { useResumeLibraryViewModel } from "../../viewmodel/useResumeLibraryViewModel";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, Icon, PageHeader, Pager } from "@/shared/ui";
import { A4Preview, PreviewAction } from "../components/DocumentUi";
import { ResumePaper } from "../components/ResumePaper";
import { useProfilePhoto } from "@/features/profile";
import { AtsChip, HeaderBadge, Screen, date, isLegacyGeneration, message } from "./documentPageSupport";

export function ResumeLibraryPage() {
  const navigate = useNavigate();
  const vm = useResumeLibraryViewModel();

  return (
    <Screen
      padded={false}
      header={
        <PageHeader
          icon="description"
          title="Mes CV"
          subtitle="Vos versions prêtes à l’emploi"
          badge={
            vm.list.data ? (
              <HeaderBadge>
                {vm.list.data.total} version{vm.list.data.total > 1 ? "s" : ""}
              </HeaderBadge>
            ) : undefined
          }
          secondary={
            <Button icon="upload_file" onClick={() => void navigate("/documents/analyze")}>
              Importer
            </Button>
          }
          primary={
            <Button
              variant="primary"
              icon="auto_awesome"
              onClick={() => void navigate("/documents/generate-resume")}
            >
              Nouveau CV
            </Button>
          }
        />
      }
    >
      <div className="flex min-h-0 flex-1">
        <div className="flex w-[40%] min-w-[280px] flex-col border-r border-line bg-surface">
          <div className="border-b border-line px-5 pt-4 pb-3">
            <div className="mb-[11px] flex items-center justify-between">
              <span className="text-section">Bibliothèque</span>
              <span className="text-label text-ink-faint">
                {vm.list.data?.total ?? 0} version{(vm.list.data?.total ?? 0) > 1 ? "s" : ""}
              </span>
            </div>
            <label className="flex h-8 items-center gap-2 rounded-button border border-line bg-page px-2.5">
              <Icon name="search" size={16} className="text-ink-faint" />
              <input
                type="search"
                value={vm.search}
                onChange={(e) => vm.updateSearch(e.target.value)}
                placeholder="Rechercher une version…"
                className="min-w-0 flex-1 bg-transparent text-body text-ink outline-none placeholder:text-ink-faint"
              />
            </label>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto p-2.5">
            {vm.list.error ? (
              <ErrorBanner message={message(vm.list.error)} onRetry={() => void vm.list.refetch()} />
            ) : vm.list.isLoading ? (
              <p className="p-6 text-center text-ink-muted">Chargement…</p>
            ) : vm.versions.length ? (
              <ul className="space-y-1.5">
                {vm.versions.map((resume) => (
                  <li key={resume.id}>
                    <button
                      type="button"
                      aria-pressed={vm.selectedId === resume.id}
                      onClick={() => vm.setSelected(resume.id)}
                      className={`flex w-full gap-3 rounded-tile border p-3 text-left transition-colors ${
                        vm.selectedId === resume.id
                          ? "border-accent-border bg-accent-tint"
                          : "border-transparent hover:bg-neutral-tint"
                      }`}
                    >
                      <span className="flex h-[50px] w-[38px] flex-none flex-col gap-[3px] rounded-tag border border-line bg-page px-[5px] py-1.5">
                        <span
                          className={`h-[3px] w-[70%] rounded-sm ${
                            vm.selectedId === resume.id ? "bg-accent" : "bg-accent/40"
                          }`}
                        />
                        <span className="h-[2px] w-full rounded-sm bg-line" />
                        <span className="h-[2px] w-[85%] rounded-sm bg-line" />
                        <span className="h-[2px] w-[95%] rounded-sm bg-line" />
                        <span className="h-[2px] w-[60%] rounded-sm bg-line" />
                      </span>
                      <span className="min-w-0 flex-1">
                        <span className="mb-[3px] flex items-center gap-2">
                          <span className="truncate text-item font-semibold">{resume.name}</span>
                          {vm.selectedId === resume.id && vm.atsScore !== undefined ? (
                            <AtsChip score={vm.atsScore} />
                          ) : null}
                        </span>
                        <span className="block text-label text-ink-faint">{date(resume.created_at)}</span>
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : vm.search.trim() ? (
              <EmptyState
                icon="search"
                title="Aucun résultat"
                description="Aucune version ne correspond à cette recherche."
                action={
                  <Button icon="filter_alt_off" onClick={() => vm.updateSearch("")}>
                    Tout effacer
                  </Button>
                }
              />
            ) : (
              <EmptyState
                icon="description"
                title="Aucune version"
                description="Générez puis sauvegardez votre premier CV ciblé."
                action={
                  <Button
                    icon="auto_awesome"
                    onClick={() => void navigate("/documents/generate-resume")}
                  >
                    Générer un CV
                  </Button>
                }
              />
            )}
          </div>
          <Pager
            page={vm.page}
            page_size={vm.pageSize}
            total={vm.list.data?.total ?? 0}
            label="versions"
            dense
            onPageChange={vm.setPage}
          />
        </div>
        <div className="flex min-w-0 flex-1 flex-col bg-page">
          <div className="flex flex-none items-center justify-between gap-3 border-b border-line bg-surface px-4 py-3">
            <div className="flex min-w-0 items-center gap-[9px]">
              <Icon name="visibility" size={17} className="text-ink-faint" />
              <p className="truncate text-body font-mid">Aperçu</p>
            </div>
            {vm.version ? (
              <div className="flex flex-none items-center gap-1.5">
                {vm.workspace ? (
                  <>
                    <PreviewAction
                      icon="edit"
                      onClick={() =>
                        void navigate("/documents/generate-resume", {
                          state: { workspace: vm.workspace, name: vm.version?.name },
                        })
                      }
                    >
                      Modifier
                    </PreviewAction>
                    <PreviewAction
                      icon="content_copy"
                      label="Dupliquer"
                      compact
                      disabled={vm.isDuplicating}
                      onClick={() => vm.duplicate()}
                    >
                      Dupliquer
                    </PreviewAction>
                    <PreviewAction
                      icon="download"
                      label="Exporter PDF"
                      compact
                      onClick={() => void vm.exportWorkspacePdf()}
                    >
                      PDF
                    </PreviewAction>
                  </>
                ) : null}
                {vm.generation ? (
                  <>
                    <PreviewAction
                      icon="edit"
                      onClick={() =>
                        void navigate("/documents/generate-resume", {
                          state: { generation: vm.generation, name: vm.version?.name },
                        })
                      }
                    >
                      Modifier
                    </PreviewAction>
                    <PreviewAction
                      icon="content_copy"
                      label="Dupliquer"
                      compact
                      disabled={vm.isDuplicating}
                      onClick={() => vm.duplicate()}
                    >
                      Dupliquer
                    </PreviewAction>
                    <PreviewAction
                      icon="download"
                      label="Exporter PDF"
                      compact
                      onClick={() => void vm.exportLegacyPdf()}
                    >
                      PDF
                    </PreviewAction>
                  </>
                ) : null}
                <PreviewAction
                  tone="danger"
                  icon="delete"
                  label="Supprimer"
                  compact
                  onClick={() => vm.setDeleteId(vm.version?.id ?? null)}
                >
                  Supprimer
                </PreviewAction>
              </div>
            ) : null}
          </div>
          <div className="min-h-0 flex-1 overflow-auto">
            {vm.detail.data ? (
              <ResumeSavedPreview version={vm.detail.data} />
            ) : (
              <EmptyState
                icon="visibility"
                title="Sélectionnez une version"
                description="Son contenu détaillé apparaîtra ici."
              />
            )}
          </div>
        </div>
      </div>
      <ConfirmDialog
        open={vm.deleteId !== null}
        title="Supprimer cette version ?"
        description="Le CV disparaîtra définitivement de la bibliothèque locale."
        note="Votre profil et vos autres versions seront conservés."
        busy={vm.isDeleting}
        onCancel={() => vm.setDeleteId(null)}
        onConfirm={() => {
          if (vm.deleteId) vm.remove(vm.deleteId);
        }}
      />
    </Screen>
  );
}

function ResumeSavedPreview({ version }: { version: ResumeVersion }) {
  // La photo suit le profil courant, comme à l'export PDF : une version enregistrée avant
  // son ajout l'affiche donc, et une version rouverte après sa suppression ne l'affiche plus.
  const photo = useProfilePhoto().data ?? null;

  const workspace = normalizeResumeWorkspace(version.content);
  if (workspace) {
    return (
      <div className="flex min-h-0 flex-1 justify-center overflow-auto bg-page p-[26px]">
        <ResumePaper workspace={workspace} editable={false} onChange={() => {}} photo={photo} />
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
