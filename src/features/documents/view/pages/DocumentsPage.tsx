import { useRef, useState } from "react";
import type { KeyboardEvent, ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import type { CoverLetter, ResumeSummary } from "@/shared/types/generated/documents";
import { documentsService } from "../../services/documentsService";
import { COVER_LETTERS_KEY, RESUME_KEY } from "../../viewmodel/documentKeys";
import { useDocumentsViewModel } from "../../viewmodel/useDocumentsViewModel";
import type { DocumentFilter, DocumentSelection } from "../../viewmodel/useDocumentsViewModel";
import { labelTone } from "./documentPageSupport";
import { PATHS } from "@/shared/lib/paths";
import { useChrome } from "@/shared/lib/chrome";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { AppError } from "@/shared/types/app-error";
import { cn } from "@/shared/lib/cn";
import { Avatar, Button, ConfirmDialog, EmptyState, ErrorBanner, GlyphButton, Kbd, Menu, Skeleton } from "@/shared/ui";
import type { MenuAnchor, MenuEntry } from "@/shared/ui";

const TABS: ReadonlyArray<{ value: DocumentFilter; label: string; path: string }> = [
  { value: "all", label: "Tous", path: PATHS.documents },
  { value: "resumes", label: "CV", path: PATHS.resumes },
  { value: "letters", label: "Lettres", path: PATHS.letters },
  { value: "analyses", label: "Analyses", path: PATHS.analyses },
];

/** `AAAA-MM-JJ…` → `JJ-MM`. */
function shortDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}`;
}

/** `AAAA-MM-JJ…` → `JJ-MM-AA`. */
function shortYearDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}-${iso.slice(2, 4)}`;
}

/** Teinte d'un score ATS : bon dès 80, correct dès 65, faible en dessous. */
function scoreTone(score: number): { chip: string; text: string; bar: string } {
  if (score >= 80) return { chip: "bg-tint-g-bg text-tint-g-tx", text: "text-tint-g-tx", bar: "bg-st-g" };
  if (score >= 65) return { chip: "bg-tint-ac-bg text-tint-ac-tx", text: "text-ac-tx", bar: "bg-ac" };
  return { chip: "bg-tint-c-bg text-tint-c-tx", text: "text-tint-c-tx", bar: "bg-st-c" };
}

/** Pictogramme de feuille, miniature des lignes et de l'inspecteur. */
function Sheet({ large = false, active = false }: { large?: boolean; active?: boolean }) {
  return (
    <span
      aria-hidden
      className={cn(
        "flex flex-none flex-col gap-[2px] rounded-[2px] border bg-panel",
        large ? "h-[62px] w-[46px] gap-[3px] px-[7px] pt-[8px]" : "h-5 w-4 px-[3px] pt-[4px]",
        active ? "border-ac" : "border-bd-menu",
      )}
    >
      {[1, 0.8, 0.9, 0.6].map((width, index) => (
        <span key={index} className="h-px bg-tx-6" style={{ width: `${width * 100}%` }} />
      ))}
    </span>
  );
}

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

  // Décomptes des onglets : mêmes clés que la navigation pour CV et lettres.
  const resumeCount = useQuery({
    queryKey: [...RESUME_KEY, "navigation"],
    queryFn: () => documentsService.listResumePage({ page: 1, page_size: 1, search: "" }),
  });
  const letterCount = useQuery({
    queryKey: [...COVER_LETTERS_KEY, "navigation"],
    queryFn: () => documentsService.listCoverLettersPage({ page: 1, page_size: 1, search: "" }),
  });
  const analysisCount = useQuery({
    queryKey: [...RESUME_KEY, "navigation", "analyses"],
    queryFn: () => documentsService.listResumePage({ page: 1, page_size: 1, search: "", scored_only: true }),
  });
  const counts: Record<DocumentFilter, number | undefined> = {
    all:
      resumeCount.data && letterCount.data ? resumeCount.data.total + letterCount.data.total : undefined,
    resumes: resumeCount.data?.total,
    letters: letterCount.data?.total,
    analyses: analysisCount.data?.total,
  };

  const open = () => {
    if (vm.workspace) void navigate(PATHS.generateResume, { state: { workspace: vm.workspace, name: vm.version?.name } });
    else if (vm.generation) void navigate(PATHS.generateResume, { state: { generation: vm.generation, name: vm.version?.name } });
    else if (vm.letter) void navigate(PATHS.writeLetter, { state: { cover_letter: vm.letter } });
  };
  const canOpen = Boolean(vm.workspace ?? vm.generation ?? vm.letter);

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
                icon="description"
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
            <div className="h-4 flex-none" />
          </aside>
        ) : null}
      </div>

      <Menu open={menu !== null} anchor={menu} label="Autres actions" entries={menuEntries} onClose={() => setMenu(null)} />

      <ConfirmDialog
        open={vm.pendingDelete !== null}
        title={vm.pendingDelete?.kind === "letter" ? "Supprimer cette lettre ?" : "Supprimer ce CV ?"}
        description="Le document disparaît définitivement de la bibliothèque locale."
        note="Votre profil et vos autres documents sont conservés."
        footnote="action définitive"
        busy={vm.isDeleting}
        onCancel={() => vm.askDelete(null)}
        onConfirm={vm.confirmDelete}
      />
    </div>
  );
}

