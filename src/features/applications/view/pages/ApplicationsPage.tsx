import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useApplicationsViewModel } from "../../viewmodel/useApplicationsViewModel";
import type { TrackingView } from "../../viewmodel/useApplicationsViewModel";
import { useScheduleFollowUp } from "../../viewmodel/useScheduleFollowUp";
import type { Application, ApplicationStatus } from "@/shared/types/generated/applications";
import { Statuses } from "../../model/statuses";
import { formatReference } from "../../model/presentation";
import { EMPTY_FILTER } from "../../model/schemas/application-filter.schema";
import { ApplicationFormModal } from "../components/ApplicationFormModal";
import { ApplicationFilters } from "../components/ApplicationFilters";
import { ApplicationGroupList } from "../components/ApplicationGroupList";
import type { Anchor } from "../components/ApplicationGroupList";
import { ApplicationInspector } from "../components/ApplicationInspector";
import { DeleteApplicationDialog } from "../components/DeleteApplicationDialog";
import { applicationMenu } from "../components/applicationActions";
import type { ApplicationHandlers } from "../components/applicationActions";
import { KanbanBoard } from "../components/KanbanBoard";
import { FollowUpFormModal } from "@/features/followups";
import { AppError } from "@/shared/types/app-error";
import { PATHS } from "@/shared/lib/paths";
import { useChrome } from "@/shared/lib/chrome";
import { useRegisterCommands } from "@/shared/lib/commands";
import type { Command } from "@/shared/lib/commands";
import { openExternal } from "@/shared/services/external-link";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { useMediaQuery, WIDE_QUERY } from "@/shared/hooks/useMediaQuery";
import {
  Button,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  LineIcon,
  Menu,
  StatusGlyph,
} from "@/shared/ui";
import type { MenuEntry } from "@/shared/ui";

interface OpenMenu {
  readonly label: string;
  readonly anchor: Anchor;
  readonly entries: readonly MenuEntry[];
}

/** Ancre par défaut d'un menu ouvert au clavier, sans ligne visible sous la main. */
function centre(): Anchor {
  return { x: window.innerWidth / 2 - 118, y: window.innerHeight / 3 };
}

/**
 * Écran Candidatures — vue Liste (`screens/02-applications-list.png`, écran de référence)
 * et vue Kanban, sur le même filtre et le même inspecteur.
 *
 * Contrat clavier (`INTERACTIONS.md` §6) : `N` nouvelle, `S` statut, `R` relance, `⏎`
 * ouvrir, `⌘⏎` modifier la fiche, `⌘D` dupliquer, `⌘⌫` supprimer — sur la candidature
 * sélectionnée ou focalisée. Sous 1060 px, l'inspecteur devient un panneau flottant.
 */
