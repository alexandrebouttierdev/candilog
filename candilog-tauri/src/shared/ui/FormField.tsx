import type { ReactNode } from "react";
import { useId } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/**
 * Libellé, champ, aide et erreur d'un champ de formulaire.
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
  children,
}: {
  label: string;
  required?: boolean;
  help?: string | undefined;
  error?: string | undefined;
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
    <div className="flex min-w-0 flex-col gap-1">
      <label htmlFor={id} className="text-label text-ink-muted">
        {label}
        {required ? <span className="ml-0.5 text-danger">*</span> : null}
      </label>

      {children({ id, "aria-describedby": describedBy, "aria-invalid": Boolean(error) })}

      {error ? (
        <p id={`${id}-error`} className="flex items-center gap-1 text-meta text-danger">
          <Icon name="error" size={13} />
          {error}
        </p>
      ) : help ? (
        <p id={`${id}-help`} className="text-meta text-ink-faint">
          {help}
        </p>
      ) : null}
    </div>
  );
}

/** Classes communes aux contrôles de saisie : hauteur 36 px, rayon 9 px, focus accent. */
export function controlClasses(invalid = false, extra?: string): string {
  return cn(
    "min-h-field w-full rounded-field border bg-surface px-3 text-body text-ink",
    "placeholder:text-ink-faint",
    "transition-[border-color,background-color] duration-150",
    "disabled:cursor-not-allowed disabled:bg-neutral-tint disabled:text-ink-faint",
    invalid ? "border-danger" : "border-line focus:border-accent",
    extra,
  );
}
