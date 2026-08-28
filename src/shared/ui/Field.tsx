import type { InputHTMLAttributes, SelectHTMLAttributes, TextareaHTMLAttributes } from "react";
import { controlClasses } from "./FormField";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/**
 * Contrôles de saisie stylés aux jetons.
 *
 * Ils restent des `input` / `select` / `textarea` natifs : les composants de saisie
 * réimplémentés perdent la navigation clavier, la saisie vocale et la restitution par les
 * lecteurs d'écran, que le guide exige toutes.
 */

export function TextInput({
  invalid,
  className,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return <input className={controlClasses(invalid, className)} {...props} />;
}

export function TextArea({
  invalid,
  className,
  rows = 3,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { invalid?: boolean }) {
  return (
    <textarea
      rows={rows}
      className={controlClasses(invalid, cn("py-2.5 leading-relaxed", className))}
      {...props}
    />
  );
}

/**
 * List déroulante.
 *
 * Le chevron natif est remplacé par l'icône `expand_more` des maquettes : celui de la
 * plateforme varie d'un système à l'autre et casse l'alignement de la barre de filtres.
 */
export function Select({
  invalid,
  dense = false,
  className,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement> & {
  invalid?: boolean;
  /** Gabarit des barres d'en-tête : 33 px sur fond surface, comme les boutons voisins. */
  dense?: boolean;
}) {
  return (
    <div className={cn("relative min-w-0", className)}>
      <select
        className={controlClasses(
          invalid,
          cn(
            "appearance-none pr-9",
            dense && "h-control min-h-control rounded-button bg-surface shadow-e1",
          ),
        )}
        {...props}
      >
        {children}
      </select>
      <Icon
        name="expand_more"
        size={17}
        className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-ink-faint"
      />
    </div>
  );
}

/** Champ de recherche à icône, réutilisé par les listes maîtresses et les bibliothèques. */
export function SearchInput({
  value,
  onValueChange,
  placeholder,
  className,
  ...props
}: Omit<InputHTMLAttributes<HTMLInputElement>, "onChange" | "value"> & {
  value: string;
  onValueChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <div className={cn("relative min-w-0", className)}>
      <Icon
        name="search"
        size={16}
        className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-ink-faint"
      />
      <input
        type="search"
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        placeholder={placeholder}
        aria-label={placeholder}
        className={controlClasses(false, "pl-9")}
        {...props}
      />
    </div>
  );
}
