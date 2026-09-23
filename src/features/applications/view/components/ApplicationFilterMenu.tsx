import { useState } from "react";
import type { ReactNode } from "react";
import { Menu, StatusGlyph } from "@/shared/ui";
import type { MenuAnchor, MenuEntry } from "@/shared/ui";
import { FORMAT_DATE, toIsoDate } from "@/shared/lib/dates";
import { cn } from "@/shared/lib/cn";
import type { ApplicationFilterValues } from "../../model/schemas/application-filter.schema";
import { MAX_WEEKLY_HOURS } from "../../model/schemas/application-form.schema";
import {
  FIELD_LABELS,
  FIELD_ORDER,
  LIST_KEYS,
  SENT_PRESETS,
  daysAgo,
  isActive,
  isListKey,
  toggleValue,
} from "../../model/filterFields";
import type { FilterKey } from "../../model/filterFields";
import { useCompanyFilterOptions, useFilterOptions } from "../../viewmodel/useFilterOptions";

/** Second niveau ouvert : un champ, ou la saisie d'une période précise. */
type Level = FilterKey | "sent_range" | null;

/** Nombre saisi au clavier, virgule décimale acceptée ; `undefined` si vide. */
function parseHours(raw: string): number | null | undefined {
  const trimmed = raw.trim();
  if (trimmed === "") return undefined;
  const hours = Number(trimmed.replace(",", "."));
  return Number.isFinite(hours) && hours > 0 && hours <= MAX_WEEKLY_HOURS ? hours : null;
}

function isoToDisplay(iso: string | null): string {
  return iso ? `${iso.slice(8, 10)}-${iso.slice(5, 7)}-${iso.slice(0, 4)}` : "";
}

/**
 * Menu « + Filtre » à deux niveaux (`INTERACTIONS.md` §3.2) : champ, puis valeur.
 *
 * Les valeurs d'un critère multiple se cochent sans refermer le menu ; `←` revient aux
 * champs, `Échap` ferme. Les critères saisis (poste, ville, heures, période) valident leur
 * saisie avant de l'appliquer : une borne invalide ne part jamais au backend.
 */
