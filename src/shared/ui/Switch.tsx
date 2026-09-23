import { cn } from "@/shared/lib/cn";

/**
 * Interrupteur (`COMPONENTS.md` §11 du design) : piste `bg-chip`, active en `ac`.
 *
 * Un vrai `button` à rôle `switch` : `Espace` et `⏎` le basculent, et son état est annoncé
 * par les lecteurs d'écran. Le libellé visible est rendu à côté par l'appelant, relié par
 * `label` (nom accessible).
 */
export function Switch({
  checked,
  onChange,
  label,
  disabled = false,
  className,
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  disabled?: boolean;
  className?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        "relative inline-flex h-4 w-[27px] flex-none items-center rounded-full transition-color",
        checked ? "bg-ac" : "bg-chip",
        "disabled:cursor-default disabled:opacity-50",
        className,
      )}
    >
      <span
        aria-hidden
        className={cn(
          "absolute size-3 rounded-full bg-white shadow-sm transition-transform duration-[var(--dur-color)]",
          checked ? "translate-x-[13px]" : "translate-x-0.5",
        )}
      />
    </button>
  );
}
