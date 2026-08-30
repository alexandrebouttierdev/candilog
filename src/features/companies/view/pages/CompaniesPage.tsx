import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useCompaniesViewModel } from "../../viewmodel/useCompaniesViewModel";
import type { Company } from "../../services/companyService";
import { CompanyFormModal } from "../components/CompanyFormModal";
import { CompanyDetail } from "../components/CompanyDetail";
import { CompanyFilters } from "../components/CompanyFilters";
import {
  Button,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  MasterList,
  MasterListItem,
  MasterListTag,
  PageHeader,
  Pager,
  Skeleton,
  wordInitials,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";

/** Écran Relations → Companies : liste maître paginée et fiche détaillée. */
export function CompaniesPage() {
  const vm = useCompaniesViewModel();
  const naviguer = useNavigate();
  const [form, setForm] = useState<{ ouvert: boolean; cible: Company | null }>({
    ouvert: false,
    cible: null,
  });
  const [aDelete, setADelete] = useState<Company | null>(null);
  const aucunResultat = Boolean(vm.search) || vm.filtersActifs > 0;

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="apartment"
        title="Entreprises"
        subtitle="Répertoire des sociétés suivies"
      />

      <CompanyFilters
        search={vm.search}
        onSearch={vm.rechercher}
        company_type={vm.company_type}
        types={vm.types}
        count={vm.filtersActifs}
        total={vm.isLoading ? null : vm.total}
        onSelectType={vm.filtrerParType}
        onReset={vm.resetFilters}
        actions={
          <Button
            variant="primary"
            icon="add"
            onClick={() => setForm({ ouvert: true, cible: null })}
          >
            Nouvelle entreprise
          </Button>
        }
      />

      <div className="flex min-h-0 flex-1">
        <MasterList
          title="Répertoire"
          count={`${vm.total} ${vm.total > 1 ? "entreprises" : "entreprise"}`}
          footer={
            vm.total > 0 ? (
              <Pager
                dense
                page={vm.page}
                page_size={vm.page_size}
                total={vm.total}
                label="entreprises"
                onPageChange={vm.setPage}
              />
            ) : null
          }
        >
          {vm.isLoading ? (
            <ListSquelette />
          ) : vm.error ? (
            <div className="p-3">
              <ErrorBanner
                message={
                  vm.error instanceof AppError
                    ? vm.error.message
                    : "Le répertoire n'a pas pu être chargé."
                }
                onRetry={vm.recharger}
              />
            </div>
          ) : vm.items.length === 0 ? (
            <EmptyState
              icon="apartment"
              title={aucunResultat ? "Aucun résultat" : "Aucune entreprise"}
              description={
                aucunResultat
                  ? "Aucune entreprise ne correspond à ces critères."
                  : "Ajoutez une société pour commencer à suivre vos candidatures."
              }
              action={
                aucunResultat ? (
                  <Button
                    icon="filter_alt_off"
                    onClick={() => {
                      vm.resetFilters();
                      vm.rechercher("");
                    }}
                  >
                    Tout effacer
                  </Button>
                ) : undefined
              }
            />
          ) : (
            vm.items.map((company) => (
              <MasterListItem
                key={company.id}
                initials={wordInitials(company.name)}
                title={company.name}
                subtitle={
                  [company.sector, company.city].filter(Boolean).join(" · ") ||
                  undefined
                }
                meta={
                  company.type ? (
                    <MasterListTag icon="business_center">{company.type}</MasterListTag>
                  ) : undefined
                }
                selected={company.id === vm.selection?.id}
                onSelect={() => vm.selectionner(company.id)}
              />
            ))
          )}
        </MasterList>

        <div className="min-w-0 flex-1">
          {vm.selection ? (
            <CompanyDetail
              company={vm.selection}
              applications={vm.applicationsLiees}
              metrics={vm.companyMetrics}
              onEdit={() => setForm({ ouvert: true, cible: vm.selection })}
              onDelete={() => setADelete(vm.selection)}
              onOuvrirApplication={() => void naviguer("/tracking/applications")}
              onToutVoir={() => void naviguer("/tracking/applications")}
            />
          ) : (
            <div className="flex h-full items-center justify-center">
              <EmptyState
                icon="ads_click"
                title="Aucune entreprise sélectionnée"
                description="Choisissez une société dans la liste pour afficher sa fiche."
              />
            </div>
          )}
        </div>
      </div>

      <CompanyFormModal
        open={form.ouvert}
        company={form.cible}
        busy={vm.isSaving}
        onClose={() => setForm({ ouvert: false, cible: null })}
        onSubmit={(values) =>
          form.cible
            ? vm.update({ id: form.cible.id, input: values })
            : vm.create(values)
        }
      />

      <ConfirmDialog
        open={aDelete !== null}
        title="Supprimer cette entreprise ?"
        description={`« ${aDelete?.name ?? ""} » sera définitivement retirée de votre répertoire. Cette action est irréversible.`}
        note="La suppression est refusée si des candidatures y sont rattachées."
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

/** Squelette de la liste maître, aux dimensions des éléments réels. */
function ListSquelette() {
  return (
    <div role="status" aria-label="Chargement du répertoire">
      {Array.from({ length: 6 }, (_, index) => (
        <div key={index} className="mb-1 flex items-center gap-[11px] px-3 py-[11px]">
          <Skeleton className="size-8 flex-none rounded-field" />
          <div className="flex-1">
            <Skeleton className="h-3 w-2/3" />
            <Skeleton className="mt-1.5 h-2.5 w-1/3" />
          </div>
        </div>
      ))}
    </div>
  );
}
