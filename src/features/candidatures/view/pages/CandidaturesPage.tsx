import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { save } from "@tauri-apps/plugin-dialog";
import { useCandidaturesViewModel } from "../../viewmodel/useCandidaturesViewModel";
import type { Candidature } from "../../services/candidature.service";
import { candidatureService } from "../../services/candidature.service";
import { contratLabel, statutMeta } from "../../model/statuts";
import { versDateAffichee } from "@/shared/lib/dates";
import { CandidatureFormModal } from "../components/CandidatureFormModal";
import { CandidatureFilters } from "../components/CandidatureFilters";
import { CandidatureDetail } from "../components/CandidatureDetail";
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
import type { TriCandidature } from "@/shared/types/generated/candidatures";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { PAGE_SIZE } from "@/shared/types/page";

/** Densités proposées par le pied de la vue Liste. */
const DENSITES = [PAGE_SIZE, 25, 50] as const;

/** Écran Suivi → Candidatures : Kanban ou Liste, sur le même filtre. */
export function CandidaturesPage() {
  const vm = useCandidaturesViewModel();
  const [searchParams, setSearchParams] = useSearchParams();
  const notify = useUiStore((state) => state.notify);
  const [formulaire, setFormulaire] = useState<{ ouvert: boolean; cible: Candidature | null }>({
    ouvert: searchParams.get("nouvelle") === "1",
    cible: null,
  });
  const [filtresOuverts, setFiltresOuverts] = useState(false);
  const [aSupprimer, setASupprimer] = useState<Candidature | null>(null);
  const [exportEnCours, setExportEnCours] = useState(false);

  // Le bouton principal du Dashboard ouvre réellement la création, sans dupliquer le
  // formulaire ni son ViewModel dans une autre feature. Le paramètre reste dans l'URL le
  // temps de la modale, puis est consommé à sa fermeture.
  const fermerFormulaire = () => {
    setFormulaire({ ouvert: false, cible: null });
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
    const chemin = await save({
      title: "Exporter les candidatures",
      defaultPath: "candidatures.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (chemin === null) return;

    setExportEnCours(true);
    try {
      const lignes = await candidatureService.exporterCsv(vm.filtre, chemin);
      notify({
        tone: "success",
        title: "Export terminé",
        detail: `${lignes} candidature${lignes > 1 ? "s" : ""} exportée${lignes > 1 ? "s" : ""}.`,
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

  const colonnes: Column<Candidature, TriCandidature>[] = [
    {
      key: "poste",
      header: "Poste",
      sortKey: "poste",
      grow: 2.2,
      render: (row) => (
        <CellIdentity
          initials={initiales(row.entrepriseNom ?? row.poste)}
          title={row.poste}
          subtitle={row.entrepriseVille ?? undefined}
        />
      ),
    },
    {
      key: "entreprise",
      header: "Entreprise",
      sortKey: "entreprise",
      grow: 1.3,
      render: (row) => (
        <span className="truncate text-body text-ink-muted">{row.entrepriseNom ?? "—"}</span>
      ),
    },
    {
      key: "contrat",
      header: "Contrat",
      grow: 0.9,
      render: (row) => <span className="text-note text-ink-faint">{contratLabel(row.typeContrat)}</span>,
    },
    {
      key: "statut",
      header: "Statut",
      sortKey: "statut",
      grow: 1.1,
      render: (row) => {
        const statut = statutMeta(row.statut);
        return (
          <StatusPill tone={statut.tone} icon={statut.icon}>
            {statut.label}
          </StatusPill>
        );
      },
    },
    {
      key: "date",
      header: "Envoyée",
      sortKey: "date",
      grow: 0.7,
      numeric: true,
      render: (row) => (
        <span className="text-note text-ink-faint">{versDateAffichee(row.dateEnvoi)}</span>
      ),
    },
  ];

  const filtres = vm.filtres;

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
            value={vm.vue}
            onChange={vm.setVue}
            options={[
              { value: "kanban", label: "Kanban", icon: "view_kanban" },
              { value: "liste", label: "Liste", icon: "view_list" },
            ]}
          />
        }
        secondary={
          <Button icon="download" disabled={exportEnCours} onClick={() => void exporter()}>
            Exporter
          </Button>
        }
        primary={
          <Button
            variant="primary"
            icon="add"
            onClick={() => setFormulaire({ ouvert: true, cible: null })}
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
                vm.filtresActifs > 0 ? " · filtrées" : ""
              }`
        }
      >
        <FilterChip
          icon="filter_alt"
          label={filtres.statut ? statutMeta(filtres.statut).label : "Statut"}
          active={filtres.statut !== null}
          onClick={() => setFiltresOuverts(true)}
        />
        <FilterChip
          icon="badge"
          label={filtres.contrat ? contratLabel(filtres.contrat) : "Contrat"}
          active={filtres.contrat !== null}
          onClick={() => setFiltresOuverts(true)}
        />
        <FilterChip
          icon="apartment"
          label={filtres.ville || "Ville"}
          active={filtres.ville !== ""}
          onClick={() => setFiltresOuverts(true)}
        />
        <FilterChip
          icon="date_range"
          label={periodeLabel(filtres.dateDebut, filtres.dateFin)}
          active={filtres.dateDebut !== null || filtres.dateFin !== null}
          onClick={() => setFiltresOuverts(true)}
        />
        {vm.filtresActifs > 0 ? (
          <Button variant="ghost" icon="filter_alt_off" onClick={vm.reinitialiserFiltres}>
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
                title={vm.search || vm.filtresActifs > 0 ? "Aucun résultat" : "Aucune candidature"}
                description={
                  vm.search || vm.filtresActifs > 0
                    ? "Aucune candidature ne correspond à ces critères."
                    : "Créez votre première candidature pour lancer le suivi."
                }
                action={
                  vm.search || vm.filtresActifs > 0 ? (
                    <Button icon="filter_alt_off" onClick={vm.reinitialiserFiltres}>
                      Effacer les filtres
                    </Button>
                  ) : (
                    <Button
                      variant="primary"
                      icon="add"
                      onClick={() => setFormulaire({ ouvert: true, cible: null })}
                    >
                      Nouvelle candidature
                    </Button>
                  )
                }
              />
            </div>
          ) : vm.vue === "kanban" ? (
            <KanbanBoard
              candidatures={vm.items}
              repartition={vm.repartition}
              selectedId={vm.selectedId}
              onSelect={vm.selectionner}
              onStatutChange={(id, statut) => void vm.changerStatut({ id, statut })}
              onCreate={() => setFormulaire({ ouvert: true, cible: null })}
            />
          ) : (
            <div className="min-h-0 flex-1 overflow-auto px-7 pt-[18px] pb-[26px]">
              <DataTable
                columns={colonnes}
                rows={vm.items}
                rowKey={(row) => row.id}
                sort={{ key: vm.tri, direction: vm.descendant ? "desc" : "asc" }}
                onSortChange={vm.trierPar}
                onRowClick={(row) => vm.selectionner(row.id)}
                isSelected={(row) => row.id === vm.selectedId}
                footer={
                  <Pager
                    page={vm.page}
                    pageSize={vm.pageSize}
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
          <CandidatureDetail
            candidature={vm.selection}
            onClose={() => vm.selectionner(null)}
            onEdit={() => setFormulaire({ ouvert: true, cible: vm.selection })}
            onDelete={() => setASupprimer(vm.selection)}
            onStatutChange={(statut) => void vm.changerStatut({ id: vm.selection!.id, statut })}
          />
        ) : null}
      </div>

      <CandidatureFormModal
        open={formulaire.ouvert}
        candidature={formulaire.cible}
        busy={vm.isSaving}
        onClose={fermerFormulaire}
        onSubmit={(valeurs) =>
          formulaire.cible
            ? vm.modifier({ id: formulaire.cible.id, input: valeurs })
            : vm.creer(valeurs)
        }
      />

      <CandidatureFilters
        open={filtresOuverts}
        filtres={vm.filtres}
        onClose={() => setFiltresOuverts(false)}
        onApply={vm.appliquerFiltres}
        onReset={vm.reinitialiserFiltres}
      />

      <ConfirmDialog
        open={aSupprimer !== null}
        title="Supprimer cette candidature ?"
        description={`« ${aSupprimer?.poste ?? ""} » chez ${aSupprimer?.entrepriseNom ?? "cette entreprise"} sera définitivement supprimée, ainsi que les entretiens et relances rattachés.`}
        note="L'entreprise et le contact associés sont conservés."
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

/** Libellé de la puce de période : la borne renseignée, ou les deux. */
function periodeLabel(debut: string | null, fin: string | null): string {
  if (debut && fin) return `${versDateAffichee(debut)} → ${versDateAffichee(fin)}`;
  if (debut) return `Depuis le ${versDateAffichee(debut)}`;
  if (fin) return `Jusqu’au ${versDateAffichee(fin)}`;
  return "Période";
}

/** Initiales de l'entreprise, pour la pastille de la colonne « Poste ». */
function initiales(valeur: string): string {
  return valeur
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((mot) => mot[0])
    .join("")
    .toUpperCase();
}
