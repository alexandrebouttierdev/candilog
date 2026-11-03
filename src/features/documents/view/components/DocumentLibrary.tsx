/**
 * Briques de la bibliothèque de documents : lignes de CV et de lettres, groupes, blocs de
 * l'inspecteur (score ATS, détails, sections) et formats de date courts. Extraites de
 * `DocumentsPage`, qui ne garde que l'orchestration de l'écran.
 */
import type { ReactNode } from "react";
import type { CoverLetter, ResumeSummary } from "@/shared/types/generated/documents";
import type { useDocumentsViewModel } from "../../viewmodel/useDocumentsViewModel";
import { labelTone } from "../pages/documentPageSupport";
import { cn } from "@/shared/lib/cn";
import { Avatar } from "@/shared/ui";
import type { MenuAnchor } from "@/shared/ui";
/** `AAAA-MM-JJ…` → `JJ-MM`. */
export function shortDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}`;
}

/** `AAAA-MM-JJ…` → `JJ-MM-AA`. */
export function shortYearDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}-${iso.slice(2, 4)}`;
}

/** Teinte d'un score ATS : bon dès 80, correct dès 65, faible en dessous. */
export function scoreTone(score: number): { chip: string; text: string; bar: string } {
  if (score >= 80) return { chip: "bg-tint-g-bg text-tint-g-tx", text: "text-tint-g-tx", bar: "bg-st-g" };
  if (score >= 65) return { chip: "bg-tint-ac-bg text-tint-ac-tx", text: "text-ac-tx", bar: "bg-ac" };
  return { chip: "bg-tint-c-bg text-tint-c-tx", text: "text-tint-c-tx", bar: "bg-st-c" };
}

/** Pictogramme de feuille, miniature des lignes et de l'inspecteur. */
export function Sheet({ large = false, active = false }: { large?: boolean; active?: boolean }) {
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


export function Group({
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

export function Row({
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

export function ResumeRow({
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

export function LetterRow({
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

export function AtsSection({ vm, onOpen }: { vm: ReturnType<typeof useDocumentsViewModel>; onOpen: () => void }) {
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

export function Detail({ label, value, mono = false }: { label: string; value: string | null | undefined; mono?: boolean }) {
  return (
    <>
      <dt className="text-tx-5">{label}</dt>
      <dd className={cn("min-w-0 truncate", mono && "font-mono text-caps", value ? "text-tx-2" : "text-tx-6")}>
        {value ?? "—"}
      </dd>
    </>
  );
}

export function Section({ title, count, children }: { title: string; count?: number; children: ReactNode }) {
  return (
    <section className="mt-4 border-t border-bd-soft pt-3">
      <h3 className="caps mb-2 flex items-center">
        {title}
        {count !== undefined ? <span className="ml-auto font-mono text-tx-7">{count}</span> : null}
      </h3>
      {children}
    </section>
  );
}