export function ApplicationFilterMenu({
  open,
  anchor,
  filters,
  onApply,
  onClose,
}: {
  open: boolean;
  anchor: MenuAnchor | null;
  filters: ApplicationFilterValues;
  onApply: (filters: ApplicationFilterValues) => void;
  onClose: () => void;
}) {
  const [level, setLevel] = useState<Level>(null);
  const [text, setText] = useState("");
  const [second, setSecond] = useState("");
  const [error, setError] = useState<string | null>(null);
  const { options } = useFilterOptions(null);
  const companies = useCompanyFilterOptions(text, open && level === "company");

  // Un menu rouvert repart toujours du premier niveau.
  const [wasOpen, setWasOpen] = useState(open);
  if (open !== wasOpen) {
    setWasOpen(open);
    if (open) setLevel(null);
  }

  const goTo = (next: Level) => {
    setLevel(next);
    setError(null);
    if (next === "job_title") setText(filters.job_title);
    else if (next === "city") setText(filters.city);
    else if (next === "hours") {
      setText(filters.min_weekly_hours === null ? "" : String(filters.min_weekly_hours).replace(".", ","));
      setSecond(filters.max_weekly_hours === null ? "" : String(filters.max_weekly_hours).replace(".", ","));
    } else if (next === "sent_range") {
      setText(isoToDisplay(filters.start_date));
      setSecond(isoToDisplay(filters.end_date));
    } else setText("");
  };

  const applyAndClose = (next: ApplicationFilterValues) => {
    onApply(next);
    onClose();
  };

  const applyText = (key: "job_title" | "city") => {
    const value = text.trim();
    if (value === "") return setError("Saisissez un texte à rechercher.");
    applyAndClose({ ...filters, [key]: value });
  };

  const applyHours = () => {
    const min = parseHours(text);
    const max = parseHours(second);
    if (min === null || max === null) {
      return setError(`Indiquez un nombre d'heures entre 0 et ${MAX_WEEKLY_HOURS}.`);
    }
    if (min === undefined && max === undefined) return setError("Indiquez au moins une borne.");
    if (min !== undefined && max !== undefined && min > max) {
      return setError("Le maximum d'heures est inférieur au minimum.");
    }
    applyAndClose({ ...filters, min_weekly_hours: min ?? null, max_weekly_hours: max ?? null });
  };

  const applyRange = () => {
    const start = text.trim() === "" ? null : toIsoDate(text);
    const end = second.trim() === "" ? null : toIsoDate(second);
    if ((text.trim() !== "" && start === null) || (second.trim() !== "" && end === null)) {
      return setError(`Date invalide — format attendu ${FORMAT_DATE}.`);
    }
    if (start === null && end === null) return setError("Indiquez au moins une date.");
    if (start && end && start > end) return setError("La fin de période précède son début.");
    applyAndClose({ ...filters, start_date: start, end_date: end });
  };

  const header = (title: string, back: boolean) => (
    <div className="flex items-center gap-2 px-[9px] pt-1 pb-1.5">
      {back ? (
        <button
          type="button"
          aria-label="Revenir aux champs"
          onClick={() => goTo(null)}
          className="text-tiny text-tx-5 hover:text-tx"
        >
          ‹
        </button>
      ) : null}
      <span className="caps">{title}</span>
    </div>
  );

  const field = (props: {
    label: string;
    value: string;
    placeholder: string;
    inputMode?: "decimal";
    autoFocus?: boolean;
    onChange: (value: string) => void;
  }) => (
    <input
      aria-label={props.label}
      value={props.value}
      placeholder={props.placeholder}
      inputMode={props.inputMode}
      // Le champ est le seul contenu utile du niveau : il prend le focus à l'ouverture.
      autoFocus={props.autoFocus}
      aria-invalid={error !== null || undefined}
      onChange={(event) => {
        props.onChange(event.target.value);
        setError(null);
      }}
      className={cn(
        "h-7 w-full min-w-0 rounded-r6 border bg-panel px-2 text-ui text-tx outline-none placeholder:text-tx-6",
        error ? "border-st-c" : "border-bd-menu focus:border-ac",
      )}
    />
  );

  const errorLine = error ? (
    <p role="alert" className="px-[9px] pb-1 text-tiny text-st-c">
      {error}
    </p>
  ) : null;

  let title = "Filtrer par";
  let content: ReactNode = null;
  let entries: MenuEntry[];

  if (level === null) {
    entries = FIELD_ORDER.map((key) => ({
      kind: "item",
      id: `champ-${key}`,
      label: FIELD_LABELS[key],
      meta: isListKey(key)
        ? String(options(key).length)
        : { company: "liste", sent: "délai", hours: "h", job_title: "texte", city: "texte" }[key],
      leading: isActive(filters, key) ? (
        <span aria-hidden className="size-1.5 flex-none rounded-full bg-ac" />
      ) : (
        <span aria-hidden className="size-1.5 flex-none" />
      ),
      keepOpen: true,
      onSelect: () => goTo(key),
    }));
  } else if (level === "sent_range") {
    title = "Envoyée — période";
    content = (
      <div className="flex flex-col gap-1.5 px-[9px] pb-1.5">
        {field({ label: "Du", value: text, placeholder: `${FORMAT_DATE} — début`, autoFocus: true, onChange: setText })}
        {field({ label: "Au", value: second, placeholder: `${FORMAT_DATE} — fin`, onChange: setSecond })}
      </div>
    );
    entries = [{ kind: "item", id: "appliquer", label: "Appliquer la période", keepOpen: true, onSelect: applyRange }];
  } else if (isListKey(level)) {
    title = FIELD_LABELS[level];
    const selected: readonly string[] = filters[LIST_KEYS[level]];
    entries = options(level).map((option) => ({
      kind: "item",
      id: `valeur-${option.value}`,
      label: option.label,
      checked: selected.includes(option.value),
      ...(option.glyph ? { leading: <StatusGlyph tone={option.glyph} /> } : {}),
      keepOpen: true,
      onSelect: () => onApply(toggleValue(filters, level, option.value)),
    }));
  } else if (level === "company") {
    title = "Entreprise";
    content = (
      <div className="px-[9px] pb-1.5">
        {field({ label: "Rechercher une entreprise", value: text, placeholder: "Rechercher…", autoFocus: true, onChange: setText })}
      </div>
    );
    const items = companies.data?.items ?? [];
    entries =
      items.length === 0
        ? [
            {
              kind: "item",
              id: "aucune",
              label: companies.isPending ? "Chargement…" : "Aucune entreprise",
              disabled: true,
              onSelect: () => undefined,
            },
          ]
        : items.map((company) => ({
            kind: "item",
            id: `entreprise-${company.id}`,
            label: company.label,
            ...(company.meta ? { meta: company.meta } : {}),
            checked: filters.company_id === company.id,
            onSelect: () => onApply({ ...filters, company_id: company.id }),
          }));
  } else if (level === "sent") {
    title = "Envoyée";
    entries = [
      ...SENT_PRESETS.map(
        (days): MenuEntry => ({
          kind: "item",
          id: `plus-${days}`,
          label: `il y a plus de ${days} jours`,
          onSelect: () => onApply({ ...filters, start_date: null, end_date: daysAgo(days) }),
        }),
      ),
      { kind: "separator", id: "sep" },
      ...([7, 30] as const).map(
        (days): MenuEntry => ({
          kind: "item",
          id: `moins-${days}`,
          label: `ces ${days} derniers jours`,
          onSelect: () => onApply({ ...filters, start_date: daysAgo(days), end_date: null }),
        }),
      ),
      { kind: "item", id: "periode", label: "Entre deux dates…", keepOpen: true, onSelect: () => goTo("sent_range") },
    ];
  } else if (level === "hours") {
    title = "Heures par semaine";
    content = (
      <div className="flex gap-1.5 px-[9px] pb-1.5">
        {field({ label: "Minimum d'heures", value: text, placeholder: "min", inputMode: "decimal", autoFocus: true, onChange: setText })}
        {field({ label: "Maximum d'heures", value: second, placeholder: "max", inputMode: "decimal", onChange: setSecond })}
      </div>
    );
    entries = [{ kind: "item", id: "appliquer", label: "Appliquer", keepOpen: true, onSelect: applyHours }];
  } else {
    const key = level;
    title = FIELD_LABELS[key];
    content = (
      <div className="px-[9px] pb-1.5">
        {field({
          label: FIELD_LABELS[key],
          value: text,
          placeholder: key === "city" ? "Rennes…" : "Développeur…",
          autoFocus: true,
          onChange: setText,
        })}
      </div>
    );
    entries = [
      {
        kind: "item",
        id: "appliquer",
        label: text.trim() ? `${key === "city" ? "Ville" : "Poste"} : « ${text.trim()} »` : "Appliquer",
        keepOpen: true,
        onSelect: () => applyText(key),
      },
    ];
  }

  return (
    <Menu
      open={open}
      anchor={anchor}
      label="Ajouter un filtre"
      width={level === "company" || level === "sent_range" ? 280 : 236}
      entries={entries}
      onClose={onClose}
      {...(level === null ? {} : { onBack: () => goTo(level === "sent_range" ? "sent" : null) })}
      header={
        <>
          {header(title.toUpperCase(), level !== null)}
          {content}
          {errorLine}
        </>
      }
    />
  );
}
