import { Icon } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import {
  FOURNISSEURS,
  idProvider,
  type FournisseurOption,
} from "../../model/providers";
import type { ProviderKind } from "@/shared/types/generated/settings";
import logoOllama from "@/assets/providers/ollama.svg";
import logoClaude from "@/assets/providers/claude.svg";
import logoOpenai from "@/assets/providers/openai.svg";
import logoGemini from "@/assets/providers/googlegemini.svg";
import logoMistral from "@/assets/providers/mistralai.svg";
import logoNvidia from "@/assets/providers/nvidia.svg";
import logoCustom from "@/assets/providers/custom.svg";

const LOGOS: Record<FournisseurOption["id"], { src: string; mono: boolean }> = {
  ollama: { src: logoOllama, mono: true },
  claude: { src: logoClaude, mono: false },
  openai: { src: logoOpenai, mono: true },
  gemini: { src: logoGemini, mono: false },
  mistral: { src: logoMistral, mono: false },
  nvidia: { src: logoNvidia, mono: false },
  custom: { src: logoCustom, mono: true },
};

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
        const logo = LOGOS[fournisseur.id];
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
            <span className="flex size-8 flex-none items-center justify-center rounded-control bg-surface ring-1 ring-inset ring-line">
              <img
                src={logo.src}
                alt={fournisseur.label}
                width={20}
                height={20}
                className={cn("size-5", logo.mono && "dark:invert")}
              />
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
