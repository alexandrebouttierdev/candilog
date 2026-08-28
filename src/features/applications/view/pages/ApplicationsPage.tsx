import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { save } from "@tauri-apps/plugin-dialog";
import { useApplicationsViewModel } from "../../viewmodel/useApplicationsViewModel";
import type { Application } from "../../services/applicationService";
import { applicationService } from "../../services/applicationService";
import { contract_label, status_meta } from "../../model/statuses";
import { versDateAffichee } from "@/shared/lib/dates";
import { ApplicationFormModal } from "../components/ApplicationFormModal";
import { ApplicationFilters } from "../components/ApplicationFilters";
import { ApplicationDetail } from "../components/ApplicationDetail";
import { KanbanBoard } from "../components/KanbanBoard";
import { ContextBarAccessory, ContextSearch } from "@/app/layout/ContextBar";
import {
  Button,
  CellIdentity,
  ConfirmDialog,
  DataTable,
  EmptyState,
  ErrorBanner,
  FilterBar,
  FilterChip,
  PageHeader,
  Pager,
  SegmentedControl,
  SkeletonRows,
  StatusPill,
} from "@/shared/ui";
import type { Column } from "@/shared/ui";
import type { ApplicationSort } from "@/shared/types/generated/applications";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { PAGE_SIZE } from "@/shared/types/page";

/** Densités proposées par le pied de la vue List. */
const DENSITES = [PAGE_SIZE, 25, 50] as const;

