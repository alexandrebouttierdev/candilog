import { useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { CompanyFormModal } from "@/features/companies";
import type { Company } from "@/features/companies";
import { ContactFormModal } from "@/features/contacts";
import type { Contact } from "@/features/contacts";
import { FollowUpFormModal } from "@/features/followups";
import { useScheduleFollowUp } from "@/features/applications";
import type { Application } from "@/features/applications";
import { PATHS, applicationsPath } from "@/shared/lib/paths";
import { useChrome } from "@/shared/lib/chrome";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { useMediaQuery, WIDE_QUERY } from "@/shared/hooks/useMediaQuery";
import { AppError } from "@/shared/types/app-error";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, Kbd, Menu } from "@/shared/ui";
import type { MenuAnchor } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { useRelationsViewModel } from "../../viewmodel/useRelationsViewModel";
import type { RelationKind } from "../../viewmodel/useRelationsViewModel";
import { RelationList, companyRow, contactRow } from "../components/RelationList";
import { RelationInspector } from "../components/RelationInspector";
import { RelationFilters } from "../components/RelationFilters";

type Form =
  | { kind: "company"; editing: Company | null }
  | { kind: "contact"; editing: Contact | null }
  | null;

/**
 * Relations (`screens/05-companies.png`, `06-network.png`) : entreprises et contacts sur un
 * même écran, bascule dans la barre d'outils, liste groupée et inspecteur de 300 px.
 *
 * Contrat clavier : `N` nouvelle fiche, `F` filtre, `/` recherche, `↑ ↓` parcourent la
 * liste, `⏎` ouvre la fiche (panneau flottant sous 1060 px).
 */