export function ApplicationsPage({ view }: { view?: TrackingView } = {}) {
  const vm = useApplicationsViewModel(view);
  const navigate = useNavigate();
  const wide = useMediaQuery(WIDE_QUERY);
  const scheduleFollowUp = useScheduleFollowUp();
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
  const [pendingDelete, setPendingDelete] = useState<Application | null>(null);
  const [pendingBulkDelete, setPendingBulkDelete] = useState<string[] | null>(null);
  const [checkedIds, setCheckedIds] = useState<Set<string>>(() => new Set());
  // « Refusée » replié par défaut, comme dans la maquette : ce qui est clos se consulte,
  // il ne s'impose pas en tête de liste.
  const [collapsed, setCollapsed] = useState<Set<ApplicationStatus>>(() => new Set(["REFUS"]));
  const [menu, setMenu] = useState<OpenMenu | null>(null);
  const [followUpFor, setFollowUpFor] = useState<Application | null>(null);
  const [floating, setFloating] = useState(false);

  const selection = vm.selection;
  const isFresh = !vm.isLoading && vm.total === 0 && !vm.search && vm.activeFilterCount === 0;
  const floatingOpen = !wide && floating && selection !== null;
  useDismissable({ open: floatingOpen, onDismiss: () => setFloating(false) });

  // Le bouton principal d'Aujourd'hui et la palette ouvrent la création par `?new=1`. Le
  // paramètre reste dans l'URL le temps de la modale, puis est consommé à sa fermeture.
  const closeForm = () => {
    setForm({ isOpen: false, editing: null, status: null });
    if (searchParams.get("new") === "1") {
      // Seul `new` est consommé : effacer toute la query effacerait aussi la fiche
      // ouverte dans l'inspecteur.
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

  const openCreate = (status: ApplicationStatus | null = null) =>
    setForm({ isOpen: true, editing: null, status });

  const statusMenu = (application: Application, anchor: Anchor) =>
    setMenu({
      label: `Statut de ${formatReference(application.reference_number)}`,
      anchor,
      entries: [
        { kind: "section", id: "titre", label: "Changer le statut" },
        ...Statuses.map(
          (status): MenuEntry => ({
            kind: "item",
            id: status.value,
            label: status.label,
            leading: <StatusGlyph tone={status.glyph} />,
            checked: status.value === application.status,
            onSelect: () => void vm.changeStatus({ id: application.id, status: status.value }),
          }),
        ),
      ],
    });

  const handlers: ApplicationHandlers = {
    open: (application) => {
      vm.select(application.id);
      setFloating(true);
    },
    edit: (application) => setForm({ isOpen: true, editing: application, status: null }),
    openOffer: (application) => {
      if (application.job_url) void openExternal(application.job_url);
    },
    changeStatus: (application) => statusMenu(application, centre()),
    scheduleFollowUp: (application) => setFollowUpFor(application),
    generateResume: () => void navigate(PATHS.generateResume),
    generateLetter: () => void navigate(PATHS.writeLetter),
    analyzeResume: () => void navigate(PATHS.analyzeResume),
    duplicate: (application) => void vm.duplicate(application.id),
    remove: (application) => setPendingDelete(application),
  };

  const openActions = (application: Application, anchor: Anchor) =>
    setMenu({
      label: `Actions sur ${formatReference(application.reference_number)}`,
      anchor,
      entries: applicationMenu(application, handlers),
    });

  // Raccourcis d'écran sur la candidature sélectionnée. Ils se taisent pendant une saisie
  // ou quand une surface est ouverte (`useShortcut`).
  const onSelection = (run: (application: Application) => void) => () => {
    if (selection) run(selection);
  };
  useShortcut("n", () => openCreate());
  useShortcut("s", onSelection((application) => statusMenu(application, centre())));
  useShortcut("r", onSelection(handlers.scheduleFollowUp));
  useShortcut("mod+enter", onSelection(handlers.edit));
  useShortcut("mod+d", onSelection(handlers.duplicate));
  useShortcut("mod+backspace", onSelection(handlers.remove));

  const selectionLabel = selection ? formatReference(selection.reference_number) : "";
  const on = (run: (application: Application) => void) => () => {
    if (selection) run(selection);
  };
  const commands: readonly Command[] = selection
    ? [
        { id: "app-follow-up", group: "selection", glyph: "↻", label: "Programmer une relance", detail: selection.job_title, shortcut: "r", run: on(handlers.scheduleFollowUp) },
        { id: "app-status", group: "selection", glyph: "▲", label: "Changer le statut", detail: selectionLabel, shortcut: "s", run: on(handlers.changeStatus) },
        { id: "app-edit", group: "selection", glyph: "✎", label: "Modifier la fiche", detail: selectionLabel, shortcut: "mod+enter", run: on(handlers.edit) },
        { id: "app-duplicate", group: "selection", glyph: "◫", label: "Dupliquer", detail: selectionLabel, shortcut: "mod+d", run: on(handlers.duplicate) },
        { id: "app-delete", group: "selection", glyph: "▤", label: "Supprimer…", detail: selectionLabel, shortcut: "mod+backspace", run: on(handlers.remove) },
      ]
    : [];
  useRegisterCommands(commands);

  useChrome({
    crumb: view === "kanban" ? "Kanban" : "Toutes",
    status: isFresh
      ? view === "kanban"
        ? "0 carte · 4 colonnes · le tableau se remplira tout seul"
        : "0 candidature · base neuve"
      : view === "kanban"
        ? `${vm.total} cartes · 4 colonnes · glissez pour changer de statut`
        : `${selection ? "1 sélectionnée · " : ""}${vm.total} candidature${vm.total > 1 ? "s" : ""}`,
    keys: [
      { label: "Statut", shortcut: "s" },
      { label: "Relance", shortcut: "r" },
      { label: "Ouvrir", shortcut: "enter" },
    ],
  });

  const toggleChecked = (id: string) => {
    setCheckedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  /** Exporte le filtre courant, ou uniquement les lignes cochées. */
  const exportRows = async () => {
    const ids = [...checkedIds];
    // Les identifiants cochés suffisent : les combiner au filtre courant exclurait une
    // ligne sélectionnée puis masquée par une recherche ou un statut.
    const filter =
      ids.length > 0
        ? { ...EMPTY_FILTER, sort: vm.sort, descending: vm.descending, search: "", ids }
        : vm.filter;
    await vm.exportCsv(filter);
  };

  const filtersActive = vm.search !== "" || vm.activeFilterCount > 0;
  const inspector = selection ? (
    <ApplicationInspector
      application={selection}
      floating={!wide}
      onClose={() => (wide ? vm.select(null) : setFloating(false))}
      onEdit={() => handlers.edit(selection)}
      onMenu={(anchor) => openActions(selection, anchor)}
      onStatusMenu={(anchor) => statusMenu(selection, anchor)}
      onScheduleFollowUp={() => handlers.scheduleFollowUp(selection)}
    />
  ) : null;

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
            {checkedIds.size > 0 ? (
              <>
                <span className="text-small font-medium text-tx">
                  {checkedIds.size} cochée{checkedIds.size > 1 ? "s" : ""}
                </span>
                <Button variant="ghost" size="compact" onClick={() => setCheckedIds(new Set())}>
                  Tout décocher
                </Button>
                <Button variant="danger" size="compact" onClick={() => setPendingBulkDelete([...checkedIds])}>
                  Supprimer
                </Button>
              </>
            ) : null}
            <Button size="compact" disabled={vm.isExporting} onClick={() => void exportRows()}>
              <LineIcon name="export-csv" size={13} />
              CSV
            </Button>
            <Button variant="primary" size="compact" shortcut="n" onClick={() => openCreate()}>
              Ajouter une candidature
            </Button>
          </>
        }
      />

      <div className="relative flex min-h-0 flex-1">
        <div className="flex min-w-0 flex-1 flex-col">
          {vm.error ? (
            <div className="px-3.5 pt-3.5">
              <ErrorBanner
                message={
                  vm.error instanceof AppError
                    ? vm.error.message
                    : "Les candidatures n'ont pas pu être chargées."
                }
                onRetry={vm.reload}
              />
            </div>
          ) : !vm.isLoading && vm.total === 0 ? (
            <div className="flex min-h-0 flex-1 items-start justify-center pt-[min(12vh,90px)]">
              {isFresh ? (
                <EmptyState
                  icon="work"
                  title={view === "kanban" ? "Le tableau est vide" : "Votre suivi commence ici"}
                  description={
                    view === "kanban"
                      ? "Les quatre colonnes existent déjà : En attente, Relancée, Entretien, Refusée. Elles se rempliront à mesure que vous ajouterez des candidatures — faites-les glisser pour changer un statut."
                      : "Ajoutez votre première candidature : intitulé et entreprise suffisent. Candilog s'occupe du reste — relances, documents, statuts."
                  }
                  action={
                    <>
                      <Button variant="primary" size="empty" shortcut="n" onClick={() => openCreate()}>
                        Nouvelle candidature
                      </Button>
                      <Button variant="ghost" size="empty" onClick={() => void navigate(PATHS.profile)}>
                        Importer un CV pour commencer
                      </Button>
                    </>
                  }
                />
              ) : (
                <EmptyState
                  icon="filter_alt_off"
                  title="Aucune candidature ne correspond"
                  description={
                    vm.search
                      ? `Aucune candidature ne contient « ${vm.search} »${vm.activeFilterCount > 0 ? ` avec ${vm.activeFilterCount === 1 ? "le filtre actif" : `les ${vm.activeFilterCount} filtres actifs`}` : ""}. Modifiez la recherche pour voir le reste du suivi.`
                      : `${vm.activeFilterCount === 1 ? "Un filtre est actif" : `${vm.activeFilterCount} filtres sont actifs`}. Retirez-en un pour voir le reste du suivi.`
                  }
                  action={
                    filtersActive ? (
                      <Button
                        size="empty"
                        onClick={() => {
                          vm.resetFilters();
                          vm.setSearch("");
                        }}
                      >
                        Effacer les filtres
                      </Button>
                    ) : null
                  }
                />
              )}
            </div>
          ) : vm.view === "kanban" ? (
            <KanbanBoard
              columns={vm.kanbanColumns}
              selected_id={vm.selected_id}
              checkedIds={checkedIds}
              filtered={filtersActive}
              onMenu={openActions}
              onSelect={(id) => {
                vm.select(id);
                setFloating(true);
              }}
              onToggleSelect={toggleChecked}
              onStatusChange={(id, status) => void vm.changeStatus({ id, status })}
              onCreate={(status) => openCreate(status)}
              onPageChange={vm.setKanbanPage}
            />
          ) : (
            <ApplicationGroupList
              groups={vm.kanbanColumns}
              collapsed={collapsed}
              onToggleGroup={(status) =>
                setCollapsed((current) => {
                  const next = new Set(current);
                  if (next.has(status)) next.delete(status);
                  else next.add(status);
                  return next;
                })
              }
              selectedId={vm.selected_id}
              checkedIds={checkedIds}
              loading={vm.isLoading}
              handlers={{
                onSelect: handlers.open,
                onToggleCheck: toggleChecked,
                onMenu: openActions,
                onStatusMenu: statusMenu,
                onCreate: (status) => openCreate(status),
                onShowMore: vm.showMore,
                onKey: (application, event) => {
                  const mod = event.metaKey || event.ctrlKey;
                  const key = event.key.toLowerCase();
                  if (key === "enter" && mod) handlers.edit(application);
                  else if (key === "enter") handlers.open(application);
                  else if (key === "s" && !mod) statusMenu(application, centre());
                  else if (key === "r" && !mod) handlers.scheduleFollowUp(application);
                  else if (key === "d" && mod) handlers.duplicate(application);
                  else if ((key === "backspace" || key === "delete") && mod) handlers.remove(application);
                  else return false;
                  // Le geste est traité ici : il ne doit pas remonter aux raccourcis d'écran,
                  // qui agiraient une seconde fois sur la sélection.
                  event.stopPropagation();
                  return true;
                },
              }}
            />
          )}
        </div>

        {wide ? inspector : null}
        {floatingOpen ? (
          <>
            <button
              type="button"
              aria-label="Fermer la fiche"
              tabIndex={-1}
              onClick={() => setFloating(false)}
              className="absolute inset-0 z-30 bg-scrim opacity-50"
            />
            {inspector}
          </>
        ) : null}
      </div>

      <ApplicationFormModal
        open={form.isOpen}
        application={form.editing}
        defaultStatus={form.status}
        busy={vm.isSaving}
        onClose={closeForm}
        onSubmit={(values) =>
          form.editing ? vm.update({ id: form.editing.id, input: values }) : vm.create(values)
        }
      />

      <FollowUpFormModal
        open={followUpFor !== null}
        follow_up={null}
        application_id={followUpFor?.id ?? null}
        busy={scheduleFollowUp.isPending}
        onClose={() => setFollowUpFor(null)}
        onSubmit={(values) => scheduleFollowUp.mutateAsync(values)}
      />

      <DeleteApplicationDialog
        application={pendingDelete}
        busy={vm.isDeleting}
        onCancel={() => setPendingDelete(null)}
        onConfirm={() => {
          const target = pendingDelete;
          setPendingDelete(null);
          if (!target) return;
          void vm.delete(target.id).then(() =>
            setCheckedIds((current) => {
              const next = new Set(current);
              next.delete(target.id);
              return next;
            }),
          );
        }}
      />

      <ConfirmDialog
        open={pendingBulkDelete !== null}
        title={`Supprimer ${pendingBulkDelete?.length ?? 0} candidatures ?`}
        description="Les candidatures cochées disparaissent de votre suivi, avec leurs relances, entretiens et historique."
        note="Les entreprises et les contacts associés sont conservés."
        footnote="action définitive"
        busy={vm.isDeleting}
        onCancel={() => setPendingBulkDelete(null)}
        onConfirm={() => {
          const ids = pendingBulkDelete;
          setPendingBulkDelete(null);
          if (!ids || ids.length === 0) return;
          void vm.deleteMany(ids).then(() => setCheckedIds(new Set()));
        }}
      />

      <Menu
        open={menu !== null}
        anchor={menu?.anchor ?? null}
        entries={menu?.entries ?? []}
        label={menu?.label ?? ""}
        onClose={() => setMenu(null)}
      />
    </div>
  );
}