/** Écran Tracking → Applications : Kanban ou List, sur le même filtre. */
export function ApplicationsPage() {
  const vm = useApplicationsViewModel();
  const [searchParams, setSearchParams] = useSearchParams();
  const notify = useUiStore((state) => state.notify);
  const [form, setForm] = useState<{ ouvert: boolean; cible: Application | null }>({
    ouvert: searchParams.get("nouvelle") === "1",
    cible: null,
  });
  const [filtersOuverts, setFiltersOuverts] = useState(false);
  const [aDelete, setADelete] = useState<Application | null>(null);
  const [exportEnCours, setExportEnCours] = useState(false);

  // Le bouton principal du Dashboard ouvre réellement la création, sans dupliquer le
  // formulaire ni son ViewModel dans une autre feature. Le paramètre reste dans l'URL le
  // temps de la modale, puis est consommé à sa fermeture.
  const fermerForm = () => {
    setForm({ ouvert: false, cible: null });
    if (searchParams.get("nouvelle") === "1") {
      setSearchParams({}, { replace: true });
    }
  };

  /**
   * Exporte le filtre courant.
   *
   * La destination est choisie dans le sélecteur natif : la fenêtre n'a aucune permission
   * d'écriture, et la commande Rust n'écrit qu'au chemin que l'utilisateur désigne (§44).
   */
  const exporter = async () => {
    const path = await save({
      title: "Exporter les candidatures",
      defaultPath: "candidatures.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (path === null) return;

    setExportEnCours(true);
    try {
      const rows = await applicationService.exportCsv(vm.filter, path);
      notify({
        tone: "success",
        title: "Export terminé",
        detail: `${rows} candidature${rows > 1 ? "s" : ""} exportée${rows > 1 ? "s" : ""}.`,
      });
    } catch (error) {
      notify({
        tone: "error",
        title: "Export impossible",
        detail: error instanceof AppError ? error.message : undefined,
      });
    } finally {
      setExportEnCours(false);
    }
  };

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
          subtitle={row.company_city ?? undefined}
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
      key: "contrat",
      header: "Contrat",
      grow: 0.9,
      render: (row) => <span className="text-note text-ink-faint">{contract_label(row.contract_type)}</span>,
    },
    {
      key: "statut",
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
        <span className="text-note text-ink-faint">{versDateAffichee(row.sent_date)}</span>
      ),
    },
  ];

  const filters = vm.filters;

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextSearch
          value={vm.search}
          onChange={vm.rechercher}
          placeholder="Rechercher un poste, une entreprise…"
          width={250}
        />
      </ContextBarAccessory>

      <PageHeader
        icon="work"
        title="Candidatures"
        subtitle="Suivi de vos dossiers"
        toolbar={
          <SegmentedControl
            label="Mode d’affichage"
            value={vm.view}
            onChange={vm.setView}
            options={[
              { value: "kanban", label: "Kanban", icon: "view_kanban" },
              { value: "liste", label: "Liste", icon: "view_list" },
            ]}
          />
        }
        secondary={
          <Button icon="download" disabled={exportEnCours} onClick={() => void exporter()}>
            Export
          </Button>
        }
        primary={
          <Button
            variant="primary"
            icon="add"
            onClick={() => setForm({ ouvert: true, cible: null })}
          >
            Nouvelle candidature
          </Button>
        }
      />

      <FilterBar
        summary={
          vm.isLoading
            ? null
            : `${vm.total} candidature${vm.total > 1 ? "s" : ""}${
                vm.filtersActifs > 0 ? " · filtrées" : ""
              }`
        }
      >
        <FilterChip
          icon="filter_alt"
          label={filters.status ? status_meta(filters.status).label : "Statut"}
          active={filters.status !== null}
          onClick={() => setFiltersOuverts(true)}
        />
        <FilterChip
          icon="badge"
          label={filters.contract ? contract_label(filters.contract) : "Contrat"}
          active={filters.contract !== null}
          onClick={() => setFiltersOuverts(true)}
        />
        <FilterChip
          icon="apartment"
          label={filters.city || "Ville"}
          active={filters.city !== ""}
          onClick={() => setFiltersOuverts(true)}
        />
        <FilterChip
          icon="date_range"
          label={periodLabel(filters.start_date, filters.end_date)}
          active={filters.start_date !== null || filters.end_date !== null}
          onClick={() => setFiltersOuverts(true)}
        />
        {vm.filtersActifs > 0 ? (
          <Button variant="ghost" icon="filter_alt_off" onClick={vm.resetFilters}>
            Effacer
          </Button>
        ) : null}
      </FilterBar>

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
                onRetry={vm.recharger}
              />
            </div>
          ) : vm.isLoading ? (
            <div className="px-7 pt-[18px]">
              <div className="overflow-hidden rounded-card border border-line bg-surface shadow-e1">
                <SkeletonRows rows={6} columns={5} />
              </div>
            </div>
          ) : vm.total === 0 ? (
            <div className="px-7 pt-[18px]">
              <EmptyState
                bordered
                icon="work"
                title={vm.search || vm.filtersActifs > 0 ? "Aucun résultat" : "Aucune candidature"}
                description={
                  vm.search || vm.filtersActifs > 0
                    ? "Aucune candidature ne correspond à ces critères."
                    : "Créez votre première candidature pour lancer le suivi."
                }
                action={
                  vm.search || vm.filtersActifs > 0 ? (
                    <Button icon="filter_alt_off" onClick={vm.resetFilters}>
                      Effacer les filters
                    </Button>
                  ) : (
                    <Button
                      variant="primary"
                      icon="add"
                      onClick={() => setForm({ ouvert: true, cible: null })}
                    >
                      Nouvelle candidature
                    </Button>
                  )
                }
              />
            </div>
          ) : vm.view === "kanban" ? (
            <KanbanBoard
              applications={vm.items}
              breakdown={vm.breakdown}
              selected_id={vm.selected_id}
              onSelect={vm.selectionner}
              onStatusChange={(id, status) => void vm.changeStatus({ id, status })}
              onCreate={() => setForm({ ouvert: true, cible: null })}
            />
          ) : (
            <div className="min-h-0 flex-1 overflow-auto px-7 pt-[18px] pb-[26px]">
              <DataTable
                columns={columns}
                rows={vm.items}
                row_key={(row) => row.id}
                sort={{ key: vm.sort, direction: vm.descending ? "desc" : "asc" }}
                onSortChange={vm.trierPar}
                onRowClick={(row) => vm.selectionner(row.id)}
                isSelected={(row) => row.id === vm.selected_id}
                footer={
                  <Pager
                    page={vm.page}
                    page_size={vm.page_size}
                    total={vm.total}
                    label="candidatures"
                    pageSizes={DENSITES}
                    onPageChange={vm.setPage}
                    onPageSizeChange={vm.setPageSize}
                  />
                }
              />
            </div>
          )}
        </div>

        {vm.selection ? (
          <ApplicationDetail
            application={vm.selection}
            onClose={() => vm.selectionner(null)}
            onEdit={() => setForm({ ouvert: true, cible: vm.selection })}
            onDelete={() => setADelete(vm.selection)}
            onStatusChange={(status) => void vm.changeStatus({ id: vm.selection!.id, status })}
          />
        ) : null}
      </div>

      <ApplicationFormModal
        open={form.ouvert}
        application={form.cible}
        busy={vm.isSaving}
        onClose={fermerForm}
        onSubmit={(values) =>
          form.cible
            ? vm.update({ id: form.cible.id, input: values })
            : vm.create(values)
        }
      />

      <ApplicationFilters
        open={filtersOuverts}
        filters={vm.filters}
        onClose={() => setFiltersOuverts(false)}
        onApply={vm.appliquerFilters}
        onReset={vm.resetFilters}
      />

      <ConfirmDialog
        open={aDelete !== null}
        title="Supprimer cette candidature ?"
        description={`« ${aDelete?.job_title ?? ""} » chez ${aDelete?.company_name ?? "cette entreprise"} sera définitivement supprimée, ainsi que les entretiens et relances rattachés.`}
        note="L'entreprise et le contact associés sont conservés."
        busy={vm.isDeleting}
        onCancel={() => setADelete(null)}
        onConfirm={() => {
          const cible = aDelete;
          setADelete(null);
          if (cible) void vm.delete(cible.id);
        }}
      />
    </div>
  );
}

/** Libellé de la puce de période : la borne renseignée, ou les deux. */
function periodLabel(start: string | null, end: string | null): string {
  if (start && end) return `${versDateAffichee(start)} → ${versDateAffichee(end)}`;
  if (start) return `Depuis le ${versDateAffichee(start)}`;
  if (end) return `Jusqu’au ${versDateAffichee(end)}`;
  return "Période";
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
