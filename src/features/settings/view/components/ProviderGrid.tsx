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

/**
 * Tuiles de fournisseur : logo, nom, et sélection portée par la tuile elle-même.
 *
 * Une tuile bordée dit qu'elle se clique ; l'ancienne grille sans filet laissait sept logos
 * de 18 px flotter sur toute la largeur et ne se distinguait d'une légende que par le
 * curseur. La sélection reprend le couple `accent-border` / `accent-tint` des items actifs.
 */
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
      className="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(112px,1fr))]"
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
              "flex min-w-0 flex-col items-center gap-2 rounded-tile border px-2 py-3",
              "transition-[background-color,border-color] duration-hover ease-in-out",
              "focus-visible:outline-1 focus-visible:outline-accent-focus",
              selected
                ? "border-accent-border bg-accent-tint"
                : "border-control bg-fill hover:bg-fill-hover",
            )}
          >
            <span
              className="flex size-9 flex-none items-center justify-center rounded-control bg-surface"
            >
              <img
                src={logo.src}
                alt=""
                width={20}
                height={20}
                className={cn("size-5", logo.mono && "dark:invert")}
              />
            </span>
            <span
              className={cn(
                "w-full truncate text-center text-label font-mid",
                selected ? "text-accent" : "text-ink-muted",
              )}
            >
              {fournisseur.label}
            </span>
          </button>
        );
      })}
    </div>
  );
}
