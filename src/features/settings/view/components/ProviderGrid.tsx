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

export function logoFournisseur(id: FournisseurOption["id"]) {
  return LOGOS[id];
}

export function defFournisseur(provider: ProviderKind): FournisseurOption {
  const id = idProvider(provider);
  return FOURNISSEURS.find((item) => item.id === id) ?? FOURNISSEURS[0]!;
}

/** Tuiles logo du fournisseur : sélection par barre d'accent, pas de cartes. */
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
      className="grid grid-cols-4 gap-1.5 min-[720px]:grid-cols-7"
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
              "flex min-w-0 flex-col items-center gap-1.5 rounded-none px-1.5 py-2",
              "transition-colors duration-hover",
              selected ? "row-selected" : "hover:bg-surface-hover",
            )}
          >
            <span className="flex size-8 flex-none items-center justify-center rounded-control bg-fill">
              <img
                src={logo.src}
                alt={fournisseur.label}
                width={18}
                height={18}
                className={cn("size-[18px]", logo.mono && "dark:invert")}
              />
            </span>
            <span className="w-full truncate text-center text-meta text-ink">{fournisseur.label}</span>
          </button>
        );
      })}
    </div>
  );
}
