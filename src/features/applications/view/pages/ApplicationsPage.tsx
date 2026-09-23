import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useApplicationsViewModel } from "../../viewmodel/useApplicationsViewModel";
import type { TrackingView } from "../../viewmodel/useApplicationsViewModel";
import type { Application, ApplicationStatus } from "@/shared/types/generated/applications";
import { status_meta } from "../../model/statuses";
import { applicationTypeLabel, weeklyDurationLabel } from "@/features/referentials";
import { toDisplayDate } from "@/shared/lib/dates";
import { ApplicationFormModal } from "../components/ApplicationFormModal";
import { ApplicationFilters } from "../components/ApplicationFilters";
import { ApplicationDetail } from "../components/ApplicationDetail";
import { KanbanBoard } from "../components/KanbanBoard";
import {
  Button,
  CellIdentity,
  ConfirmDialog,
  DataTable,
  EmptyState,
  ErrorBanner,
  Pager,
  SegmentedControl,
  SkeletonRows,
  StatusPill,
} from "@/shared/ui";
import type { Column } from "@/shared/ui";
import type { ApplicationSort } from "@/shared/types/generated/applications";
import { AppError } from "@/shared/types/app-error";
import { PAGE_SIZE } from "@/shared/types/page";
import { EMPTY_FILTER } from "../../model/schemas/application-filter.schema";

/** Densités proposées par le pied de la vue Liste. */
const DENSITIES = [PAGE_SIZE, 25, 50] as const;

