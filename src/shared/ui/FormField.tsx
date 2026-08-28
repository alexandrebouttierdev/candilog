import type { ReactNode } from "react";
import { useId } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/**
 * Libellé, champ, aide et erreur d'un champ de formulaire.
 *
 * Géométrie des maquettes : libellé 11,5 px/550 gris moyen à 6 px du champ, astérisque
 * rouge pour le requis, aide ou erreur en 11 px précédée d'une icône de 14 px.
 *
 * L'erreur est rendue **sous le champ** et non dans une infobulle : le guide l'exige, et une
 * infobulle seule est invisible au clavier comme au lecteur d'écran. `aria-describedby` et
 * `aria-invalid` sont câblés ici plutôt que laissés à chaque formulaire, où ils finiraient
 * par manquer sur la moitié des champs.
 */
export function FormField({
  label,
  required = false,
  help,
  error,
  className,
  children,
}: {
  label: string;
  required?: boolean;
  help?: string | undefined;
  error?: string | undefined;
  className?: string;
  /** Reçoit les attributs à poser sur le contrôle : identifiant et description. */
  children: (props: {
    id: string;
    "aria-describedby": string | undefined;
    "aria-invalid": boolean;
  }) => ReactNode;
}) {
  const id = useId();
  const describedBy = error ? `${id}-error` : help ? `${id}-help` : undefined;

  return (
    <div className={cn("flex min-w-0 flex-col", className)}>
      <label
        htmlFor={id}
        className="mb-1.5 flex items-center gap-[5px] text-label font-mid text-ink-muted"
      >
        {label}
        {required ? <span className="font-normal text-danger">*</span> : null}
      </label>

      {children({ id, "aria-describedby": describedBy, "aria-invalid": Boolean(error) })}

      {error ? (
        <p
          id={`${id}-error`}
          className="mt-1.5 flex items-center gap-[5px] text-meta leading-[1.45] text-danger"
        >
          <Icon name="error" size={14} className="flex-none" />
          {error}
        </p>
      ) : help ? (
        <p
          id={`${id}-help`}
          className="mt-1.5 flex items-center gap-[5px] text-meta leading-[1.45] text-ink-faint"
        >
          <Icon name="info" size={14} className="flex-none" />
          {help}
        </p>
      ) : null}
    </div>
  );
}

/**
 * Classes communes aux contrôles de saisie.
 *
 * Les maquettes posent le champ sur le fond de page, pas sur la surface : dans une modale
 * blanche, un champ blanc ne se distinguerait que par son filet. Le focus remonte le fond
 * en surface, passe le filet en accent et ajoute un halo de 3 px en teinte accent.
 */
export function controlClasses(invalid = false, extra?: string): string {
  return cn(
    "min-h-field w-full rounded-field border bg-page px-3 text-body text-ink",
    "placeholder:text-ink-faint",
    "transition-[border-color,background-color,box-shadow] duration-150",
    "disabled:cursor-not-allowed disabled:bg-neutral-tint disabled:text-ink-faint",
    "focus:bg-surface focus:shadow-[0_0_0_3px_var(--color-accent-tint)] focus:outline-none",
    invalid ? "border-danger focus:border-danger" : "border-line focus:border-accent",
    extra,
  );
}
