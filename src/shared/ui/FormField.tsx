import type { ReactNode } from "react";
import { useId } from "react";
import { cn } from "@/shared/lib/cn";

/**
 * Libellé, champ, aide et erreur d'un champ de formulaire (`COMPONENTS.md` §2 du design).
 *
 * Libellé 11,5 px `tx-4` à 5 px du champ ; aide contextuelle à droite du libellé en
 * 10,5 px `tx-6` (`JJ-MM-AAAA`) ; un champ requis encore vide affiche la mention
 * `obligatoire` en `st-a` — pas de contour rouge. Une erreur de validation reste écrite
 * sous le champ, en clair : elle dit ce qui bloque et quoi faire.
 *
 * L'erreur est rendue **sous le champ** et non dans une infobulle : le guide l'exige, et une
 * infobulle seule est invisible au clavier comme au lecteur d'écran. `aria-describedby` et
 * `aria-invalid` sont câblés ici plutôt que laissés à chaque formulaire, où ils finiraient
 * par manquer sur la moitié des champs.
 */
export function FormField({
  label,
  required = false,
  missing = false,
  hint,
  help,
  error,
  className,
  children,
}: {
  label: string;
  required?: boolean;
  /** Champ requis encore vide : affiche la mention `obligatoire`. */
  missing?: boolean;
  /** Aide contextuelle courte, à droite du libellé (`JJ-MM-AAAA`, `vide si en poste`). */
  hint?: string | undefined;
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
      <div className="mb-[5px] flex items-baseline gap-1.5">
        <label htmlFor={id} className="text-sub text-tx-4">
          {label}
          {required ? <span className="sr-only"> (obligatoire)</span> : null}
        </label>
        {required && missing ? (
          <span aria-hidden className="text-caps text-st-a">
            obligatoire
          </span>
        ) : null}
        {hint ? <span className="ml-auto text-caps text-tx-6">{hint}</span> : null}
      </div>

      {children({ id, "aria-describedby": describedBy, "aria-invalid": Boolean(error) })}

      {error ? (
        <p
          id={`${id}-error`}
          className="mt-1.5 text-tiny leading-[1.45] text-st-c"
        >
          {error}
        </p>
      ) : help ? (
        <p
          id={`${id}-help`}
          className="mt-1.5 text-tiny leading-[1.45] text-tx-5"
        >
          {help}
        </p>
      ) : null}
    </div>
  );
}

/**
 * Classes communes aux contrôles de saisie : 30 px, rayon 7, fond `field-bg`, contour
 * `bd-menu` passant à `ac` au focus. Le texte indicatif montre un **format**
 * (`JJ-MM-AAAA`), jamais un nom propre crédible qu'on confondrait avec une donnée saisie.
 */
export function controlClasses(invalid = false, extra?: string): string {
  return cn(
    "min-h-input w-full rounded-r7 border bg-input px-2.5 text-ui text-tx",
    "placeholder:text-tx-6",
    "transition-color",
    "disabled:cursor-not-allowed disabled:text-tx-5",
    "read-only:text-tx-4",
    "focus:outline-none",
    invalid ? "border-st-c focus:border-st-c" : "border-bd-menu focus:border-ac",
    extra,
  );
}
