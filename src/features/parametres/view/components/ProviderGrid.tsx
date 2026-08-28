import { Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import {
  FOURNISSEURS,
  identifiantProvider,
  type FournisseurOption,
} from "../../model/providers";
import type { ProviderKind } from "@/shared/types/generated/parametres";

/** Grille de choix du fournisseur IA : des cartes, pas une liste déroulante. */
export function ProviderGrid({
  value,
  onChange,
}: {
  value: ProviderKind;
  onChange: (id: FournisseurOption["id"]) => void;
}) {
  const actif = identifiantProvider(value);

  return (
    <div
      role="radiogroup"
      aria-label="Fournisseur IA"
      className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4"
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
              "flex min-h-11 items-start gap-3 rounded-card border px-3 py-3 text-left",
              "transition-[background-color,border-color,color] duration-150",
              selected
                ? "border-accent bg-accent-tint text-ink"
                : "border-line bg-surface text-ink hover:bg-neutral-tint",
            )}
          >
            <Icon
              name={fournisseur.icon}
              size={18}
              filled={selected}
              className={selected ? "text-accent" : "text-ink-muted"}
            />
            <span>
              <span className="block text-label font-semibold">{fournisseur.label}</span>
              <span className="mt-0.5 block text-meta text-ink-muted">{fournisseur.hint}</span>
            </span>
          </button>
        );
      })}
    </div>
  );
}
