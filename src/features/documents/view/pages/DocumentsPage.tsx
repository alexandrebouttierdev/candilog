import { useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import { useNavigate } from "react-router-dom";
import { useDocumentCounts } from "../../viewmodel/useDocumentCounts";
import { useDocumentsViewModel } from "../../viewmodel/useDocumentsViewModel";
import type { DocumentFilter, DocumentSelection } from "../../viewmodel/useDocumentsViewModel";
import { baseResumeContent, routeForContent } from "../../model/resumeWorkspace";
import { labelTone } from "./documentPageSupport";
import {
  AtsSection,
  Detail,
  Group,
  LetterRow,
  ResumeRow,
  Section,
  Sheet,
  shortDate,
  shortYearDate,
} from "../components/DocumentLibrary";
import { PATHS } from "@/shared/lib/paths";
import { useChrome } from "@/shared/lib/chrome";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { AppError } from "@/shared/types/app-error";
import { cn } from "@/shared/lib/cn";
import { Button, ConfirmDialog, EmptyState, ErrorBanner, GlyphButton, Kbd, Menu, Skeleton } from "@/shared/ui";
import type { MenuAnchor, MenuEntry } from "@/shared/ui";

const TABS: ReadonlyArray<{ value: DocumentFilter; label: string; path: string }> = [
  { value: "all", label: "Tous", path: PATHS.documents },
  { value: "resumes", label: "CV", path: PATHS.resumes },
  { value: "letters", label: "Lettres", path: PATHS.letters },
  { value: "analyses", label: "Analyses", path: PATHS.analyses },
];

/**
 * Documents (`screens/07-resumes.png`, `08-cover-letters.png`) : CV et lettres dans une même
 * bibliothèque, filtres en onglets, inspecteur de 300 px — aperçu, score ATS, détails.
 *
 * « Ouvrir » rouvre le document dans son éditeur A4 (générateur ou rédacteur), qui porte
 * l'aperçu pleine page et la retouche directe (`DECISIONS` D2).
 */
export function DocumentsPage({ filter }: { filter: DocumentFilter }) {
  const vm = useDocumentsViewModel(filter);
  const navigate = useNavigate();
  const searchInput = useRef<HTMLInputElement>(null);
  const [menu, setMenu] = useState<MenuAnchor | null>(null);

  const counts = useDocumentCounts();

  const open = () => {
    // `documentId` : enregistrer depuis le générateur (ou le CV de base) ajoute une version
    // à ce document plutôt que d'en créer un second.
    const documentId = vm.version?.id;
    const name = vm.version?.name;
    if (vm.version && routeForContent(vm.version.content) === PATHS.baseResume) {
      const base = baseResumeContent(vm.version.content);
      if (base) {
        void navigate(PATHS.baseResume, { state: { ...base, name, documentId } });
        return;
      }
    }
    if (vm.workspace) void navigate(PATHS.generateResume, { state: { workspace: vm.workspace, name, documentId } });
    else if (vm.generation) void navigate(PATHS.generateResume, { state: { generation: vm.generation, name, documentId } });
    else if (vm.letter) void navigate(PATHS.writeLetter, { state: { cover_letter: vm.letter } });
  };
  const canOpen = Boolean(vm.workspace ?? vm.generation ?? vm.letter);
  const currentVersion = vm.versions.find((version) => version.is_current) ?? null;

  useShortcut("n", () => void navigate(PATHS.generateResume));
  useShortcut("/", () => searchInput.current?.focus());
  useShortcut("mod+e", () => void vm.exportPdf());

  const total = counts.all;
  useChrome({
    crumb: TABS.find((tab) => tab.value === filter)?.label ?? "Tous",
    ...(total === undefined ? {} : { aside: `${total} document${total > 1 ? "s" : ""}` }),
    status:
      counts.resumes === undefined || counts.letters === undefined
        ? ""
        : `${total ?? 0} documents · ${counts.resumes} CV · ${counts.letters} lettre${counts.letters > 1 ? "s" : ""}`,
    keys: [
      { label: "Ouvrir", shortcut: "enter" },
      { label: "Exporter", shortcut: "mod+e" },
    ],
  });

  const rows: Array<{ selection: DocumentSelection }> = [
    ...vm.resumes.map((item) => ({ selection: { kind: "resume" as const, id: item.id } })),
    ...vm.letters.map((item) => ({ selection: { kind: "letter" as const, id: item.id } })),
  ];
  const isCurrent = (selection: DocumentSelection) =>
    vm.current?.kind === selection.kind && vm.current.id === selection.id;

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const index = rows.findIndex((row) => isCurrent(row.selection));
      const next = rows[Math.min(Math.max(index + (event.key === "ArrowDown" ? 1 : -1), 0), rows.length - 1)];
      if (next) vm.select(next.selection);
    } else if (event.key === "Enter" && canOpen) {
      event.preventDefault();
      open();
    }
  };

  const menuEntries: MenuEntry[] = [
    ...(vm.current?.kind === "resume"
      ? [{ kind: "item" as const, id: "dupliquer", label: "Dupliquer", disabled: !vm.version || vm.isDuplicating, onSelect: vm.duplicate }]
      : [{ kind: "item" as const, id: "copier", label: "Copier le texte", onSelect: () => void vm.copyLetter() }]),
    { kind: "separator", id: "sep" },
    {
      kind: "item",
      id: "supprimer",
      label: "Supprimer…",
      tone: "danger",
      onSelect: () => vm.askDelete(vm.current),
    },
  ];

  const empty = !vm.isLoading && !vm.error && vm.resumes.length === 0 && vm.letters.length === 0;

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-toolbar flex-none items-center gap-[7px] border-b border-bd-soft px-3.5">
        <div role="tablist" aria-label="Type de document" className="flex flex-none items-center gap-0.5">
          {TABS.map((tab) => (
            <button
              key={tab.value}
              type="button"
              role="tab"
              aria-selected={filter === tab.value}
              onClick={() => void navigate(tab.path)}
              className={cn(
                "inline-flex h-[23px] items-center gap-1.5 rounded-r6 px-2 text-small whitespace-nowrap",
                filter === tab.value ? "bg-elev font-medium text-tx" : "text-tx-3 hover:text-tx",
              )}
            >
              {tab.label}
              {counts[tab.value] !== undefined ? (
                <span className="font-mono text-[10px] text-tx-5">{counts[tab.value]}</span>
              ) : null}
            </button>
          ))}
        </div>
        <label className="ml-1.5 flex h-[23px] w-[150px] flex-none items-center gap-[7px] rounded-r6 border border-transparent bg-elev px-[9px] text-small transition-[width] duration-100 focus-within:w-[200px] focus-within:border-ac focus-within:bg-panel">
          <span aria-hidden className="flex-none text-tiny text-tx-5">
            ⌕
          </span>
          <input
            ref={searchInput}
            type="search"
            aria-label="Rechercher un document"
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
        <span className="ml-auto flex flex-none items-center gap-[7px]">
          <Button variant="ghost" size="compact" onClick={() => void navigate(PATHS.analyzeResume)}>
            Importer
          </Button>
          <Button size="compact" onClick={() => void navigate(PATHS.writeLetter)}>
            Générer une lettre
          </Button>
          <Button size="compact" onClick={() => void navigate(PATHS.baseResume)}>
            CV de base
          </Button>
          <Button variant="primary" size="compact" shortcut="n" onClick={() => void navigate(PATHS.generateResume)}>
            Générer un CV
          </Button>
        </span>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="flex min-w-0 flex-1 flex-col">
          {vm.error ? (
            <div className="p-3.5">
              <ErrorBanner
                message={vm.error instanceof AppError ? vm.error.message : "Les documents n'ont pas pu être chargés."}
                onRetry={vm.reload}
              />
            </div>
          ) : vm.isLoading ? (
            <div role="status" aria-label="Chargement des documents">
              {[168, 214, 142, 196, 178].map((width, index) => (
                <div key={width} className="flex h-10 items-center gap-[11px] px-3.5">
                  <Skeleton index={index} className="h-5 w-4 rounded-[2px]" />
                  <Skeleton index={index} className="h-[9px]" />
                  <span style={{ width }} aria-hidden />
                  <Skeleton index={index} className="ml-auto h-[9px] w-10" />
                </div>
              ))}
            </div>
          ) : empty ? (
            <div className="flex min-h-0 flex-1 items-start justify-center pt-[min(12vh,90px)]">
              <EmptyState
                title={vm.search ? "Aucun document ne correspond" : filter === "analyses" ? "Aucune analyse pour l'instant" : "Aucun document pour l'instant"}
                description={
                  vm.search
                    ? `Aucun document ne contient « ${vm.search} ».`
                    : filter === "analyses"
                      ? "Un CV généré pour une offre porte son score ATS : il apparaîtra ici."
                      : "Générez un CV ciblé ou une lettre depuis une offre : ils s'enregistrent ici, prêts à exporter en PDF."
                }
                action={
                  vm.search ? (
                    <Button size="empty" onClick={() => vm.setSearch("")}>
                      Effacer la recherche
                    </Button>
                  ) : (
                    <>
                      <Button variant="primary" size="empty" onClick={() => void navigate(PATHS.generateResume)}>
                        Générer un CV
                      </Button>
                      <Button variant="secondary" size="empty" onClick={() => void navigate(PATHS.writeLetter)}>
                        Générer une lettre
                      </Button>
                    </>
                  )
                }
              />
            </div>
          ) : (
            <div
              role="listbox"
              aria-label="Documents"
              tabIndex={0}
              onKeyDown={onKeyDown}
              className="min-h-0 flex-1 overflow-y-auto outline-none"
            >
              {vm.showResumes && vm.resumesTotal > 0 ? (
                <Group
                  label={filter === "analyses" ? "CV analysés" : "CV"}
                  count={vm.resumesTotal}
                  onAdd={() => void navigate(PATHS.generateResume)}
                  more={vm.resumesTotal - vm.resumes.length}
                  onMore={() => vm.showMore("resume")}
                >
                  {vm.resumes.map((resume) => (
                    <ResumeRow
                      key={resume.id}
                      resume={resume}
                      selected={isCurrent({ kind: "resume", id: resume.id })}
                      onSelect={() => vm.select({ kind: "resume", id: resume.id })}
                      onOpen={open}
                      onMenu={setMenu}
                    />
                  ))}
                </Group>
              ) : null}
              {vm.showLetters && vm.lettersTotal > 0 ? (
                <Group
                  label="Lettres de motivation"
                  count={vm.lettersTotal}
                  onAdd={() => void navigate(PATHS.writeLetter)}
                  more={vm.lettersTotal - vm.letters.length}
                  onMore={() => vm.showMore("letter")}
                >
                  {vm.letters.map((letter) => (
                    <LetterRow
                      key={letter.id}
                      letter={letter}
                      selected={isCurrent({ kind: "letter", id: letter.id })}
                      onSelect={() => vm.select({ kind: "letter", id: letter.id })}
                      onOpen={open}
                      onMenu={setMenu}
                    />
                  ))}
                </Group>
              ) : null}
            </div>
          )}
        </div>

        {vm.current ? (
          <aside
            aria-label="Fiche du document"
            className="hidden w-inspector flex-none flex-col overflow-y-auto border-l border-bd-soft bg-panel px-4 pt-3.5 wide:flex"
          >
            <div className="flex gap-[11px]">
              <Sheet large active />
              <div className="min-w-0 flex-1">
                <h2 className="text-lead leading-[1.3] font-medium text-tx">
                  {vm.resumeSummary?.name ?? vm.letter?.name}
                </h2>
                <p className="mt-0.5 text-sub text-tx-5">
                  {vm.resumeSummary
                    ? [vm.resumeSummary.target_title, `enregistré le ${shortDate(vm.resumeSummary.created_at)}`].filter(Boolean).join(" · ")
                    : [vm.letter?.job_title, vm.letter ? `ton ${labelTone(vm.letter.tone).toLowerCase()}` : null].filter(Boolean).join(" · ")}
                </p>
                <div className="mt-2 flex gap-[5px]">
                  <Button variant="primary" size="compact" disabled={!canOpen} onClick={open}>
                    Ouvrir
                  </Button>
                  <Button size="compact" disabled={!canOpen} onClick={() => void vm.exportPdf()}>
                    PDF
                  </Button>
                  <GlyphButton
                    glyph="⋯"
                    label="Autres actions"
                    onClick={(event) => setMenu(event.currentTarget.getBoundingClientRect())}
                  />
                </div>
                {vm.current.kind === "resume" && !vm.isLoadingVersion && !canOpen ? (
                  <p className="mt-1.5 text-tiny text-tx-5">Ancienne version sans contenu structuré : ni aperçu ni export.</p>
                ) : null}
              </div>
            </div>

            {vm.workspace || vm.generation ? <AtsSection vm={vm} onOpen={open} /> : null}

            <Section title="Détails">
              <dl className="grid grid-cols-[88px_minmax(0,1fr)] gap-x-1 gap-y-1.5 text-small">
                {vm.resumeSummary ? (
                  <>
                    <Detail label="Type" value="Curriculum vitæ" />
                    <Detail label="Offre ciblée" value={vm.resumeSummary.target_title} />
                    <Detail label="Enregistré" value={shortYearDate(vm.resumeSummary.created_at)} mono />
                  </>
                ) : vm.letter ? (
                  <>
                    <Detail label="Type" value="Lettre de motivation" />
                    <Detail label="Entreprise" value={vm.letter.company} />
                    <Detail label="Poste" value={vm.letter.job_title} />
                    <Detail label="Destinataire" value={vm.letter.recipient} />
                    <Detail label="Référence" value={vm.letter.job_reference} mono />
                    <Detail label="Ton" value={labelTone(vm.letter.tone)} />
                    <Detail label="Enregistrée" value={shortYearDate(vm.letter.created_at)} mono />
                  </>
                ) : null}
              </dl>
            </Section>

            <Section title="Versions" count={vm.versions.length}>
              {vm.versionsError ? (
                <p className="text-sub text-tx-6">{vm.versionsError}</p>
              ) : (
                <ul>
                  {vm.versions.map((version) => {
                    const label = (
                      <>
                        <span className="w-6 flex-none font-mono text-caps text-tx-5">v{version.version_number}</span>
                        <span className={cn("min-w-0 flex-1 truncate", version.is_current ? "text-tx" : "text-tx-3")}>
                          {version.note ?? "Version enregistrée"}
                        </span>
                        {version.is_current ? (
                          <span className="flex-none text-tiny text-tx-5">courante</span>
                        ) : null}
                        <span className="w-10 flex-none text-right font-mono text-caps text-tx-6">
                          {shortDate(version.created_at)}
                        </span>
                      </>
                    );
                    return (
                      <li key={version.id}>
                        {version.is_current ? (
                          <div className="-mx-2 flex h-7 items-center gap-2 px-2 text-small">{label}</div>
                        ) : (
                          <button
                            type="button"
                            title={`Revenir à la version v${version.version_number}`}
                            disabled={vm.isRestoring}
                            onClick={() => vm.askRestore(version)}
                            className="-mx-2 flex h-7 w-[calc(100%+16px)] items-center gap-2 rounded-r6 px-2 text-left text-small hover:bg-hover"
                          >
                            {label}
                          </button>
                        )}
                      </li>
                    );
                  })}
                </ul>
              )}
            </Section>
            <div className="h-4 flex-none" />
          </aside>
        ) : null}
      </div>

      <Menu open={menu !== null} anchor={menu} label="Autres actions" entries={menuEntries} onClose={() => setMenu(null)} />

      <ConfirmDialog
        open={vm.pendingDelete !== null}
        title={vm.pendingDelete?.kind === "letter" ? "Supprimer cette lettre ?" : "Supprimer ce CV ?"}
        description={
          vm.pendingDelete?.id === vm.current?.id && vm.versions.length > 1
            ? `Le document et ses ${vm.versions.length} versions disparaissent définitivement de la bibliothèque locale.`
            : "Le document disparaît définitivement de la bibliothèque locale."
        }
        note="Votre profil et vos autres documents sont conservés."
        footnote="action définitive"
        busy={vm.isDeleting}
        onCancel={() => vm.askDelete(null)}
        onConfirm={vm.confirmDelete}
      />

      <ConfirmDialog
        open={vm.pendingRestore !== null}
        register="confirmation"
        title={`Revenir à la version v${vm.pendingRestore?.version_number ?? ""} ?`}
        description={`La version v${vm.pendingRestore?.version_number ?? ""} deviendra la version courante du document. Les autres versions ne sont pas supprimées : elles restent dans l'historique.`}
        consequences={[
          ...(currentVersion
            ? [{ label: "Version courante", value: `v${currentVersion.version_number} · ${shortDate(currentVersion.created_at)}` }]
            : []),
          ...(vm.pendingRestore
            ? [{ label: "Version restaurée", value: `v${vm.pendingRestore.version_number} · ${shortDate(vm.pendingRestore.created_at)}` }]
            : []),
          { label: "Versions conservées", value: String(vm.versions.length) },
        ]}
        footnote={currentVersion ? `réversible : v${currentVersion.version_number} reste dans l'historique` : undefined}
        confirmLabel={`Restaurer v${vm.pendingRestore?.version_number ?? ""}`}
        busy={vm.isRestoring}
        onCancel={() => vm.askRestore(null)}
        onConfirm={vm.confirmRestore}
      />
    </div>
  );
}

