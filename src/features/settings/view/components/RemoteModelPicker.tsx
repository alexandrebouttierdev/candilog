import { cn } from "@/shared/lib/cn";
import { Icon } from "@/shared/ui";

/**
 * Grille de modèles distants : chaque modèle est une tuile sélectionnable.
 * Même langage visuel que les cartes locales — filet, teinte accent, coche.
 */
export function RemoteModelPicker({
  models,
  value,
  onChange,
  providerLabel,
}: {
  models: string[];
  value: string;
  onChange: (model: string) => void;
  /** Libellé du fournisseur pour le aria-label du groupe. */
  providerLabel: string;
}) {
  if (models.length === 0) return null;

  return (
    <div
      role="radiogroup"
      aria-label={`Modèles ${providerLabel}`}
      className="grid gap-2 [grid-template-columns:repeat(auto-fill,minmax(200px,1fr))]"
    >
      {models.map((model) => {
        const selected = model === value;
        return (
          <button
            key={model}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={model}
            onClick={() => onChange(model)}
            className={cn(
              "relative flex min-h-[72px] flex-col items-start gap-1.5 rounded-card border px-3 py-2.5 text-left",
              "transition-[background-color,border-color] duration-hover",
              "focus-visible:outline-1 focus-visible:outline-accent-focus",
              selected
                ? "border-accent bg-accent-tint-12"
                : "border-line bg-surface hover:border-control-strong hover:bg-fill",
            )}
          >
            {selected ? (
              <Icon
                name="check_circle"
                size={16}
                filled
                className="absolute right-2 top-2 text-accent"
              />
            ) : null}
            <span
              className={cn(
                "flex size-8 flex-none items-center justify-center rounded-tile",
                selected ? "bg-surface" : "bg-fill",
              )}
              aria-hidden="true"
            >
              <Icon
                name="smart_toy"
                size={16}
                className={selected ? "text-accent" : "text-ink-muted"}
              />
            </span>
            <span
              className={cn(
                "w-full pr-5 break-all font-mono text-label leading-snug",
                selected ? "font-mid text-ink" : "text-ink-muted",
              )}
            >
              {model}
            </span>
            {selected ? (
              <span className="text-eyebrow uppercase text-accent">Sélectionné</span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}
