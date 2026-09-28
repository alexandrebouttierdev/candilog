import { useCallback, useRef, useState } from "react";
import { CompanySizes, useReferentials } from "@/features/referentials";
import { Roles } from "@/features/contacts";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { Kbd, Menu } from "@/shared/ui";
import type { MenuAnchor, MenuEntry } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import type { RelationCriteria, RelationKind } from "../../viewmodel/useRelationsViewModel";

type Field = "sector_id" | "company_type_id" | "company_size" | "tracking_role";

const LABELS: Record<Field, string> = {
  sector_id: "Secteur",
  company_type_id: "Type d'entreprise",
  company_size: "Taille",
  tracking_role: "Rôle",
};

/**
 * Puces et menu « + Filtre » (`F`) de Relations. Les critères de la v1 sont conservés —
 * secteur, type et taille d'une entreprise ; rôle d'un contact —, chacun à une valeur,
 * comme le backend les accepte.
 */
export function RelationFilters({
  kind,
  criteria,
  onChange,
}: {
  kind: RelationKind;
  criteria: RelationCriteria;
  onChange: (criteria: RelationCriteria) => void;
}) {
  const referentials = useReferentials().data;
  const [anchor, setAnchor] = useState<MenuAnchor | null>(null);
  const [field, setField] = useState<Field | null>(null);
  const button = useRef<HTMLButtonElement>(null);
  const close = useCallback(() => setAnchor(null), []);

  const fields: readonly Field[] =
    kind === "companies" ? ["sector_id", "company_type_id", "company_size"] : ["tracking_role"];

  const options = (key: Field): ReadonlyArray<{ value: string; label: string }> => {
    switch (key) {
      case "sector_id":
        return referentials.sectors.map((item) => ({ value: item.id, label: item.name }));
      case "company_type_id":
        return referentials.company_types.map((item) => ({ value: item.code, label: item.name }));
      case "company_size":
        return CompanySizes.map((item) => ({ value: item.value, label: item.label }));
      case "tracking_role":
        return Roles.map((role) => ({ value: role, label: role }));
    }
  };

  const open = () => {
    const rect = button.current?.getBoundingClientRect();
    if (!rect) return;
    setField(null);
    setAnchor(rect);
  };
  useShortcut("f", open);

  const entries: MenuEntry[] =
    field === null
      ? fields.map((key) => ({
          kind: "item",
          id: `champ-${key}`,
          label: LABELS[key],
          meta: String(options(key).length),
          keepOpen: true,
          onSelect: () => setField(key),
        }))
      : options(field).map((option) => ({
          kind: "item",
          id: `valeur-${option.value}`,
          label: option.label,
          checked: criteria[field] === option.value,
          onSelect: () =>
            onChange({ ...criteria, [field]: criteria[field] === option.value ? null : option.value }),
        }));

  const active = fields.filter((key) => criteria[key] !== null);

  return (
    <>
      {active.map((key) => (
        <span
          key={key}
          className="inline-flex h-[23px] max-w-[260px] flex-none items-center gap-1.5 rounded-r6 bg-elev pr-1 pl-2 text-small text-tx-2"
        >
          <span className="flex-none">{LABELS[key]}</span>
          <span className="flex-none text-tx-4">est</span>
          <span className="truncate font-medium text-tx">
            {options(key).find((option) => option.value === criteria[key])?.label ?? criteria[key]}
          </span>
          <button
            type="button"
            aria-label={`Retirer le filtre ${LABELS[key]}`}
            onClick={() => onChange({ ...criteria, [key]: null })}
            className="flex size-4 flex-none items-center justify-center rounded-r4 text-[10px] text-tx-5 hover:bg-hover hover:text-tx-2"
          >
            ✕
          </button>
        </span>
      ))}
      <button
        ref={button}
        type="button"
        aria-haspopup="menu"
        aria-expanded={anchor !== null}
        aria-keyshortcuts="F"
        onClick={open}
        className={cn(
          "inline-flex h-[23px] flex-none items-center gap-[5px] rounded-r6 px-2 text-small text-tx-5 hover:text-tx-3",
          anchor && "bg-elev",
        )}
      >
        <span aria-hidden className="text-tiny">
          +
        </span>
        Filtre
        <Kbd shortcut="f" tone="ghost" decorative />
      </button>
      <Menu
        open={anchor !== null}
        anchor={anchor}
        label="Filtrer les relations"
        entries={entries}
        onClose={close}
        {...(field === null ? {} : { onBack: () => setField(null) })}
        header={
          <div className="flex items-center gap-2 px-[9px] pt-1 pb-1.5">
            {field ? (
              <button type="button" aria-label="Revenir aux champs" onClick={() => setField(null)} className="text-tiny text-tx-5 hover:text-tx">
                ‹
              </button>
            ) : null}
            <span className="caps">{field ? LABELS[field].toUpperCase() : "FILTRER PAR"}</span>
          </div>
        }
      />
    </>
  );
}
