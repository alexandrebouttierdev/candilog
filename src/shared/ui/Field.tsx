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
 * Liste déroulante.
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
            dense && "h-control min-h-control rounded-button bg-fill",
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
  variant = "field",
  className,
  ...props
}: Omit<InputHTMLAttributes<HTMLInputElement>, "onChange" | "value"> & {
  value: string;
  onValueChange: (value: string) => void;
  placeholder: string;
  /** `toolbar` : recherche de barre d'outils du design — 23 px, fond `bg-elev`, sans contour. */
  variant?: "field" | "toolbar";
}) {
  const toolbar = variant === "toolbar";

  return (
    <div className={cn("relative min-w-0", toolbar && "w-full min-w-[150px] max-w-[300px] flex-[0_1_220px]", className)}>
      <Icon
        name="search"
        size={toolbar ? 13 : 15}
        className={cn(
          "pointer-events-none absolute top-1/2 -translate-y-1/2",
          toolbar ? "left-2 text-tx-5" : "left-2.5 text-tx-5",
        )}
      />
      <input
        type="search"
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        placeholder={placeholder}
        aria-label={placeholder}
        className={controlClasses(
          false,
          toolbar
            ? "h-[23px] min-h-[23px] rounded-r6 border-transparent bg-elev py-0 pr-2 pl-6 text-small focus:border-ac"
            : "pl-8",
        )}
        {...props}
      />
    </div>
  );
}
