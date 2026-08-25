import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { useCandidaturesViewModel } from "../../viewmodel/useCandidaturesViewModel";
import type { Candidature } from "../../services/candidature.service";
import { candidatureService } from "../../services/candidature.service";
import { contratLabel, statutMeta } from "../../model/statuts";
import { versDateAffichee } from "../../model/schemas/candidature-form.schema";
import { CandidatureFormModal } from "../components/CandidatureFormModal";
import { CandidatureFilters } from "../components/CandidatureFilters";
import { CandidatureDetail } from "../components/CandidatureDetail";
import { KanbanBoard } from "../components/KanbanBoard";
import {
  Button,
  ConfirmDialog,
  DataTable,
  EmptyState,
  ErrorBanner,
  Icon,
  PageHeader,
  Pager,
  SkeletonRows,
  StatusPill,
} from "@/shared/ui";
import type { Column } from "@/shared/ui";
import type { TriCandidature } from "@/shared/types/generated/candidatures";
import { controlClasses } from "@/shared/ui/FormField";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { cn } from "@/shared/lib/cn";

/** Écran Suivi → Candidatures : Kanban ou Liste, sur le même filtre. */
export function CandidaturesPage() {
  const vm = useCandidaturesViewModel();
  const notify = useUiStore((state) => state.notify);
  const [formulaire, setFormulaire] = useState<{ ouvert: boolean; cible: Candidature | null }>({
    ouvert: false,
    cible: null,
  });
  const [filtresOuverts, setFiltresOuverts] = useState(false);
  const [aSupprimer, setASupprimer] = useState<Candidature | null>(null);
  const [exportEnCours, setExportEnCours] = useState(false);

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
      render: (row) => <span className="font-medium text-ink">{row.poste}</span>,
    },
    {
      key: "entreprise",
      header: "Entreprise",
      sortKey: "entreprise",
      render: (row) => (
        <span className="text-ink-muted">
          {row.entrepriseNom ?? "—"}
          {row.entrepriseVille ? ` · ${row.entrepriseVille}` : ""}
        </span>
      ),
    },
    {
      key: "contrat",
      header: "Contrat",
      width: "110px",
      render: (row) => contratLabel(row.typeContrat),
    },
    {
      key: "statut",
      header: "Statut",
      sortKey: "statut",
      width: "150px",
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
      width: "120px",
      numeric: true,
      render: (row) => versDateAffichee(row.dateEnvoi),
    },
  ];

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="work"
        title="Candidatures"
        subtitle="Suivi de vos dossiers"
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

      <div className="flex flex-none items-center gap-2 border-b border-line bg-surface-alt px-6 py-2.5">
        <div className="relative w-[280px]">
          <Icon
            name="search"
            size={16}
            className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-faint"
          />
          <input
            type="search"
            value={vm.search}
            onChange={(event) => vm.rechercher(event.target.value)}
            placeholder="Rechercher un poste ou une entreprise…"
            aria-label="Rechercher une candidature"
            className={controlClasses(false, "pl-8")}
          />
        </div>

        <Button icon="filter_alt" onClick={() => setFiltresOuverts(true)}>
          Filtres
          {vm.filtresActifs > 0 ? (
            <span className="tabular ml-1 rounded-pill bg-accent px-1.5 text-meta text-white">
              {vm.filtresActifs}
            </span>
          ) : null}
        </Button>

        {vm.filtresActifs > 0 ? (
          <Button variant="ghost" icon="filter_alt_off" onClick={vm.reinitialiserFiltres}>
            Effacer
          </Button>
        ) : null}

        <div className="flex-1" />

        <div className="flex items-center gap-0.5 rounded-button border border-line bg-surface p-0.5">
          {(["kanban", "liste"] as const).map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => vm.setVue(mode)}
              aria-pressed={vm.vue === mode}
              className={cn(
                "flex items-center gap-1.5 rounded-[6px] px-2.5 py-1 text-meta transition-colors duration-150",
                vm.vue === mode
                  ? "bg-accent-tint text-accent"
                  : "text-ink-muted hover:text-ink",
              )}
            >
              <Icon name={mode === "kanban" ? "view_kanban" : "view_list"} size={15} />
              {mode === "kanban" ? "Kanban" : "Liste"}
            </button>
          ))}
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="flex min-w-0 flex-1 flex-col">
          {vm.error ? (
            <div className="p-6">
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
            <div className="p-6">
              <div className="overflow-hidden rounded-card border border-line bg-surface">
                <SkeletonRows rows={6} columns={5} />
              </div>
            </div>
          ) : vm.total === 0 ? (
            <div className="flex flex-1 items-center justify-center">
              <EmptyState
                icon="work"
                title={
                  vm.search || vm.filtresActifs > 0
                    ? "Aucun résultat"
                    : "Aucune candidature"
                }
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
            <div className="flex min-h-0 flex-1 flex-col gap-3 p-6">
              <DataTable
                columns={colonnes}
                rows={vm.items}
                rowKey={(row) => row.id}
                sort={{ key: vm.tri, direction: vm.descendant ? "desc" : "asc" }}
                onSortChange={vm.trierPar}
                onRowClick={(row) => vm.selectionner(row.id)}
              />
              <div className="overflow-hidden rounded-card border border-line">
                <Pager
                  page={vm.page}
                  pageSize={vm.pageSize}
                  total={vm.total}
                  label="candidatures"
                  onPageChange={vm.setPage}
                />
              </div>
            </div>
          )}
        </div>

        {vm.selection ? (
          <CandidatureDetail
            candidature={vm.selection}
            onClose={() => vm.selectionner(null)}
            onEdit={() => setFormulaire({ ouvert: true, cible: vm.selection })}
            onDelete={() => setASupprimer(vm.selection)}
            onStatutChange={(statut) =>
              void vm.changerStatut({ id: vm.selection!.id, statut })
            }
          />
        ) : null}
      </div>

      <CandidatureFormModal
        open={formulaire.ouvert}
        candidature={formulaire.cible}
        busy={vm.isSaving}
        onClose={() => setFormulaire({ ouvert: false, cible: null })}
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
