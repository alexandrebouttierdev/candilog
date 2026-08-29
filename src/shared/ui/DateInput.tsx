import {
  useRef,
  type ChangeEvent,
  type InputHTMLAttributes,
  type Ref,
} from "react";
import { controlClasses } from "./FormField";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import { versDateAffichee, versDateIso } from "@/shared/lib/dates";

/**
 * Champ date (ou heure) : saisie texte **et** sélecteur natif.
 *
 * Le texte reste la valeur du formulaire (`JJ-MM-AAAA` / `HH:MM`) pour ne pas casser les
 * schémas Zod. Le `input` natif n'a pas de `name` : il ne part pas en double à la soumission.
 * Son indicateur est étiré sur l'icône, le seul moyen fiable d'ouvrir le calendrier WebKit
 * sans `showPicker()` — absent ou capricieux selon le WebView Tauri.
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

const overlay =
  "absolute inset-y-0 right-0 z-10 w-9 cursor-pointer border-0 bg-transparent p-0 text-transparent " +
  "[&::-webkit-calendar-picker-indicator]:absolute [&::-webkit-calendar-picker-indicator]:inset-0 " +
  "[&::-webkit-calendar-picker-indicator]:h-full [&::-webkit-calendar-picker-indicator]:w-full " +
  "[&::-webkit-calendar-picker-indicator]:cursor-pointer [&::-webkit-calendar-picker-indicator]:opacity-0";

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
  const saisie = typeof value === "string" ? value : String(defaultValue ?? "");
  const iso = versDateIso(saisie) ?? "";

  return (
    <div className={cn("relative min-w-0", className)}>
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
      <input
        type="date"
        tabIndex={-1}
        aria-label="Choisir une date"
        value={iso}
        onChange={(event) => {
          const next = event.target.value;
          emit(onChange, textRef.current, name, next ? versDateAffichee(next) : "");
        }}
        className={cn(overlay, dense && "w-7")}
      />
      <Icon
        name="calendar_month"
        size={dense ? 14 : 17}
        className="pointer-events-none absolute top-1/2 right-2.5 z-0 -translate-y-1/2 text-ink-faint"
      />
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
  const saisie = typeof value === "string" ? value : String(defaultValue ?? "");
  const native = heureNative(saisie);

  return (
    <div className={cn("relative min-w-0", className)}>
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
      <input
        type="time"
        step={60}
        tabIndex={-1}
        aria-label="Choisir une heure"
        value={native}
        onChange={(event) => {
          emit(onChange, textRef.current, name, heureSaisie(event.target.value));
        }}
        className={cn(overlay, dense && "w-7")}
      />
      <Icon
        name="schedule"
        size={dense ? 14 : 17}
        className="pointer-events-none absolute top-1/2 right-2.5 z-0 -translate-y-1/2 text-ink-faint"
      />
    </div>
  );
}

function denseClasses(invalid: boolean | undefined): string {
  return cn(
    "h-[25px] min-h-[25px] w-full rounded-chip border bg-fill px-2 pr-7 text-label text-ink",
    "placeholder:text-ink-disabled focus:border-accent focus:outline-none",
    invalid ? "border-danger" : "border-control",
  );
}

/** `HH:MM` si la saisie est une heure réelle, sinon vide pour le `input type="time"`. */
function heureNative(saisie: string): string {
  return /^([01]\d|2[0-3]):[0-5]\d/.test(saisie.trim()) ? saisie.trim().slice(0, 5) : "";
}

/** Le sélecteur natif envoie parfois `HH:MM:SS` : le schéma n'accepte que `HH:MM`. */
function heureSaisie(native: string): string {
  return native.slice(0, 5);
}
