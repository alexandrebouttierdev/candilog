import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type ChangeEvent,
  type InputHTMLAttributes,
  type Ref,
} from "react";
import { controlClasses } from "./FormField";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import { versDateAffichee, versDateIso } from "@/shared/lib/dates";
import { useDismissable } from "@/shared/hooks/useDismissable";
import {
  DAYS,
  dateFromIso,
  decalerMonth,
  gridDuMonth,
  labelDay,
  monthLabel,
} from "@/features/calendar/model/month";

/**
 * Champ date (ou heure) : saisie texte **et** popover.
 *
 * Le calendrier natif WebKit (WebView Tauri) ignore le clic à l'extérieur : on rend le
 * sélecteur nous-mêmes, avec la même fermeture que le menu Filtres (mousedown hors racine,
 * Échap). Le texte reste la valeur du formulaire (`JJ-MM-AAAA` / `HH:MM`).
 */

type ChampProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & {
  invalid?: boolean;
  /** Gabarit 25 px des champs du popover Filtres. */
  dense?: boolean;
  ref?: Ref<HTMLInputElement>;
};

function assignRef<T>(ref: Ref<T> | undefined, node: T | null) {
  if (!ref) return;
  if (typeof ref === "function") ref(node);
  else (ref as { current: T | null }).current = node;
}

function setNativeValue(element: HTMLInputElement, value: string) {
  const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
  descriptor?.set?.call(element, value);
}

function emit(
  onChange: ChampProps["onChange"],
  element: HTMLInputElement | null,
  name: string | undefined,
  value: string,
) {
  if (element) setNativeValue(element, value);
  onChange?.({
    target: element ?? { value, name },
    currentTarget: element ?? { value, name },
  } as ChangeEvent<HTMLInputElement>);
}

function useFermeDehors(open: boolean, onClose: () => void) {
  const root = useRef<HTMLDivElement>(null);
  useDismissable({ open, onDismiss: onClose });
  useEffect(() => {
    if (!open) return;
    const onPointer = (event: MouseEvent) => {
      if (!root.current?.contains(event.target as Node)) onClose();
    };
    document.addEventListener("mousedown", onPointer);
    return () => document.removeEventListener("mousedown", onPointer);
  }, [open, onClose]);
  return root;
}

function moisDe(saisie: string): { year: number; month: number } {
  const iso = versDateIso(saisie);
  const date = iso ? dateFromIso(iso) : new Date();
  return { year: date.getFullYear(), month: date.getMonth() };
}

