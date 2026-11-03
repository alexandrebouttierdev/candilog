import { cn } from "@/shared/lib/cn";
import { LineIcon } from "@/shared/ui";
import type { ProviderOption } from "../../model/providers";
import { providerLogo } from "./ProviderGrid";

/**
 * Liste compacte de modèles distants : logo du fournisseur, identifiant, état sélectionné.
 */
export function RemoteModelPicker({
  models,
  value,
  onChange,
  providerLabel,
  providerId,
}: {
  models: string[];
  value: string;
  onChange: (model: string) => void;
  /** Libellé du fournisseur pour le aria-label du groupe. */
  providerLabel: string;
  /** Identifiant fournisseur pour afficher son logo sur chaque tuile. */
  providerId: ProviderOption["id"];
}) {
  if (models.length === 0) return null;
  const logo = providerLogo(providerId);

  return (
    <div
      role="radiogroup"
      aria-label={`Modèles ${providerLabel}`}
      className="flex flex-col gap-1.5"
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
              "relative flex min-h-10 items-center gap-2.5 rounded-button border px-3 py-2 text-left",
              "transition-[background-color,border-color] duration-hover",
              "focus-visible:outline-1 focus-visible:outline-accent-focus",
              selected
                ? "border-ac bg-accent-tint-12"
                : "border-bd bg-panel hover:border-control-strong hover:bg-chip",
            )}
          >
            <span
              className={cn(
                "flex size-7 flex-none items-center justify-center rounded-tile",
                selected ? "bg-panel" : "bg-chip",
              )}
              aria-hidden="true"
            >
              {logo ? (
                <img
                  src={logo.src}
                  alt=""
                  width={16}
                  height={16}
                  className={cn("size-4 object-contain", logo.mono && "dark:invert")}
                />
              ) : (
                <LineIcon name="ai" size={15} className="text-tx-3" />
              )}
            </span>
            <span
              className={cn(
                "min-w-0 flex-1 truncate font-mono text-note leading-snug",
                selected ? "font-mid text-tx" : "text-tx-3",
              )}
            >
              {model}
            </span>
            {selected ? (
              <span aria-hidden className="flex-none text-ac-tx">
                ✓
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}