export function RelationsPage({ kind }: { kind: RelationKind }) {
  const vm = useRelationsViewModel(kind);
  const navigate = useNavigate();
  const wide = useMediaQuery(WIDE_QUERY);
  const scheduleFollowUp = useScheduleFollowUp();
  const searchInput = useRef<HTMLInputElement>(null);
  const [form, setForm] = useState<Form>(null);
  const [pendingDelete, setPendingDelete] = useState<Company | Contact | null>(null);
  const [menu, setMenu] = useState<MenuAnchor | null>(null);
  const [followUpFor, setFollowUpFor] = useState<Application | null>(null);
  const [floating, setFloating] = useState(false);

  const companies = kind === "companies";
  const selection = vm.company ?? vm.contact;
  const floatingOpen = !wide && floating && selection !== null;
  useDismissable({ open: floatingOpen, onDismiss: () => setFloating(false) });

  const openCreate = () => setForm(companies ? { kind: "company", editing: null } : { kind: "contact", editing: null });
  const openEdit = () => {
    if (vm.company) setForm({ kind: "company", editing: vm.company });
    else if (vm.contact) setForm({ kind: "contact", editing: vm.contact });
  };
  useShortcut("n", openCreate);
  useShortcut("/", () => searchInput.current?.focus());

  const filtered = vm.search !== "" || vm.activeCriteria > 0;
  const fresh = !vm.isLoading && !vm.error && vm.total === 0 && !filtered;
  const { companies: companyCount, contacts: contactCount } = vm.counts;
  const activeCompanies = vm.companyGroups.find((group) => group.key === "active")?.total;

  useChrome({
    crumb: companies ? "Entreprises" : "Contacts",
    status:
      companyCount === null || contactCount === null
        ? ""
        : `${companyCount} entreprise${companyCount > 1 ? "s" : ""} · ${contactCount} contact${contactCount > 1 ? "s" : ""}${
            companies && activeCompanies !== undefined ? ` · ${activeCompanies} relation${activeCompanies > 1 ? "s" : ""} active${activeCompanies > 1 ? "s" : ""}` : ""
          }`,
    keys: [
      { label: "Ouvrir", shortcut: "enter" },
      { label: companies ? "Nouvelle" : "Nouveau", shortcut: "n" },
    ],
  });

  const groups = companies
    ? vm.companyGroups.map((group) => ({ ...group, rows: group.items.map(companyRow) }))
    : vm.contactGroups.map((group) => ({ ...group, rows: group.items.map(contactRow) }));

  const inspector = selection ? (
    <RelationInspector
      company={vm.company}
      contact={vm.contact}
      applications={vm.linkedApplications}
      applicationsTotal={vm.linkedTotal}
      floating={!wide}
      onClose={() => setFloating(false)}
      onMenu={(event) => setMenu(event.currentTarget.getBoundingClientRect())}
      onEdit={openEdit}
      onNewApplication={() =>
        void navigate(applicationsPath({ create: true, ...(vm.company ? { companyId: vm.company.id } : {}) }))
      }
      onFollowUp={setFollowUpFor}
      onOpenApplication={(id) => void navigate(applicationsPath({ id }))}
      onOpenCompany={(id) => void navigate(`${PATHS.companies}?id=${encodeURIComponent(id)}`)}
    />
  ) : null;

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-toolbar flex-none items-center gap-[9px] border-b border-bd-soft px-3.5">
        <div role="tablist" aria-label="Population" className="flex flex-none gap-px rounded-r7 bg-chip p-0.5">
          {(
            [
              { value: "companies", label: "Entreprises", count: companyCount, path: PATHS.companies },
              { value: "contacts", label: "Contacts", count: contactCount, path: PATHS.contacts },
            ] as const
          ).map((tab) => (
            <button
              key={tab.value}
              type="button"
              role="tab"
              aria-selected={kind === tab.value}
              onClick={() => void navigate(tab.path)}
              className={cn(
                "inline-flex items-center gap-[7px] rounded-r5 px-[11px] py-1 text-small whitespace-nowrap",
                kind === tab.value ? "bg-panel font-medium text-tx" : "text-tx-4 hover:text-tx-2",
              )}
            >
              {tab.label}
              {tab.count !== null ? <span className="font-mono text-[10px] text-tx-5">{tab.count}</span> : null}
            </button>
          ))}
        </div>
        <label className="flex h-[23px] w-[150px] flex-none items-center gap-[7px] rounded-r6 border border-transparent bg-elev px-[9px] text-small transition-[width] duration-100 focus-within:w-[200px] focus-within:border-ac focus-within:bg-panel">
          <span aria-hidden className="flex-none text-tiny text-tx-5">
            ⌕
          </span>
          <input
            ref={searchInput}
            type="search"
            aria-label={companies ? "Rechercher une entreprise" : "Rechercher un contact"}
            aria-keyshortcuts="/"
            placeholder="Rechercher"
            value={vm.search}
            onChange={(event) => vm.setSearch(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.stopPropagation();
                if (vm.search) vm.setSearch("");
                else event.currentTarget.blur();
              }
            }}
            className="peer min-w-0 flex-1 bg-transparent text-small text-tx outline-none placeholder:text-tx-5 [&::-webkit-search-cancel-button]:hidden"
          />
          {vm.search ? null : <Kbd shortcut="/" tone="ghost" decorative className="peer-focus:hidden" />}
        </label>
        <div className="flex min-w-0 flex-1 items-center gap-[7px] overflow-x-auto">
          <RelationFilters kind={kind} criteria={vm.criteria} onChange={vm.setCriteria} />
        </div>
        <Button variant="primary" size="compact" shortcut="n" onClick={openCreate}>
          {companies ? "Nouvelle entreprise" : "Nouveau contact"}
        </Button>
      </div>

      <div className="relative flex min-h-0 flex-1">
        <div className="flex min-w-0 flex-1 flex-col">
          {vm.error ? (
            <div className="p-3.5">
              <ErrorBanner
                message={vm.error instanceof AppError ? vm.error.message : "Les relations n'ont pas pu être chargées."}
                onRetry={vm.reload}
              />
            </div>
          ) : !vm.isLoading && vm.total === 0 ? (
            <div className="flex min-h-0 flex-1 items-start justify-center pt-[min(12vh,90px)]">
              <EmptyState
                icon={companies ? "apartment" : "person"}
                title={
                  fresh
                    ? companies
                      ? "Aucune entreprise pour l'instant"
                      : "Aucun contact pour l'instant"
                    : "Aucune relation ne correspond"
                }
                description={
                  fresh
                    ? companies
                      ? "Les entreprises apparaissent d'elles-mêmes : chaque candidature crée la sienne. Vous pouvez aussi en ajouter une pour suivre une société avant même qu'elle recrute."
                      : "Ajoutez un contact quand vous aurez parlé à quelqu'un — un recruteur, un ancien collègue. Candilog le rattachera à la bonne candidature."
                    : "Modifiez la recherche ou retirez un filtre pour voir le reste."
                }
                action={
                  fresh ? (
                    <>
                      <Button variant="primary" size="empty" onClick={openCreate}>
                        {companies ? "Nouvelle entreprise" : "Nouveau contact"}
                      </Button>
                      <Button variant="secondary" size="empty" onClick={() => void navigate(PATHS.applications)}>
                        Depuis mes candidatures
                      </Button>
                    </>
                  ) : (
                    <Button
                      size="empty"
                      onClick={() => {
                        vm.setSearch("");
                        vm.setCriteria({ sector_id: null, company_type_id: null, company_size: null, tracking_role: null });
                      }}
                    >
                      Effacer les filtres
                    </Button>
                  )
                }
              />
            </div>
          ) : (
            <RelationList
              groups={groups}
              selectedId={selection?.id ?? null}
              loading={vm.isLoading}
              onSelect={(id) => vm.select(id)}
              onOpen={(id) => {
                vm.select(id);
                setFloating(true);
              }}
              onShowMore={vm.showMore}
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

      <CompanyFormModal
        open={form?.kind === "company"}
        company={form?.kind === "company" ? form.editing : null}
        busy={vm.isSaving}
        onClose={() => setForm(null)}
        onSubmit={(input) => vm.saveCompany({ id: form?.kind === "company" ? (form.editing?.id ?? null) : null, input })}
      />
      <ContactFormModal
        open={form?.kind === "contact"}
        contact={form?.kind === "contact" ? form.editing : null}
        busy={vm.isSaving}
        onClose={() => setForm(null)}
        onSubmit={(input) => vm.saveContact({ id: form?.kind === "contact" ? (form.editing?.id ?? null) : null, input })}
      />
      <FollowUpFormModal
        open={followUpFor !== null}
        follow_up={null}
        application_id={followUpFor?.id ?? null}
        busy={scheduleFollowUp.isPending}
        onClose={() => setFollowUpFor(null)}
        onSubmit={(values) => scheduleFollowUp.mutateAsync(values)}
      />

      <Menu
        open={menu !== null}
        anchor={menu}
        label="Actions sur la fiche"
        onClose={() => setMenu(null)}
        entries={[
          { kind: "item", id: "modifier", label: "Modifier la fiche", onSelect: openEdit },
          { kind: "separator", id: "sep" },
          {
            kind: "item",
            id: "supprimer",
            label: "Supprimer…",
            tone: "danger",
            onSelect: () => setPendingDelete(selection),
          },
        ]}
      />

      <ConfirmDialog
        open={pendingDelete !== null}
        register="destruction"
        title={companies ? "Supprimer cette entreprise ?" : "Supprimer ce contact ?"}
        description={
          pendingDelete
            ? `« ${"first_name" in pendingDelete ? `${pendingDelete.first_name} ${pendingDelete.name}` : pendingDelete.name} » disparaît de vos relations.`
            : ""
        }
        note={
          companies
            ? "La suppression est refusée si des candidatures y sont rattachées ; ses contacts restent, détachés."
            : "La suppression est refusée si le contact est l'interlocuteur d'une candidature ou d'un entretien."
        }
        footnote="action définitive"
        busy={vm.isDeleting}
        onCancel={() => setPendingDelete(null)}
        onConfirm={() => {
          const target = pendingDelete;
          setPendingDelete(null);
          // L'échec est déjà annoncé par un toast : la promesse rejetée n'a rien de plus à dire.
          if (target) vm.remove({ kind, id: target.id }).catch(() => undefined);
        }}
      />
    </div>
  );
}
