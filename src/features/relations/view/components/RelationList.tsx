import { useRef, useState } from "react";
import type { KeyboardEvent, ReactNode } from "react";
import type { Company } from "@/features/companies";
import type { Contact } from "@/features/contacts";
import { formatReference } from "@/features/applications";
import { Avatar, Skeleton, StatusGlyph } from "@/shared/ui";
import type { GlyphTone } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

/** Ligne affichable, commune aux entreprises et aux contacts. */
export interface RelationRow {
  readonly id: string;
  readonly name: string;
  readonly sub: string;
  readonly avatarKind: "company" | "person";
  /** Colonne de rattachement : « 2 contacts » ou l'entreprise du contact. */
  readonly link: ReactNode;
  readonly reference: number | null;
  readonly date: string | null;
}

export interface RelationGroup {
  readonly key: string;
  readonly label: string;
  readonly note: string;
  readonly glyph: GlyphTone;
  readonly rows: readonly RelationRow[];
  readonly total: number;
}

function shortDate(iso: string | null): string {
  return iso ? `${iso.slice(8, 10)}-${iso.slice(5, 7)}` : "";
}

const NONE = <span className="text-small text-tx-7">—</span>;

export function companyRow(company: Company): RelationRow {
  const contacts = company.activity.contacts;
  return {
    id: company.id,
    name: company.name,
    sub: [company.sector_name, company.city].filter(Boolean).join(" · ") || "Secteur non renseigné",
    avatarKind: "company",
    link:
      contacts > 0 ? (
        <span className="inline-flex h-5 items-center rounded-r5 bg-chip px-[7px] text-tiny whitespace-nowrap text-tx-3">
          {contacts} contact{contacts > 1 ? "s" : ""}
        </span>
      ) : (
        NONE
      ),
    reference: company.activity.last_reference_number,
    date: company.activity.last_sent_date,
  };
}

export function contactRow(contact: Contact): RelationRow {
  return {
    id: contact.id,
    name: `${contact.first_name} ${contact.name}`.trim(),
    sub: [contact.job_title, contact.tracking_role].filter(Boolean).join(" · ") || "Rôle non renseigné",
    avatarKind: "person",
    link: contact.company_name ? (
      <span className="flex min-w-0 items-center gap-2">
        <Avatar name={contact.company_name} kind="company" size={19} />
        <span className="truncate text-small text-tx-3">{contact.company_name}</span>
      </span>
    ) : (
      NONE
    ),
    reference: contact.activity.last_reference_number,
    date: contact.activity.last_sent_date,
  };
}

/**
 * Liste groupée de Relations (`screens/05-companies.png`) : en-têtes de groupe sur
 * `bg-group` avec leur note, lignes de 38 px — avatar, nom et sous-titre, rattachement,
 * référence de la dernière candidature, date. Composite ARIA : `↑ ↓` parcourent les lignes,
 * `⏎` ouvre la fiche.
 */