export function DateInput({
  invalid,
  dense = false,
  className,
  value,
  defaultValue,
  onChange,
  placeholder = "JJ-MM-AAAA",
  ref,
  name,
  ...props
}: ChampProps) {
  const textRef = useRef<HTMLInputElement>(null);
  const [open, setOpen] = useState(false);
  const fermer = useCallback(() => setOpen(false), []);
  const root = useFermeDehors(open, fermer);
  const saisie = typeof value === "string" ? value : String(defaultValue ?? "");
  const choisi = versDateIso(saisie);
  const [curseur, setCurseur] = useState(() => moisDe(saisie));
  const cells = gridDuMonth(curseur.year, curseur.month);

  return (
    <div ref={root} className={cn("relative min-w-0", className)}>
      <input
        {...props}
        ref={(node) => {
          textRef.current = node;
          assignRef(ref, node);
        }}
        type="text"
        name={name}
        inputMode="numeric"
        autoComplete="off"
        spellCheck={false}
        placeholder={placeholder}
        {...(value !== undefined ? { value } : { defaultValue })}
        onChange={onChange}
        className={dense ? denseClasses(invalid) : controlClasses(invalid, "pr-9")}
      />
      <button
        type="button"
        aria-label="Choisir une date"
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => {
          if (!open) setCurseur(moisDe(saisie));
          setOpen((actuel) => !actuel);
        }}
        className={declencheur(dense)}
      >
        <Icon name="calendar_month" size={dense ? 14 : 17} />
      </button>
      {open ? (
        <div
          role="dialog"
          aria-label="Calendrier"
          className="glass-popover absolute top-[calc(100%+6px)] right-0 z-50 w-[252px] rounded-overlay border border-overlay p-2.5 shadow-overlay"
        >
          <div className="mb-2 flex items-center gap-1">
            <button
              type="button"
              aria-label="Mois précédent"
              onClick={() => setCurseur((c) => decalerMonth(c.year, c.month, -1))}
              className="flex size-7 items-center justify-center rounded-button text-ink-muted hover:bg-fill-hover hover:text-ink"
            >
              <Icon name="chevron_left" size={18} />
            </button>
            <p className="min-w-0 flex-1 text-center text-note font-semibold capitalize text-ink">
              {monthLabel(curseur.year, curseur.month)}
            </p>
            <button
              type="button"
              aria-label="Mois suivant"
              onClick={() => setCurseur((c) => decalerMonth(c.year, c.month, 1))}
              className="flex size-7 items-center justify-center rounded-button text-ink-muted hover:bg-fill-hover hover:text-ink"
            >
              <Icon name="chevron_right" size={18} />
            </button>
          </div>
          <div className="grid grid-cols-7">
            {DAYS.map((day) => (
              <span
                key={day}
                className="pb-1 text-center text-micro uppercase text-ink-faint"
              >
                {day}
              </span>
            ))}
            {cells.map((day) => (
              <button
                key={day.iso}
                type="button"
                aria-label={labelDay(day.iso)}
                aria-current={day.iso === choisi ? "date" : undefined}
                onClick={() => {
                  emit(onChange, textRef.current, name, versDateAffichee(day.iso));
                  fermer();
                }}
                className={cn(
                  "tabular mx-auto flex size-7 items-center justify-center rounded-pill text-meta",
                  day.iso === choisi
                    ? "bg-accent font-medium text-on-accent"
                    : day.today
                      ? "font-medium text-accent hover:bg-accent-tint"
                      : day.in_month
                        ? "text-ink hover:bg-fill-hover"
                        : "text-ink-faint hover:bg-fill-hover",
                )}
              >
                {day.number}
              </button>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}

export function TimeInput({
  invalid,
  dense = false,
  className,
  value,
  defaultValue,
  onChange,
  placeholder = "HH:MM",
  ref,
  name,
  ...props
}: ChampProps) {
  const textRef = useRef<HTMLInputElement>(null);
  const [open, setOpen] = useState(false);
  const fermer = useCallback(() => setOpen(false), []);
  const root = useFermeDehors(open, fermer);
  const saisie = typeof value === "string" ? value : String(defaultValue ?? "");
  const actuel = heureNative(saisie) || "14:00";
  const [heures, minutes] = actuel.split(":");

  const appliquer = (h: string, m: string) => {
    emit(onChange, textRef.current, name, `${h}:${m}`);
  };

  return (
    <div ref={root} className={cn("relative min-w-0", className)}>
      <input
        {...props}
        ref={(node) => {
          textRef.current = node;
          assignRef(ref, node);
        }}
        type="text"
        name={name}
        inputMode="numeric"
        autoComplete="off"
        spellCheck={false}
        placeholder={placeholder}
        {...(value !== undefined ? { value } : { defaultValue })}
        onChange={onChange}
        className={dense ? denseClasses(invalid) : controlClasses(invalid, "pr-9")}
      />
      <button
        type="button"
        aria-label="Choisir une heure"
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => setOpen((actuel) => !actuel)}
        className={declencheur(dense)}
      >
        <Icon name="schedule" size={dense ? 14 : 17} />
      </button>
      {open ? (
        <div
          role="dialog"
          aria-label="Horloge"
          className="glass-popover absolute top-[calc(100%+6px)] right-0 z-50 flex items-center gap-1.5 rounded-overlay border border-overlay p-2.5 shadow-overlay"
        >
          <select
            aria-label="Heure du jour"
            value={heures}
            onChange={(event) => appliquer(event.target.value, minutes ?? "00")}
            className={controlClasses(false, "h-control min-h-control w-[4.25rem] appearance-none px-2")}
          >
            {Array.from({ length: 24 }, (_, h) => {
              const v = String(h).padStart(2, "0");
              return (
                <option key={v} value={v}>
                  {v}
                </option>
              );
            })}
          </select>
          <span className="text-body text-ink-faint">:</span>
          <select
            aria-label="Minutes"
            value={minutes}
            onChange={(event) => appliquer(heures ?? "00", event.target.value)}
            className={controlClasses(false, "h-control min-h-control w-[4.25rem] appearance-none px-2")}
          >
            {Array.from({ length: 60 }, (_, m) => {
              const v = String(m).padStart(2, "0");
              return (
                <option key={v} value={v}>
                  {v}
                </option>
              );
            })}
          </select>
        </div>
      ) : null}
    </div>
  );
}

function declencheur(dense: boolean): string {
  return cn(
    "absolute inset-y-0 right-0 z-10 flex items-center justify-center text-ink-faint",
    "hover:text-ink focus-visible:outline-1 focus-visible:outline-accent-focus",
    dense ? "w-7" : "w-9",
  );
}

function denseClasses(invalid: boolean | undefined): string {
  return cn(
    "h-[25px] min-h-[25px] w-full rounded-chip border bg-fill px-2 pr-7 text-label text-ink",
    "placeholder:text-ink-disabled focus:border-accent focus:outline-none",
    invalid ? "border-danger" : "border-control",
  );
}

/** `HH:MM` si la saisie est une heure réelle, sinon vide. */
function heureNative(saisie: string): string {
  return /^([01]\d|2[0-3]):[0-5]\d/.test(saisie.trim()) ? saisie.trim().slice(0, 5) : "";
}
