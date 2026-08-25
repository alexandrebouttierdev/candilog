import { useState } from "react";
import { useEntreprisesViewModel } from "../../viewmodel/useEntreprisesViewModel";
import type { Entreprise } from "../../services/entreprise.service";
import { EntrepriseFormModal } from "../components/EntrepriseFormModal";
import { EntrepriseDetail } from "../components/EntrepriseDetail";
import {
  Button,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  MasterList,
  MasterListItem,
  PageHeader,
  Pager,
  Select,
  Skeleton,
  initiales,
} from "@/shared/ui";
import { AppError } from "@/shared/types/app-error";

/** Écran Relations → Entreprises : liste maître paginée et fiche détaillée. */
export function EntreprisesPage() {
  const vm = useEntreprisesViewModel();
  const [formulaire, setFormulaire] = useState<{ ouvert: boolean; cible: Entreprise | null }>({
    ouvert: false,
    cible: null,
  });
  const [aSupprimer, setASupprimer] = useState<Entreprise | null>(null);

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="apartment"
        title="Entreprises"
        subtitle="Répertoire des sociétés que vous suivez"
        primary={
          <Button
            variant="primary"
            icon="add"
            onClick={() => setFormulaire({ ouvert: true, cible: null })}
          >
            Nouvelle entreprise
          </Button>
        }
      />

      <div className="flex min-h-0 flex-1">
        <MasterList
          search={vm.search}
          searchPlaceholder="Rechercher une entreprise…"
          onSearchChange={vm.rechercher}
          toolbar={
            vm.types.length > 0 ? (
              <Select
                aria-label="Filtrer par type"
                value={vm.companyType ?? ""}
                onChange={(event) => vm.filtrerParType(event.target.value || null)}
              >
                <option value="">Tous les types</option>
                {vm.types.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </Select>
            ) : null
          }
          footer={
            vm.total > 0 ? (
              <Pager
                page={vm.page}
                pageSize={vm.pageSize}
                total={vm.total}
                label="entreprises"
                onPageChange={vm.setPage}
              />
            ) : null
          }
        >
          {vm.isLoading ? (
            <ListeSquelette />
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
              title={vm.search ? "Aucun résultat" : "Aucune entreprise"}
              description={
                vm.search
                  ? "Aucune entreprise ne correspond à cette recherche."
                  : "Ajoutez une société pour commencer à suivre vos candidatures."
              }
            />
          ) : (
            vm.items.map((entreprise) => (
              <MasterListItem
                key={entreprise.id}
                initials={initiales(entreprise.nom)}
                title={entreprise.nom}
                subtitle={
                  [entreprise.secteur, entreprise.ville].filter(Boolean).join(" · ") ||
                  undefined
                }
                selected={entreprise.id === vm.selectedId}
                onSelect={() => vm.selectionner(entreprise.id)}
              />
            ))
          )}
        </MasterList>

        <div className="min-w-0 flex-1">
          {vm.selection ? (
            <EntrepriseDetail
              entreprise={vm.selection}
              onEdit={() => setFormulaire({ ouvert: true, cible: vm.selection })}
              onDelete={() => setASupprimer(vm.selection)}
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

      <EntrepriseFormModal
        open={formulaire.ouvert}
        entreprise={formulaire.cible}
        busy={vm.isSaving}
        onClose={() => setFormulaire({ ouvert: false, cible: null })}
        onSubmit={(valeurs) =>
          formulaire.cible
            ? vm.modifier({ id: formulaire.cible.id, input: valeurs })
            : vm.creer(valeurs)
        }
      />

      <ConfirmDialog
        open={aSupprimer !== null}
        title="Supprimer cette entreprise ?"
        description={`« ${aSupprimer?.nom ?? ""} » sera définitivement retirée de votre répertoire. Cette action est irréversible.`}
        note="La suppression est refusée si des candidatures y sont rattachées."
        busy={vm.isDeleting}
        onCancel={() => setASupprimer(null)}
        onConfirm={() => {
          const cible = aSupprimer;
          setASupprimer(null);
          if (cible) void vm.supprimer(cible.id);
        }}
      />
    </div>
  );
}

/** Squelette de la liste maître, aux dimensions des éléments réels. */
function ListeSquelette() {
  return (
    <div role="status" aria-label="Chargement du répertoire">
      {Array.from({ length: 6 }, (_, index) => (
        <div key={index} className="flex min-h-row items-center gap-2.5 border-b border-line px-3">
          <Skeleton className="size-8 flex-none rounded-pill" />
          <div className="flex-1">
            <Skeleton className="h-3 w-2/3" />
            <Skeleton className="mt-1.5 h-2.5 w-1/3" />
          </div>
        </div>
      ))}
    </div>
  );
}
