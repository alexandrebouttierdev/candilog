import { Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import {
  FOURNISSEURS,
  idProvider,
  type FournisseurOption,
} from "../../model/providers";
import type { ProviderKind } from "@/shared/types/generated/settings";

/** Grid de choix du fournisseur IA : des cartes, pas une liste déroulante. */
export function ProviderGrid({
  value,
  onChange,
}: {
  value: ProviderKind;
  onChange: (id: FournisseurOption["id"]) => void;
}) {
  const actif = idProvider(value);

  return (
    <div
      role="radiogroup"
      aria-label="Fournisseur IA"
      className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4 [grid-template-columns:repeat(auto-fit,minmax(min(140px,100%),1fr))]"
    >
      {FOURNISSEURS.map((fournisseur) => {
        const selected = fournisseur.id === actif;
        return (
          <button
            key={fournisseur.id}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={fournisseur.label}
            onClick={() => onChange(fournisseur.id)}
            className={cn(
              "flex min-w-0 items-center gap-[9px] rounded-field border px-3 py-[11px] text-left",
              "transition-[background-color,border-color] duration-150",
              selected
                ? "border-accent-border bg-accent-tint"
                : "border-line bg-surface hover:bg-neutral-tint",
            )}
          >
            <span
              className={cn(
                "flex size-[26px] flex-none items-center justify-center rounded-control",
                selected ? "bg-accent text-white" : "bg-neutral-tint text-ink-muted",
              )}
            >
              <Icon name={fournisseur.icon} size={15} filled={selected} />
            </span>
            <span className="min-w-0 flex-1 truncate text-body font-mid text-ink">
              {fournisseur.label}
            </span>
            {selected ? <Icon name="check" size={16} className="ml-auto flex-none text-accent" /> : null}
          </button>
        );
      })}
    </div>
  );
}