export function RelationList({
  groups,
  selectedId,
  loading,
  onSelect,
  onOpen,
  onShowMore,
}: {
  groups: readonly RelationGroup[];
  selectedId: string | null;
  loading: boolean;
  onSelect: (id: string) => void;
  onOpen: (id: string) => void;
  onShowMore: (key: string) => void;
}) {
  const [focusId, setFocusId] = useState<string | null>(null);
  const container = useRef<HTMLDivElement>(null);
  const rows = groups.flatMap((group) => group.rows);
  const focused = rows.find((row) => row.id === focusId) ?? null;

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const current = focused ?? rows.find((row) => row.id === selectedId) ?? null;
      const index = current ? rows.indexOf(current) : -1;
      const next = rows[Math.min(Math.max(index + (event.key === "ArrowDown" ? 1 : -1), 0), rows.length - 1)];
      if (!next) return;
      setFocusId(next.id);
      onSelect(next.id);
      container.current?.querySelector(`[data-row="${next.id}"]`)?.scrollIntoView({ block: "nearest" });
    } else if (event.key === "Enter") {
      const target = focused ?? rows.find((row) => row.id === selectedId);
      if (target) {
        event.preventDefault();
        onOpen(target.id);
      }
    }
  };

  if (loading) {
    return (
      <div role="status" aria-label="Chargement des relations">
        <div className="flex h-group-head items-center gap-[9px] bg-group px-3.5">
          <Skeleton className="size-[11px] rounded-full" />
          <Skeleton className="h-[9px] w-[76px]" />
        </div>
        {[168, 214, 142, 196, 178, 232, 156].map((width, index) => (
          <div key={width} className="flex h-[38px] items-center gap-[11px] px-3.5">
            <Skeleton index={index} className="size-6 rounded-r6" />
            <Skeleton index={index} className="h-[9px]" />
            <span style={{ width }} aria-hidden />
            <Skeleton index={index} className="ml-auto h-[9px] w-[46px]" />
          </div>
        ))}
      </div>
    );
  }

  return (
    <div
      ref={container}
      role="listbox"
      aria-label="Relations"
      tabIndex={0}
      aria-activedescendant={focused ? `relation-${focused.id}` : undefined}
      onKeyDown={onKeyDown}
      className="@container min-h-0 flex-1 overflow-y-auto outline-none"
    >
      {groups.map((group) =>
        group.total === 0 ? null : (
          <section key={group.key} aria-label={`${group.label}, ${group.total}`}>
            <div className="sticky top-0 z-[1] flex h-group-head items-center gap-[9px] bg-group px-3.5">
              <StatusGlyph tone={group.glyph} />
              <span className="text-small font-medium text-tx">{group.label}</span>
              <span className="font-mono text-caps text-tx-5">{group.total}</span>
              <span className="ml-auto truncate text-sub text-tx-6">{group.note}</span>
            </div>
            {group.rows.map((row) => {
              const selected = row.id === selectedId;
              return (
                <div
                  key={row.id}
                  id={`relation-${row.id}`}
                  data-row={row.id}
                  role="option"
                  aria-selected={selected}
                  onPointerDown={() => setFocusId(row.id)}
                  onClick={() => onSelect(row.id)}
                  onDoubleClick={() => onOpen(row.id)}
                  className={cn(
                    "grid h-[38px] cursor-default items-center gap-x-[11px] px-3.5",
                    "grid-cols-[24px_minmax(0,1fr)_minmax(0,150px)_72px_40px] @min-[760px]:grid-cols-[24px_minmax(0,1fr)_minmax(0,190px)_80px_44px]",
                    selected ? "row-selected" : "hover:bg-hover",
                    row.id === focused?.id && "row-focus",
                  )}
                >
                  <Avatar name={row.name} kind={row.avatarKind} size={24} />
                  <span className="min-w-0">
                    <span className={cn("block truncate text-row", selected ? "font-medium text-tx" : "text-tx-2")}>
                      {row.name}
                    </span>
                    <span className="block truncate text-sub text-tx-5">{row.sub}</span>
                  </span>
                  <span className="flex min-w-0 items-center">{row.link}</span>
                  <span className="flex min-w-0">
                    {row.reference !== null ? (
                      <span className="inline-flex h-5 items-center rounded-r5 bg-chip px-[7px] font-mono text-caps whitespace-nowrap text-tx-4">
                        {formatReference(row.reference)}
                      </span>
                    ) : (
                      NONE
                    )}
                  </span>
                  <span className="text-right font-mono text-caps text-tx-5">{shortDate(row.date)}</span>
                </div>
              );
            })}
            {group.total > group.rows.length ? (
              <button
                type="button"
                tabIndex={-1}
                onClick={() => onShowMore(group.key)}
                className="flex h-[38px] w-full items-center px-3.5 text-left text-small text-ac-tx hover:bg-hover"
              >
                Afficher plus
                <span className="ml-2 font-mono text-caps text-tx-5">
                  {group.total - group.rows.length} restantes
                </span>
              </button>
            ) : null}
          </section>
        ),
      )}
    </div>
  );
}