function Group({
  label,
  count,
  more,
  onAdd,
  onMore,
  children,
}: {
  label: string;
  count: number;
  more: number;
  onAdd: () => void;
  onMore: () => void;
  children: ReactNode;
}) {
  return (
    <section aria-label={`${label}, ${count}`}>
      <div className="sticky top-0 z-[1] flex h-group-head items-center gap-[9px] bg-group px-3.5">
        <Sheet />
        <span className="text-small font-medium text-tx">{label}</span>
        <span className="font-mono text-caps text-tx-5">{count}</span>
        <button
          type="button"
          tabIndex={-1}
          aria-label={`Nouveau document : ${label}`}
          onClick={onAdd}
          className="ml-auto flex size-[18px] items-center justify-center rounded-r5 text-small text-tx-5 hover:bg-elev hover:text-tx-2"
        >
          +
        </button>
      </div>
      {children}
      {more > 0 ? (
        <button
          type="button"
          tabIndex={-1}
          onClick={onMore}
          className="flex h-10 w-full items-center px-3.5 text-left text-small text-ac-tx hover:bg-hover"
        >
          Afficher plus
          <span className="ml-2 font-mono text-caps text-tx-5">{more} restants</span>
        </button>
      ) : null}
    </section>
  );
}

function Row({
  selected,
  name,
  sub,
  chips,
  date,
  avatar,
  onSelect,
  onOpen,
  onMenu,
}: {
  selected: boolean;
  name: string;
  sub: string;
  chips: ReactNode;
  date: string;
  avatar: string | null;
  onSelect: () => void;
  onOpen: () => void;
  onMenu: (anchor: MenuAnchor) => void;
}) {
  return (
    <div
      role="option"
      onContextMenu={(event) => {
        event.preventDefault();
        onSelect();
        onMenu({ x: event.clientX, y: event.clientY });
      }}
      aria-selected={selected}
      aria-label={name}
      onClick={onSelect}
      onDoubleClick={onOpen}
      className={cn(
        "flex h-10 cursor-default items-center gap-[11px] px-3.5",
        selected ? "row-selected" : "hover:bg-hover",
      )}
    >
      <Sheet active={selected} />
      <span className="min-w-0 flex-1">
        <span className={cn("block truncate text-row", selected ? "font-medium text-tx" : "text-tx-2")}>{name}</span>
        <span className="block truncate text-sub text-tx-5">{sub}</span>
      </span>
      <span className="flex flex-none items-center gap-1.5">{chips}</span>
      <span className="w-10 flex-none text-right font-mono text-caps text-tx-5">{date}</span>
      <span className="flex w-[19px] flex-none justify-center">
        {avatar ? <Avatar name={avatar} kind="company" size={19} /> : <span className="text-small text-tx-7">—</span>}
      </span>
    </div>
  );
}