/** Écran Suivi → Candidatures : Kanban ou Liste, sur le même filtre. */
export function ApplicationsPage({ view }: { view?: TrackingView } = {}) {
  const vm = useApplicationsViewModel(view);
  const [searchParams, setSearchParams] = useSearchParams();
  const [form, setForm] = useState<{
    isOpen: boolean;
    editing: Application | null;
    status: ApplicationStatus | null;
  }>({
    isOpen: searchParams.get("new") === "1",
    editing: null,
    status: null,
  });
  const [pendingDelete, setPendingDelete] = useState<string[] | null>(null);
  const [checkedIds, setCheckedIds] = useState<Set<string>>(() => new Set());

  // Le bouton principal du Dashboard ouvre réellement la création, sans dupliquer le
  // formulaire ni son ViewModel dans une autre feature. Le paramètre reste dans l'URL le
  // temps de la modale, puis est consommé à sa fermeture.
  const closeForm = () => {
    setForm({ isOpen: false, editing: null, status: null });
    if (searchParams.get("new") === "1") {
      // Seul `new` est consommé : effacer toute la query effacerait aussi la fiche
      // ouverte dans le panneau de détail.
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current);
          next.delete("new");
          return next;
        },
        { replace: true },
      );
    }
  };

  /**
   * Exporte le filtre courant, ou uniquement les lignes cochées.
   *
   * Le sélecteur et l'écriture appartiennent entièrement à la commande Rust native.
   */
  const exportRows = async () => {
    const ids = [...checkedIds];
    // Les identifiants cochés suffisent : les combiner au filtre courant exclurait
    // une ligne sélectionnée puis masquée par une recherche ou un status.
    const filter =
      ids.length > 0
        ? { ...EMPTY_FILTER, sort: vm.sort, descending: vm.descending, search: "", ids }
        : vm.filter;
    await vm.exportCsv(filter);
  };

  const toggleChecked = (id: string) => {
    setCheckedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const basculerPage = (ids: readonly string[], checked: boolean) => {
    setCheckedIds((current) => {
      const next = new Set(current);
      for (const id of ids) {
        if (checked) next.add(id);
        else next.delete(id);
      }
      return next;
    });
  };

  const suppressionEnCours = useMemo(() => {
    if (!pendingDelete || pendingDelete.length === 0) return null;
    const unique =
      pendingDelete.length === 1
        ? (vm.selection?.id === pendingDelete[0]
            ? vm.selection
            : vm.items.find((item) => item.id === pendingDelete[0]))
        : undefined;
    return { ids: pendingDelete, unique };
  }, [pendingDelete, vm.items, vm.selection]);

  const columns: Column<Application, ApplicationSort>[] = [
    {
      key: "job_title",
      header: "Poste",
      sort_key: "job_title",
      grow: 2.2,
      render: (row) => (
        <CellIdentity
          initials={initials(row.company_name ?? row.job_title)}
          title={row.job_title}
          subtitle={row.professional_domain_name ?? undefined}
        />
      ),
    },
    {
      key: "company",
      header: "Entreprise",
      sort_key: "company",
      grow: 1.3,
      render: (row) => (
        <span className="truncate text-body text-ink-muted">{row.company_name ?? "—"}</span>
      ),
    },
    {
      key: "ville",
      header: "Ville",
      grow: 0.9,
      render: (row) => (
        <span className="truncate text-note text-ink-faint">{row.effective_city ?? "—"}</span>
      ),
    },
    {
      key: "contrat",
      header: "Contrat",
      grow: 0.9,
      render: (row) => (
        <span className="text-note text-ink-faint">
          {row.contract_type_name ?? row.contract_type_code}
        </span>
      ),
    },
    {
      key: "duree",
      header: "Durée",
      grow: 1,
      render: (row) => (
        <span className="truncate text-note text-ink-faint">
          {weeklyDurationLabel(row.weekly_work_schedule, row.weekly_hours)}
        </span>
      ),
    },
    {
      key: "candidature",
      header: "Type",
      grow: 0.9,
      render: (row) => (
        <span className="truncate text-note text-ink-faint">
          {applicationTypeLabel(row.application_type)}
        </span>
      ),
    },
    {
      key: "status",
      header: "Statut",
      sort_key: "status",
      grow: 1.1,
      render: (row) => {
        const status = status_meta(row.status);
        return (
          <StatusPill tone={status.tone} icon={status.icon}>
            {status.label}
          </StatusPill>
        );
      },
    },
    {
      key: "date",
      header: "Envoyée",
      sort_key: "date",
      grow: 0.7,
      numeric: true,
      render: (row) => (
        <span className="text-note text-ink-faint">{toDisplayDate(row.sent_date)}</span>
      ),
    },
  ];

  const detail = vm.selection;

  return (
    <div className="flex h-full flex-col">
      <ApplicationFilters
        search={vm.search}
        onSearch={vm.setSearch}
        filters={vm.filters}
        count={vm.activeFilterCount}
        total={vm.isLoading ? null : vm.total}
        onApply={vm.applyFilters}
        onReset={vm.resetFilters}
        actions={
          <>
            {/* Sous la coque v2, la vue se choisit dans les onglets de la barre de titre. */}
            {view ? null : (
              <SegmentedControl
                label="Mode d'affichage"
                value={vm.view}
                onChange={vm.setView}
                options={[
                  { value: "kanban", label: "Kanban", icon: "view_kanban" },
                  { value: "list", label: "Liste", icon: "view_list" },
                ]}
              />
            )}
            {checkedIds.size > 0 ? (
              <>
                <span className="text-note font-semibold text-ink">
                  {checkedIds.size} sélectionnée{checkedIds.size > 1 ? "s" : ""}
                </span>
                <Button variant="ghost" onClick={() => setCheckedIds(new Set())}>
                  Tout désélectionner
                </Button>
                <Button
                  variant="danger"
                  icon="delete"
                  onClick={() => setPendingDelete([...checkedIds])}
                >
                  Supprimer
                </Button>
              </>
            ) : null}
            <Button icon="download" disabled={vm.isExporting} onClick={() => void exportRows()}>
              Exporter
            </Button>
            <Button
              variant="primary"
              icon="add"
              onClick={() => setForm({ isOpen: true, editing: null, status: null })}
            >
              Nouvelle
            </Button>
          </>
        }
      />

      <div className="flex min-h-0 flex-1">
        <div className="flex min-w-0 flex-1 flex-col">
          {vm.error ? (
            <div className="px-7 pt-[18px]">
              <ErrorBanner
                message={
                  vm.error instanceof AppError
                    ? vm.error.message
                    : "Les candidatures n'ont pas pu être chargées."
                }
                onRetry={vm.reload}
              />
            </div>
          ) : vm.isLoading ? (
            <div className="px-7 pt-[18px]">
              <div className="overflow-hidden rounded-card border border-line bg-surface">
                <SkeletonRows rows={6} columns={5} />
              </div>
            </div>
          ) : vm.total === 0 ? (
            <div className="px-7 pt-[18px]">
              <EmptyState
                bordered
                icon="work"
                title={vm.search || vm.activeFilterCount > 0 ? "Aucun résultat" : "Aucune candidature"}
                description={
                  vm.search || vm.activeFilterCount > 0
                    ? "Aucune candidature ne correspond à ces critères."
                    : "Créez votre première candidature pour lancer le suivi."
                }
                action={
                  vm.search || vm.activeFilterCount > 0 ? (
                    <Button
                      icon="filter_alt_off"
                      onClick={() => {
                        vm.resetFilters();
                        vm.setSearch("");
                      }}
                    >
                      Tout effacer
                    </Button>
                  ) : (
                    <Button
                      variant="primary"
                      icon="add"
                      onClick={() => setForm({ isOpen: true, editing: null, status: null })}
                    >
                      Nouvelle candidature
                    </Button>
                  )
                }
              />
            </div>
          ) : vm.view === "kanban" ? (
            <KanbanBoard
              columns={vm.kanbanColumns}
              selected_id={vm.selected_id}
              checkedIds={checkedIds}
              onSelect={vm.select}
              onToggleSelect={toggleChecked}
              onStatusChange={(id, status) => void vm.changeStatus({ id, status })}
              onCreate={(status) => setForm({ isOpen: true, editing: null, status })}
              onPageChange={vm.setKanbanPage}
            />
          ) : (
            <div className="min-h-0 flex-1 overflow-auto px-4 pt-3 pb-5">
              <DataTable
                columns={columns}
                rows={vm.items}
                row_key={(row) => row.id}
                sort={{ key: vm.sort, direction: vm.descending ? "desc" : "asc" }}
                onSortChange={vm.sortBy}
                onRowClick={(row) => vm.select(row.id)}
                isSelected={(row) => row.id === vm.selected_id}
                selection={{
                  selected: checkedIds,
                  onToggle: toggleChecked,
                  onTogglePage: basculerPage,
                  rowLabel: "Sélectionner cette candidature",
                  pageLabel: "Sélectionner les candidatures de la page",
                }}
                footer={
                  <Pager
                    page={vm.page}
                    page_size={vm.page_size}
                    total={vm.total}
                    label="candidatures"
                    pageSizes={DENSITIES}
                    onPageChange={vm.setPage}
                    onPageSizeChange={vm.setPageSize}
                  />
                }
              />
            </div>
          )}
        </div>

        {detail ? (
          <ApplicationDetail
            application={detail}
            onClose={() => vm.select(null)}
            onEdit={() => setForm({ isOpen: true, editing: detail, status: null })}
            onDelete={() => setPendingDelete([detail.id])}
            onStatusChange={(status) => void vm.changeStatus({ id: detail.id, status })}
          />
        ) : null}
      </div>

      <ApplicationFormModal
        open={form.isOpen}
        application={form.editing}
        defaultStatus={form.status}
        busy={vm.isSaving}
        onClose={closeForm}
        onSubmit={(values) =>
          form.editing
            ? vm.update({ id: form.editing.id, input: values })
            : vm.create(values)
        }
      />

      <ConfirmDialog
        open={suppressionEnCours !== null}
        title={
          suppressionEnCours && suppressionEnCours.ids.length > 1
            ? `Supprimer ${suppressionEnCours.ids.length} candidatures ?`
            : "Supprimer cette candidature ?"
        }
        description={
          suppressionEnCours && suppressionEnCours.ids.length > 1
            ? "Les candidatures sélectionnées seront définitivement supprimées, ainsi que les entretiens et relances rattachés."
            : suppressionEnCours?.unique
              ? `« ${suppressionEnCours.unique.job_title} » chez ${suppressionEnCours.unique.company_name ?? "cette entreprise"} sera définitivement supprimée, ainsi que les entretiens et relances rattachés.`
              : "Cette candidature sera définitivement supprimée, ainsi que les entretiens et relances rattachés."
        }
        note="L'entreprise et le contact associés sont conservés."
        busy={vm.isDeleting}
        onCancel={() => setPendingDelete(null)}
        onConfirm={() => {
          const ids = pendingDelete;
          setPendingDelete(null);
          if (!ids || ids.length === 0) return;
          void (ids.length === 1 ? vm.delete(ids[0]!) : vm.deleteMany(ids)).then(() => {
            setCheckedIds((current) => {
              const next = new Set(current);
              for (const id of ids) next.delete(id);
              return next;
            });
          });
        }}
      />
    </div>
  );
}

/** Initials de l'entreprise, pour la pastille de la colonne « Poste ». */
function initials(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}