function ResumeRow({
  resume,
  selected,
  onSelect,
  onOpen,
  onMenu,
}: {
  resume: ResumeSummary;
  selected: boolean;
  onSelect: () => void;
  onOpen: () => void;
  onMenu: (anchor: MenuAnchor) => void;
}) {
  return (
    <Row
      selected={selected}
      name={resume.name}
      sub={resume.target_title ? `${resume.target_title} · enregistré le ${shortDate(resume.created_at)}` : "Sans offre ciblée · base de départ"}
      chips={
        resume.ats_score !== null ? (
          <span className={cn("inline-flex h-[18px] items-center rounded-r5 px-1.5 font-mono text-[10px]", scoreTone(resume.ats_score).chip)}>
            ATS {resume.ats_score}
          </span>
        ) : null
      }
      date={shortDate(resume.created_at)}
      avatar={null}
      onSelect={onSelect}
      onOpen={onOpen}
      onMenu={onMenu}
    />
  );
}

function LetterRow({
  letter,
  selected,
  onSelect,
  onOpen,
  onMenu,
}: {
  letter: CoverLetter;
  selected: boolean;
  onSelect: () => void;
  onOpen: () => void;
  onMenu: (anchor: MenuAnchor) => void;
}) {
  return (
    <Row
      selected={selected}
      name={letter.name}
      sub={[letter.job_title, `ton ${labelTone(letter.tone).toLowerCase()}`].filter(Boolean).join(" · ")}
      chips={null}
      date={shortDate(letter.created_at)}
      avatar={letter.company}
      onSelect={onSelect}
      onOpen={onOpen}
      onMenu={onMenu}
    />
  );
}

function AtsSection({ vm, onOpen }: { vm: ReturnType<typeof useDocumentsViewModel>; onOpen: () => void }) {
  const score = vm.workspace?.score ?? vm.generation?.profile_score ?? null;
  if (!score) return null;
  const total = Math.round(score.total);
  const tone = scoreTone(total);
  const missing = score.missing.slice(0, 4);
  const findings = [
    score.present.length > 0
      ? { tone: "bg-st-g", text: `${score.present.length} exigence${score.present.length > 1 ? "s" : ""} de l'offre couverte${score.present.length > 1 ? "s" : ""} par le CV.` }
      : null,
    missing.length > 0
      ? { tone: "bg-st-a", text: `${missing.length === 1 ? "Une compétence de l'offre manque" : `${missing.length} compétences de l'offre manquent`} : ${missing.join(", ")}.` }
      : null,
  ].filter((item): item is { tone: string; text: string } => item !== null);

  return (
    <section className="mt-4 border-t border-bd-soft pt-3.5" aria-label="Score ATS">
      <p className="flex items-baseline gap-1.5">
        <span className={cn("serif-title text-[24px] leading-none", tone.text)}>{total}</span>
        <span className="text-sub text-tx-5">/ 100 · lisibilité par les robots de tri</span>
      </p>
      <div aria-hidden className="mt-2 flex gap-0.5">
        {Array.from({ length: 10 }, (_, index) => (
          <span key={index} className={cn("h-[3px] flex-1 rounded-r2", index < Math.round(total / 10) ? tone.bar : "bg-chip")} />
        ))}
      </div>
      {findings.length > 0 ? (
        <ul className="mt-3 flex flex-col gap-2">
          {findings.map((item) => (
            <li key={item.text} className="flex gap-2 text-small leading-[1.45] text-tx-3">
              <span aria-hidden className={cn("mt-[5px] size-[7px] flex-none rounded-full", item.tone)} />
              {item.text}
            </li>
          ))}
        </ul>
      ) : null}
      <button
        type="button"
        onClick={onOpen}
        className="mt-3 flex h-7 w-full items-center rounded-r7 bg-chip px-2.5 text-left text-small text-tx-2 hover:text-tx"
      >
        Voir l'analyse complète
        <span aria-hidden className="ml-auto text-tx-5">
          ›
        </span>
      </button>
    </section>
  );
}

function Detail({ label, value, mono = false }: { label: string; value: string | null | undefined; mono?: boolean }) {
  return (
    <>
      <dt className="text-tx-5">{label}</dt>
      <dd className={cn("min-w-0 truncate", mono && "font-mono text-caps", value ? "text-tx-2" : "text-tx-6")}>
        {value ?? "—"}
      </dd>
    </>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mt-4 border-t border-bd-soft pt-3">
      <h3 className="caps mb-2">{title}</h3>
      {children}
    </section>
  );
}
